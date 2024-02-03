use crate::globals::{get_open_ai_key, get_reqwest_client};
use reqwest::header::TRANSFER_ENCODING;
use crossbeam::channel::{Receiver, Sender};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamError;
use opus::Decoder;
use ogg::reading::async_api::PacketReader;
use tokio_util::io::StreamReader;
use tokio_stream::StreamExt;
use std::collections::VecDeque;

pub async fn speak(message: String, assistant_audio_sender: Sender<Vec<f32>>) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::json!({
        "model": "tts-1",
        "input": message,
        "voice": "echo",
        "response_format": "opus",
        "speed": 1.2
    });

    let response = get_reqwest_client()
        .post("https://api.openai.com/v1/audio/speech")
        .header(TRANSFER_ENCODING, "chunked")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .json(&data)
        .send()
        .await?;
    
    let stream = response.bytes_stream();
    let io_error_stream = stream.map(|res| {
        res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    });    
    let reader = StreamReader::new(io_error_stream);
    let mut packet_reader = PacketReader::new(reader);
    let mut opus_decoder = Decoder::new(48000, opus::Channels::Stereo).unwrap();

    while let Some(packet) = packet_reader.next().await {
        match packet {
            Ok(packet) => {
                let mut samples: Vec<f32> = vec![0.0; 1920];
                let _ = opus_decoder.decode_float(&packet.data, &mut samples, false);
                println!("Packet length: {:?} Sample count: {:?}", packet.data.len(), samples.len());
                if samples.len() == 1920 {
                    match assistant_audio_sender.try_send(samples[..960].to_vec()) {
                        Ok(_) => {},
                        Err(e) => {
                            if e.is_disconnected() {
                                panic!("Assistant audio channel disconnected!")
                            }
                        }
                    }
                    match assistant_audio_sender.try_send(samples[960..].to_vec()) {
                        Ok(_) => {},
                        Err(e) => {
                            if e.is_disconnected() {
                                panic!("Assistant audio channel disconnected!")
                            }
                        }
                    }
                }
            },
            Err(e) => println!("Error reading packet: {e:#?}")
        }
    }
    println!("No more packets!");

    Ok(())
}

pub fn run(assistant_audio_receiver: Receiver<Vec<f32>>) {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    println!("{:#?}", device.name());
    println!("{:#?}", config);

    fn error_callback(e: StreamError) {
        println!("Write audio stream error {e:#?}"); 
    }

    fn write_audio(output: &mut [f32], assistant_audio_receiver: Receiver<Vec<f32>>)
    {
        if let Ok(samples) = assistant_audio_receiver.try_recv() {
            let mut samples : VecDeque<f32> = VecDeque::from(samples);
            for frame in output.chunks_mut(2) {
                for sample in frame.iter_mut() {
                    // *sample = samples.next().unwrap_or(&0.0).clone();
                    *sample = samples.pop_front().unwrap_or(0.0);
                }
            }
        }
    }

    let stream = device.build_output_stream(
        &config.clone().into(),
        move |data: &mut [f32], _| write_audio(data, assistant_audio_receiver.clone()),
        move |e| error_callback(e),
        None
    ).expect("Error creating stream");

    match stream.play() {
        Ok(_) => println!("Successfully started output stream!"),
        Err(error) => println!("Failed to start output stream: {}", error),
    }

    loop {}
}
