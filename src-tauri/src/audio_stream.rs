use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, Stream, SupportedStreamConfig};

use dasp_interpolate::linear::Linear;
use dasp_signal::{self as signal, Signal};
use std::sync::{Arc, Mutex};

///holds the stream object and audio clip
pub struct RecordingHandle {
    stream: Stream,
    pub clip: Arc<Mutex<Option<InputClip>>>,
}

impl RecordingHandle {
    pub fn stop(self) -> InputClip {
        drop(self.stream);
        self.clip.lock().unwrap().take().unwrap()
    }
}

///struct containing audio clip information
#[derive(Clone)]
pub struct InputClip {
    pub samples: Vec<f32>,
    sample_rate: u32,
}

///holds stream config and audio device information
pub struct StreamData {
    audio_device: Device,
    pub config: SupportedStreamConfig,
}

impl InputClip {
    ///resample the input data for reading via Vosk
    pub fn resample(sample_rate: u32, clip: &InputClip) -> Vec<f32> {
        let mut signal = signal::from_iter(clip.samples.iter().copied());
        let a = signal.next();
        let b = signal.next();

        //transform the data using interpolation
        let linear = Linear::new(a, b);

        //return the new Vector of Interpolated values
        signal
            .from_hz_to_hz(linear, clip.sample_rate as f64, sample_rate as f64)
            .take(clip.samples.len() * (sample_rate as usize) / (clip.sample_rate as usize))
            .collect()
    }

    //Builds the needed configuration for starting an input data stream
    pub fn build_config() -> StreamData {
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

        //return a StreamData struct with all the needed info
        StreamData {
            audio_device,
            config: supported_config,
        }
    }

    fn create_clip() -> Arc<Mutex<Option<InputClip>>> {
        //grab stream config, number of channels and create the clip
        let stream_data = Self::build_config();

        let clip = InputClip {
            samples: Vec::new(),
            sample_rate: stream_data.config.sample_rate().0,
        };

        //some helpful logging about the config thats being used
        println!(
            "Using Sample Rate of: {}",
            stream_data.config.sample_rate().0
        );
        println!(
            "Using Sample Format of: {}",
            stream_data.config.sample_format()
        );
        println!(
            "Using this many Channels: {}",
            stream_data.config.channels()
        );
        println!(
            "Using buffer size of: {:?}",
            stream_data.config.buffer_size()
        );

        //return a arc mutex of the created clip
        Arc::new(Mutex::new(Some(clip)))
    }

    ///Creates and writes input audio information to a Vector and stores them in an RecordingHandle
    pub fn create_stream() -> RecordingHandle {
        let stream_data = Self::build_config();
        let channels = stream_data.config.channels();

        let clip = Self::create_clip();
        let clip_2 = clip.clone();

        //create a type for our writer, and define how we write data to the input Array
        //This array can get HUGE. 44100 items a second big. Would be better to use a buffer
        type AudioClipHandle = Arc<Mutex<Option<InputClip>>>;
        fn write_data<T: Sample>(input: &[f32], channels: u16, writer: &AudioClipHandle)
        where
            T: cpal::Sample,
        {
            if let Ok(mut m_guard) = writer.try_lock() {
                if let Some(clip) = m_guard.as_mut() {
                    for frame in input.chunks(channels.into()) {
                        clip.samples.push(frame[0]);
                    }
                }
            }
        }

        // Generic error callback function for when we are creating input streams
        let err_fn = move |err| {
            println!("Error on stream: {}", err);
        };

        //create the actual output stream based on what sample format we are using
        let stream = match stream_data.config.sample_format() {
            cpal::SampleFormat::I16 => stream_data
                .audio_device
                .build_input_stream(
                    &stream_data.config.clone().into(),
                    move |data, _: &_| write_data::<i16>(data, channels, &clip_2),
                    err_fn,
                    None,
                )
                .expect("Failed to start Sample format I16"),
            cpal::SampleFormat::U16 => stream_data
                .audio_device
                .build_input_stream(
                    &stream_data.config.clone().into(),
                    move |data, _: &_| write_data::<u16>(data, channels, &clip_2),
                    err_fn,
                    None,
                )
                .expect("Failed to start Sample format U16"),
            cpal::SampleFormat::F32 => stream_data
                .audio_device
                .build_input_stream(
                    &stream_data.config.clone().into(),
                    move |data, _: &_| write_data::<f32>(data, channels, &clip_2),
                    err_fn,
                    None,
                )
                .expect("Failed to start Sample format F32"),

            _ => todo!(),
        };

        let _ = stream.play();

        //return a recording handle with the stream object and new audio clip
        RecordingHandle { stream, clip }
    }

    pub fn resample_clip(mut clip: InputClip) -> InputClip {
        let stream_data = Self::build_config(); //build config again since we took ownership and dropped it above
        let new_samples = Self::resample(stream_data.config.sample_rate().0, &clip);
        println!("Recorded {} samples", clip.clone().samples.len());

        clip.samples = new_samples;
        clip
    }
}

/// Converts the F32 (floating point) values to the needed 16bit PCM type for processing
pub fn convert_to_16pcm(clip_data: &Vec<f32>) -> Vec<i16> {
    let pcm_samples: Vec<i16> = clip_data
        .iter()
        .map(|&sample| {
            // Scale and convert to 16-bit PCM
            (sample * i16::MAX as f32).clamp(-i16::MAX as f32, i16::MAX as f32) as i16
        })
        .collect();

    pcm_samples
}
