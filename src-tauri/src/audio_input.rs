use crossbeam::channel::{bounded, Receiver, Sender};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Device, FromSample, Sample, SampleRate, StreamError};
use std::thread;
use std::time::Duration;
use std::error::Error;
use std::sync::{Arc, Mutex};
use vosk::{DecodingState, Model, Recognizer};
use std::time::Instant;
use crate::globals::get_vosk_model;

pub fn get_audio_input_device() -> Device {
    let host = cpal::default_host();

    let audio_input_device: Device = loop {
        match host.default_input_device() {
            Some(device) => break device,
            None => thread::sleep(Duration::from_secs(1))
        }
        println!("Looking for input device.")
    };
    println!("Found input device! -> {:?}", audio_input_device.name().unwrap());

    audio_input_device
}

pub fn run_transcription(audio_input_receiver: Receiver<Vec<i16>>, sample_rate: SampleRate) -> Option<String> {
    let mut recognizer = Recognizer::new(&get_vosk_model(), sample_rate.0 as f32).unwrap();
    println!("Vosk model loaded! It hears all...");

    // start "timer" here
    let transcription_start_time = Instant::now();

    loop {
        if let Ok(data) = audio_input_receiver.try_recv() {
            let decoding_state = recognizer.accept_waveform(data.as_slice());
            if decoding_state == DecodingState::Finalized {
                // silence detected
                let transcription = recognizer.final_result().single().unwrap().text.to_string();

                if transcription.is_empty() {
                    return None
                }
                else if transcription != "huh".to_string() {
                    return Some(transcription);
                }
            }
            else if decoding_state == DecodingState::Running {
                // if partial result is nothing, and its been 3 seconds or more since the timer started, return None  
                // without this, transcription will run until something has been said
                let partial = recognizer.partial_result().partial;

                if partial == "" && transcription_start_time.elapsed() >= Duration::from_secs(3) {
                    println!("Nothing said after 3 seconds");
                    return None
                }
            }
        }
    }
}

fn run_stream(audio_input_sender: Sender<Vec<i16>>, device: Device, transcribing: Arc<Mutex<bool>>) {
    let config = device.default_input_config().unwrap();
    let (error_sender, error_receiver): (Sender<StreamError>, Receiver<StreamError>) = bounded(1);

    fn error_callback(e: StreamError, error_sender: Sender<StreamError>) {
        error_sender.send(e).ok();
    }

    fn write_data<T>(data: &[T], channels: u16, audio_input_sender: Sender<Vec<i16>>)
    where
        T: Sample,
        i16: FromSample<T>
    {
        let mut buffer: Vec<i16> = vec![];
        for frame in data.chunks(channels.into()) {
            buffer.push(frame[0].to_sample::<i16>());
        }

        match audio_input_sender.try_send(buffer) {
            Ok(_) => {},
            Err(e) => {
                if e.is_disconnected() {
                    panic!("Audio input channel disconnected!")
                }
            }
        }
    }

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[f32], _: &_| write_data(data, config.channels(), audio_input_sender.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[i16], _: &_| write_data(data, config.channels(), audio_input_sender.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[u16], _: &_| write_data(data, config.channels(), audio_input_sender.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None
        ),
        _ => panic!()
    }.expect("Failed to build audio input stream!");

    match stream.play() {
        Ok(_) => {
            println!("Successfully started audio input stream!")
        },
        Err(error) => println!("Failed to start audio input stream: {}", error),
    }

    loop {
        if let Ok(stream_error) = error_receiver.try_recv() {
            println!("ERROR OCCURRED ON INPUT STREAM");
            break
        }
        else if !*transcribing.lock().unwrap() {
            println!("TRANSCRIPTION FINISHED, EXITING INPUT STREAM");
            break
        }
    }
}

pub fn run() -> Option<String> {
    let (audio_input_sender, audio_input_receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = bounded::<Vec<i16>>(1);
    let transcribing: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // find an input device
    let audio_input_device = get_audio_input_device();
    let audio_input_config = audio_input_device.default_input_config().unwrap();

    // spawn the transcription thread with details on the device we found
    // let input_stream_running_clone = input_stream_running.clone();
    let audio_input_receiver_clone = audio_input_receiver.clone();
    *transcribing.lock().unwrap() = true; 
    let transcription_handle = thread::spawn(move || {
        run_transcription(audio_input_receiver_clone, audio_input_config.sample_rate())
    });

    // run input stream until there is some error
    let transcribing_clone = transcribing.clone();
    let audio_input_sender_clone = audio_input_sender.clone();
    let input_stream_handle = thread::spawn(move || {
        run_stream(audio_input_sender_clone, audio_input_device, transcribing_clone);
    });

    println!("Waiting for transcription to end");
    let transcription = transcription_handle.join().unwrap();
    *transcribing.lock().unwrap() = false;
    println!("Waiting for input stream to end");
    let input_end = input_stream_handle.join().unwrap();
    println!("All done");

    transcription
}
