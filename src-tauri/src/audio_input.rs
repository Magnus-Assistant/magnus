use crossbeam::channel::Sender;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, FromSample};

/*
TODO?
maybe needs linear scaling??
*/

pub fn get_default_input_device() -> Device {
    let host = cpal::default_host();

    if let Some(device) = host.default_input_device() {
        device
    }
    else {
         panic!("No default input device!") 
    }
}

pub fn run(audio_sender: Sender<Vec<i16>>, device: Device) {
    let config = device.default_input_config().unwrap();

    let error_callback = move |err| println!("Error on stream: {}", err);

    fn write_data<T: Sample>(data: &[T], channels: u16, audio_sender: Sender<Vec<i16>>)
    where
        i16: FromSample<T>
    {
        let mut buffer: Vec<i16> = vec![];
        for frame in data.chunks(channels.into()) {
            buffer.push(frame[0].to_sample::<i16>());
        }

        audio_sender.try_send(buffer).ok();
    }

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[f32], _: &_| write_data(data, config.channels(), audio_sender.clone()),
            error_callback,
            None
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[i16], _: &_| write_data(data, config.channels(), audio_sender.clone()),
            error_callback,
            None
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.clone().into(),
            move |data: &[u16], _: &_| write_data(data, config.channels(), audio_sender.clone()),
            error_callback,
            None
        ),
        _ => panic!()
    }.expect("Failed to build audio stream!");

    match stream.play() {
        Ok(_) => println!("Successfully started audio stream!"),
        Err(error) => println!("Failed to start audio stream: {}", error),
    }

    loop {}
}
