use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, InputCallbackInfo, Sample, SampleRate, Stream, StreamConfig};

//use dasp_interpolate::linear::Linear;
use dasp_signal::{self as signal, Signal};
use signal::interpolate;
use std::error;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;
use color_eyre::eyre::{eyre, Result};

#[derive(Clone)]
pub struct AudioClip {
    samples: Vec<f32>,
    sample_rate: u32,
}

impl AudioClip {
    pub fn resample(&self, sample_rate: u32) -> AudioClip {
        if self.sample_rate == sample_rate {
            // return a copy since its the same?
            return self.clone();
        }

        let mut sig = signal::from_iter(self.samples.iter().copied());
        let a = sig.next();
        let b = sig.next();

        //let linear = Linear::new(a, b);

        // AudioClip {
        //     samples: sig
        //     .from_hz_to_hz(linear, self.sample_rate as f64, sample_rate as f64)
        //     .take(self.samples.len() * (sample_rate as usize) / (self.sample_rate as usize))
        //     .collect(),
        //     sample_rate: sample_rate,
        // }
        AudioClip {
            samples: Vec::new(),
            sample_rate: sample_rate,
        }
    }

    pub fn record(name: String) -> Result<Vec<f32>> {
        // create the host and audio device
        let host = cpal::default_host();
        let audio_device = host
            .default_input_device()
            .expect("Failed to find audio device");

        //get a list of all supported configs
        let mut supported_configs_range = audio_device
            .supported_input_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();

        let clip = AudioClip { samples: Vec::new(), sample_rate: supported_config.sample_rate().0};

        let clip = Arc::new(Mutex::new(Some(clip)));
        let clip_2 = clip.clone();

        println!("We aught to be recordin...");

        let err_fn = move |err| {
            println!("Error on stream: {}", err);
        };

        let channels = supported_config.channels();

        type ClipHandle = Arc<Mutex<Option<AudioClip>>>;

        fn write_data<T: Sample>(input: &[f32], channels: u16, writer: &ClipHandle)
        where T: cpal::Sample,
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some(clip) = guard.as_mut() {
                    for frame in input.chunks(channels.into()) {
                        clip.samples.push(frame[0]);
                    }
                }
            }
        }

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::I16 => audio_device.build_input_stream(
        &supported_config.into(),
                move |data, _: &_| write_data::<f32>(data, channels, &clip_2),
                err_fn,
                None,
            ).expect("Failed"),
            cpal::SampleFormat::U16 => audio_device.build_input_stream(
                &supported_config.into(),
                move |data, _: &_| write_data::<i16>(data, channels, &clip_2),
                err_fn,
                None,
            ).expect("Failed"),
            cpal::SampleFormat::F32 => audio_device.build_input_stream(
                &supported_config.into(),
                move |data, _: &_| write_data::<u16>(data, channels, &clip_2),
                err_fn,
                None,
            ).expect("Failed"),

            (_) => todo!(),
        };

        stream.play()?;
        
        let (tx, rx) = channel();
        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))?;

        println!("Waiting for Ctrl-C...");
        rx.recv()?;
        println!("Got it! Exiting...");

        drop(stream);
        let clip = clip.lock().unwrap().take().unwrap();

        println!("Recorded {} samples", clip.samples.len());
        Ok(clip.samples)
    }

    

}

// pub fn start_stream() {
//     let stream_handle = thread::spawn(move || {
//         // create the host and audio device
//         let host = cpal::default_host();
//         let audio_device = host
//             .default_input_device()
//             .expect("Failed to find audio device");

//         //get a list of all supported configs
//         let mut supported_configs_range = audio_device
//             .supported_input_configs()
//             .expect("error while querying configs");
//         let supported_config = supported_configs_range
//             .next()
//             .expect("no supported config?!")
//             .with_max_sample_rate();

//         let mut buffer: Vec<Data> = Vec::new();

//         let mut default_config: StreamConfig =
//             StreamConfig::from(audio_device.default_input_config().unwrap());
//         //default_config.sample_rate = SampleRate(48000);
//         let stream_config: StreamConfig = StreamConfig::from(supported_config);

//         println!(
//             "SYSTEM CHOSEN: buffer: {:?} channel: {} sample: {:?}",
//             stream_config.buffer_size, stream_config.channels, stream_config.sample_rate
//         );
//         println!(
//             "DEFAULT: buffer: {:?} channel: {} sample: {:?}",
//             default_config.buffer_size, default_config.channels, default_config.sample_rate
//         );

//         //create the actual output stream
//         let stream: Result<Stream, cpal::BuildStreamError> = audio_device.build_input_stream_raw(
//             &default_config,
//             cpal::SampleFormat::F32,
//             move |data, callback| {
//                 //write_data::<f32>(data, callback)
//                 process_data(data, callback);
//             },
//             move |err| println!("{}", err),
//             None,
//         );

//         fn write_data<T: Sample>(data: &Data, _: &InputCallbackInfo) {
//             //let mut data: Vec<f32> = vec![0.0; data.len()];
//             println!("Input callback triggered!");
//             println!("Data length: {}", data.len());
//         }

//         fn process_data(data: &Data, _: &InputCallbackInfo) {
//             panic!("JUST DIE");
//             println!("Recieved input data: {:?}", data);
//         }

//         match stream {
//             Ok(s) => {
//                 let success = s.play().expect("STREAM FAILED TO START");
//                 //     match success {
//                 //         Ok(_su) => {
//                 //             println!("Play was successful")
//                 //         }
//                 //         Err(su) => {
//                 //             println!("{}", su)
//                 //         }
//                 //     }
//                 // }
//                 // Err(s) => {
//                 //     println!("{}", s);
//             }
//             Err(s) => println!("Failed to initialize stream: {}", s),
//         }

//         thread::sleep(Duration::from_secs(20));
//         println!("Exiting thread");
//     });
//     //stream_handle.join().unwrap();
//     println!("Exited function... strean dead");
// }
