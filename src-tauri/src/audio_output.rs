use crate::globals::{get_open_ai_key, get_reqwest_client};
use reqwest::header::TRANSFER_ENCODING;
use crossbeam::channel::{bounded, Receiver, Sender};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Device, Sample, StreamError, FromSample};
use opus::Decoder;
use ogg::reading::async_api::PacketReader;
use tokio_util::io::StreamReader;
use tokio_stream::StreamExt;
use std::{collections::VecDeque, error::Error, thread, time::Duration};

pub fn get_audio_output_device() -> Device {
    let host = cpal::default_host();

    let audio_output_device: Device = loop {
        match host.default_output_device() {
            Some(device) => break device,
            None => thread::sleep(Duration::from_secs(1))
        }
        println!("Looking for output device.")
    };
    println!("Found!\n{:?}", audio_output_device.name());

    audio_output_device
}

pub async fn speak(message: String, audio_output_sender: Sender<Vec<i16>>) {
    let data = serde_json::json!({
        "model": "tts-1",
        "input": message,
        "voice": "echo",
        "response_format": "opus"
    });

    let response = get_reqwest_client()
        .post("https://api.openai.com/v1/audio/speech")
        .header(TRANSFER_ENCODING, "chunked")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_open_ai_key()))
        .json(&data)
        .send()
        .await
        .unwrap();
    
    let bytes_stream = response.bytes_stream();
    let stream = bytes_stream.map(|res| {
        res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    });
    let stream_reader = StreamReader::new(stream);
    let mut packet_reader = PacketReader::new(stream_reader);

    let mut opus_decoder = Decoder::new(48000, opus::Channels::Stereo).unwrap();

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
}

pub fn run_stream(audio_output_receiver: Receiver<Vec<i16>>, device: Device) -> Box<dyn Error> {
    let config = device.default_output_config().unwrap();
    let (error_sender, error_receiver): (Sender<StreamError>, Receiver<StreamError>) = bounded(1);

    fn error_callback(e: StreamError, error_sender: Sender<StreamError>) {
        error_sender.send(e).ok();
    }

    fn write_audio<T>(output: &mut [T], audio_output_receiver: Receiver<Vec<i16>>)
    where 
        T: FromSample<i16> + Sample
    {
        if let Ok(samples) = audio_output_receiver.try_recv() {
            let mut samples: VecDeque<T> = samples.into_iter().map(T::from_sample).collect();

            for frame in output.chunks_mut(2) {
                for sample in frame.iter_mut() {
                    *sample = samples.pop_front().unwrap_or(T::EQUILIBRIUM);
                }
            }
        }
    }

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
        &config.clone().into(),
        move |data: &mut [f32], _| write_audio(data, audio_output_receiver.clone()),
        move |e| error_callback(e, error_sender.clone()),
        None
        ),
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [i16], _| write_audio(data, audio_output_receiver.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None
        ),
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [u16], _| write_audio(data, audio_output_receiver.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None
        ),
        _ => panic!()
    }.unwrap();

    match stream.play() {
        Ok(_) => println!("Successfully started audio output stream!"),
        Err(error) => println!("Failed to start audio output stream: {}", error),
    }

    loop {
        if let Ok(stream_error) = error_receiver.try_recv() {
            return Box::new(stream_error)
        }
    }
}

pub fn run(audio_output_receiver: Receiver<Vec<i16>>) {
    loop {
        let audio_output_device = get_audio_output_device();
        let error = run_stream(audio_output_receiver.clone(), audio_output_device);
        
        // many different potential errors can occur, maybe we handle them each differently??
        if let Some(stream_error) = error.downcast_ref::<StreamError>() {
            match stream_error {
                StreamError::DeviceNotAvailable => println!("Device not available error!"),
                StreamError::BackendSpecific { err } => println!("Backend specific error! {err:#?}")
            }
        }
        else if let Some(build_stream_error) = error.downcast_ref::<BuildStreamError>() {
            match build_stream_error {
                BuildStreamError::StreamConfigNotSupported => println!("Some build stream error!"),
                _ => println!("Some build stream error!")
            }
        }
    }
}
