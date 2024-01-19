use crossbeam::channel::Sender;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use std::thread;
use std::time::Duration;

/*
TODO?
maybe needs linear scaling??
maybe should collect a bit of audio data and send periodically?
*/

pub fn start_audio_stream(audio_sender: Sender<Vec<i16>>) {
    let host = cpal::default_host();

    let device = host.default_input_device()
        .expect("No output device available!");

    let mut supported_configs_range = device.supported_input_configs()
        .expect("Error while querying configs!");

    let config = supported_configs_range.next()
        .expect("No supported config?!")
        .with_max_sample_rate();

    let config_clone = config.clone();

    let error_callback = move |err| {
        println!("Error on stream: {}", err);
    };

    fn write_data<T: Sample>(data: &[f32], channels: u16, audio_sender: Sender<Vec<i16>>)
    where
        T: cpal::Sample,
    {
        let mut buffer = vec![];
        for frame in data.chunks(channels.into()) {
            buffer.push(frame[0]);
        }
        match audio_sender.send(convert_to_i16(&buffer)) {
            Ok(_) => {},
            Err(e) => println!("SendError: {e:#?}")
        }
    }
    
    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config_clone.into(),
                move |data, _: &_| write_data::<i16>(data, config.channels(), audio_sender.clone()),
                error_callback,
                None,
            )
            .expect("Failed to start Sample format I16"),
        cpal::SampleFormat::U16 => device
            .build_input_stream(
                &config_clone.into(),
                move |data, _: &_| write_data::<u16>(data, config.channels(), audio_sender.clone()),
                error_callback,
                None,
            )
            .expect("Failed to start Sample format U16"),
        cpal::SampleFormat::F32 => {
            device
            .build_input_stream(
                &config_clone.into(),
                move |data, _: &_| write_data::<f32>(data, config.channels(), audio_sender.clone()),
                error_callback,
                None,
            )
            .expect("Failed to start Sample format F32")
        }
        _ => todo!(),
    };

    match stream.play() {
        Ok(_) => println!("Successfully started audio stream!"),
        Err(error) => println!("Failed to start audio stream: {}", error),
    }

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn convert_to_i16(data: &Vec<f32>) -> Vec<i16> {
    // data.iter().map(|&f| (f as i16)).collect()
    // (data * i16::MAX as f32).clamp(-i16::MAX as f32, i16::MAX as f32) as i16
    let result: Vec<i16> = data
        .iter()
        .map(|&value| (value * i16::MAX as f32).clamp(-i16::MAX as f32, i16::MAX as f32) as i16)
        .collect();

    result
}
