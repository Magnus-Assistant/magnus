use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, Stream, StreamConfig, BufferSize};


use std::collections::VecDeque;


pub fn start_stream() {
    // create the host and audio device
    let host = cpal::default_host();
    let audio_device = host
        .default_input_device()
        .expect("Failed to find audio device");

    let x = host.input_devices();

    match x {
        Ok(x) => { 
            for i in x {
                print!("{:?}", i.default_input_config())
            }
        }
        Err(x) => {panic!("sdfsd")}
    }


    //get a list of all supported configs
    let mut supported_configs_range = audio_device
        .supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!").with_max_sample_rate();
    println!("HOW MANY TIMES ");
    for sp in supported_configs_range {
        println!("{:?} ", sp);
    }

    let mut buffer: VecDeque<i16> = VecDeque::new();

    let mut stream_config: StreamConfig = StreamConfig::from(supported_config);
    // stream_config.buffer_size = Fixed(2048);
    // stream_config.sample_rate =  SampleRate(96000);
    // stream_config.channels = 1;

           println!(
                "buffer: {:?} channel: {} sample: {:?}",
                stream_config.buffer_size, stream_config.channels, stream_config.sample_rate
            );


    //create the actual output stream
    let stream: Result<Stream, cpal::BuildStreamError> = audio_device.build_input_stream(
        &stream_config,
        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            buffer.extend(data);
            println!("{:?}", data);
        },
        move |err| println!("{}", err),
        None,
    );

    match stream {
        Ok(s) => s.play().unwrap(),
        Err(s) => {
            println!("{}", s);
        }
    }

}
