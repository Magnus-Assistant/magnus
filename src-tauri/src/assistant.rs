use crate::globals::{get_magnus_id, get_open_ai_key, get_reqwest_client, get_thread_id};
use crate::tools;
use reqwest::Error;
use std::thread;
use std::time::Duration;
use crossbeam::channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use reqwest::header::TRANSFER_ENCODING;
use opus::Decoder;
use ogg::reading::async_api::PacketReader;
use tokio_util::io::StreamReader;
use cpal::{SampleRate, SupportedStreamConfig};
use tokio_stream::StreamExt;

pub async fn run(output_stream_running: Arc<Mutex<bool>>, transcription_receiver: Receiver<String>, audio_output_sender: Sender<Vec<i16>>, audio_output_config: SupportedStreamConfig /* , assistant_response_sender: Sender<String>*/) {
    // continue to run as long as the output stream is running also
    while *output_stream_running.lock().unwrap() {
        // receive speech transcription from vosk
        if let Ok(transcription) = transcription_receiver.try_recv() {

            //could emit transcription here for the user

            let message = serde_json::json!({
                "role": "user",
                "content": transcription
            });

            println!("User Message in assistant run Fn: {}", message);

            // this is the create message we are using for audio input. Text is handle in main
            let _ = create_message(message, get_thread_id()).await;

            let run_id: String = create_run(get_thread_id())
                .await
                .unwrap_or_else(|err| {
                    panic!("Error occurred: {:?}", err);
                });
            
            let _ = run_and_wait(&run_id, get_thread_id()).await;

            let response = get_assistant_last_response(get_thread_id()).await.unwrap();
            //could emit the reponse from audio output here

            // speak response
            // TODO: Make this toggleable just like how main functions.
            // Maybe create a universal message struct that contains user and magnus messages and if it has TTS
            println!("Creating speech: {}", response);
            let _ = create_speech(response, audio_output_sender.clone(), audio_output_config.sample_rate(), audio_output_config.channels()).await;
        }
    }
} 

pub async fn create_message_thread() -> Result<String, Error> {
    let response = get_reqwest_client()
        .post("https://api.openai.com/v1/threads")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .header("OpenAI-Beta", "assistants=v1")
        .send()
        .await?;

    let thread = response.json::<serde_json::Value>().await?;

    Ok(thread["id"].to_string())
}

pub async fn create_message(user_message: serde_json::Value, thread_id: String) -> Result<(), Error> {
    get_reqwest_client()
        .post(format!(
            "https://api.openai.com/v1/threads/{}/messages",
            thread_id
        ))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .header("OpenAI-Beta", "assistants=v1")
        .json(&user_message)
        .send()
        .await?;
    println!("User: {}", user_message["content"]);
    Ok(())
}

pub async fn create_run(thread_id: String) -> Result<String, Error> {
    let data = serde_json::json!({
        "assistant_id": get_magnus_id()
    });

    let response = get_reqwest_client()
        .post(format!(
            "https://api.openai.com/v1/threads/{}/runs",
            thread_id
        ))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .header("OpenAI-Beta", "assistants=v1")
        .json(&data)
        .send()
        .await?;

    let run = response.json::<serde_json::Value>().await?;

    Ok(run["id"].to_string().trim_matches('\"').to_string())
}

pub async fn run_and_wait(run_id: &str, thread_id: String) -> Result<(), Error> {
    loop {
        let response = get_reqwest_client()
            .get(format!(
                "https://api.openai.com/v1/threads/{}/runs/{}",
                thread_id, run_id
            ))
            .header("Authorization", format!("Bearer {}", get_open_ai_key()))
            .header("OpenAI-Beta", "assistants=v1")
            .send()
            .await?;

        let run = response.json::<serde_json::Value>().await?;

        if run["status"] == "completed" {
            return Ok(());
        } else if run["status"] == "requires_action"
            && run["required_action"]["type"] == "submit_tool_outputs"
        {
            let mut tool_outputs: Vec<serde_json::Value> = vec![];

            if let Some(tool_calls) =
                run["required_action"]["submit_tool_outputs"]["tool_calls"].as_array()
            {
                for tool_call in tool_calls {
                    if let Some(tool_call_obj) = tool_call.as_object() {
                        // println!("\ntool_call:\n{:#?}", tool_call);

                        let function_name = &tool_call_obj["function"]["name"]
                            .to_string()
                            .trim_matches('"')
                            .to_string();

                        let tool_output: String;

                        let arguments = &tool_call_obj["function"]["arguments"].as_str().unwrap();

                        let arguments_object = serde_json::from_str::<
                            serde_json::Map<String, serde_json::Value>,
                        >(arguments);

                        match arguments_object {
                            Ok(args) => {
                                if args.is_empty() {
                                    tool_output = execute(function_name, None).await?;
                                } else {
                                    tool_output = execute(function_name, Some(args)).await?;
                                }
                            }
                            Err(_) => {
                                tool_output = "No arguments key found in tool call".to_string()
                            }
                        }

                        // println!("received output: {}\n for tool call: {}", tool_output, tool_call["id"]);

                        tool_outputs.push(serde_json::json!({
                            "tool_call_id": tool_call["id"],
                            "output": tool_output
                        }));
                    }
                }
                let _ = submit_tool_outputs(
                    run_id,
                    thread_id.clone(),
                    serde_json::json!({"tool_outputs": tool_outputs}),
                )
                .await;
            }
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }
}

pub async fn submit_tool_outputs(
    run_id: &str,
    thread_id: String,
    tool_outputs: serde_json::Value,
) -> Result<(), Error> {
    let _ = get_reqwest_client()
        .post(format!(
            "https://api.openai.com/v1/threads/{}/runs/{}/submit_tool_outputs",
            thread_id, run_id
        ))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .header("OpenAI-Beta", "assistants=v1")
        .json(&tool_outputs)
        .send()
        .await;

    Ok(())
}

pub async fn get_assistant_last_response(thread_id: String) -> Result<String, Error> {
    let response = get_reqwest_client()
        .get(format!(
            "https://api.openai.com/v1/threads/{}/messages",
            thread_id
        ))
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .header("OpenAI-Beta", "assistants=v1")
        .send()
        .await?;

    let messages = response.json::<serde_json::Value>().await?;

    let assistant_response = messages["data"][0]["content"][0]["text"]["value"].to_string();
    println!("Magnus: {assistant_response}");
    
    Ok(assistant_response)
}

pub async fn print_messages(thread_id: String) -> Result<(), Error> {
    let response = get_reqwest_client()
        .get(format!(
            "https://api.openai.com/v1/threads/{}/messages",
            thread_id
        ))
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .header("OpenAI-Beta", "assistants=v1")
        .send()
        .await?;

    let messages = response.json::<serde_json::Value>().await?;

    println!("messages are:");
    if let Ok(pretty_json) = serde_json::to_string_pretty(&messages) {
        println!("{}", pretty_json);
    } else {
        println!("Failed to serialize to pretty-printed JSON");
    }

    Ok(())
}

pub async fn create_speech(assistant_response: String, /*assistant_response_receiver: Receiver<String>,*/ audio_output_sender: Sender<Vec<i16>>, sample_rate: SampleRate, channels: u16) -> Result<(), Error> {
    let channels: opus::Channels = match channels {
        1 => opus::Channels::Mono,
        2 => opus::Channels::Stereo,
        _ => panic!()
    };
    let mut opus_decoder = Decoder::new(sample_rate.0, channels).unwrap();

    let data = serde_json::json!({
        "model": "tts-1",
        "input": assistant_response,
        "voice": "echo",
        "response_format": "opus"
    });

    //returns a response that contains a byte stream
    let response = get_reqwest_client()
        .post("https://api.openai.com/v1/audio/speech")
        .header(TRANSFER_ENCODING, "chunked")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .json(&data)
        .send()
        .await?;
        
    let bytes_stream = response.bytes_stream();
    let stream = bytes_stream.map(|res| {
        res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    });
    let stream_reader = StreamReader::new(stream);
    let mut packet_reader = PacketReader::new(stream_reader);

    while let Some(packet) = packet_reader.next().await {
        match packet {
            Ok(packet) => {
                let mut samples: Vec<i16> = vec![0; 1920];
                let _ = opus_decoder.decode(&packet.data, &mut samples, false);

                if samples.len() == 1920 {
                    for half in samples.chunks(960) { // we receive the audio info in a vec of size 1920, audio ouput stream needs vecs of size 960, so we send the data in two halves
                        match audio_output_sender.try_send(half.to_vec()) {
                            Ok(_) => {},
                            Err(e) => {
                                if e.is_disconnected() {
                                    panic!("Audio output channel disconnected!")
                                }
                            }
                        }
                    }
                }
            },
            Err(e) => println!("Error reading packet: {e:#?}")
        }
    }
    println!("Exiting create speech function");
    Ok(())
}

async fn execute(
    function_name: &str,
    arguments: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<String, Error> {
    println!(
        "wants to call: {}\nwith args: {:#?}",
        function_name, arguments
    );
    let result: String;

    match arguments {
        // functions with arguments
        Some(args) => {
            result = match function_name {
                "get_location_coordinates" => {
                    if let Some(location) = args.get("location").and_then(|v| v.as_str()) {
                        tools::get_location_coordinates(location).await
                    } else {
                        panic!("Failed to find location is arguments object.")
                    }
                }
                "get_forecast" => {
                    if let (Some(latitude), Some(longitude), Some(n_days)) = (
                        args.get("latitude"),
                        args.get("longitude"),
                        args.get("n_days"),
                    ) {
                        tools::get_forecast(
                            &latitude.to_string(),
                            &longitude.to_string(),
                            &n_days.to_string(),
                        )
                        .await
                    } else {
                        panic!("Failed to find latitude, longitude, or number of days in arguments object.")
                    }
                }
                _ => panic!("No function name given with arguments."),
            };
        }
        // functions without arguments
        None => {
            result = match function_name {
                "get_user_coordinates" => tools::get_user_coordinates().await,
                "get_clipboard_text" => tools::get_clipboard_text(),
                "get_screenshot" => tools::get_screenshot().await,
                "get_time" => tools::get_time(),
                "pass" => tools::pass(),
                _ => panic!("No function name given without arguments."),
            };
        }
    }

    Ok(result)
}
