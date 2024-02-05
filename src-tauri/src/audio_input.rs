use crossbeam::channel::{bounded, Receiver, Sender};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Device, FromSample, Sample, StreamError};
use std::thread;
use std::time::Duration;
use std::error::Error;

pub fn get_audio_input_device() -> Device {
    let host = cpal::default_host();

    let audio_input_device: Device = loop {
        match host.default_input_device() {
            Some(device) => break device,
            None => thread::sleep(Duration::from_secs(1))
        }
        println!("Looking for input device.")
    };
    println!("Found!\n {:?}", audio_input_device.name());

    audio_input_device
}

fn run_stream(audio_input_sender: Sender<Vec<i16>>, device: Device) -> Box<dyn Error> {
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
    }.expect("Failed to build stream!");

    match stream.play() {
        Ok(_) => println!("Successfully started audio input stream!"),
        Err(error) => println!("Failed to start audio input stream: {}", error),
    }

    loop {
        if let Ok(stream_error) = error_receiver.try_recv() {
            return Box::new(stream_error)
        }
    }
}

pub fn run(audio_input_sender: Sender<Vec<i16>>) {
    loop {
        let audio_input_device = get_audio_input_device();
        let error = run_stream(audio_input_sender.clone(), audio_input_device);
        
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
