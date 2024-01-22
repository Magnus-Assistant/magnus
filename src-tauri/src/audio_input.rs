use crossbeam::channel::Sender;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample,FromSample};

/*
TODO?
maybe needs linear scaling??
*/

pub fn run(audio_sender: Sender<Vec<i16>>) {
    let host = cpal::default_host();

    let device = host.default_input_device()
        .expect("No output device available!");

    let mut supported_configs_range = device.supported_input_configs()
        .expect("Error while querying configs!");

    let config = supported_configs_range.next()
        .expect("No supported config?!")
        .with_max_sample_rate();
    println!("{config:#?}");

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

    // let stream = device
    //     .build_input_stream(
    //         &config.clone().into(),
    //         move |data: ?, _: &_| write_data(data, config.channels(), audio_sender.clone()),
    //         error_callback,
    //         None,
    //     )
    //     .expect("Failed to build audio stream!");

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
