use crate::{assistant, audio_output_device_selection, settings};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    Device, FromSample, Sample, StreamConfig, StreamError
};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::{
    collections::VecDeque,
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn get_default_audio_output_device() -> Device {
    let host = cpal::default_host();

    let audio_output_device: Device = loop {
        match host.default_output_device() {
            Some(device) => break device,
            None => thread::sleep(Duration::from_secs(1)),
        }
        println!("Looking for output device.")
    };
    println!(
        "Found output device! -> {:?}",
        audio_output_device.name().unwrap()
    );

    audio_output_device
}

pub fn get_current_audio_output_device() -> Device {
    let settings = settings::get_settings().as_object().unwrap().clone();
    let current_selection = settings.get("audioOutputDeviceSelection").unwrap().as_str().unwrap().to_string();
    let available_devices = get_audio_output_device_list();

    for device in available_devices {
        if device.name().unwrap() == current_selection {
            return device
        }
    }

    // if selected device is not available, go with the default
    let default_device = get_default_audio_output_device();

    // set selection to default device
    audio_output_device_selection(default_device.name().unwrap());
    
    default_device
}

pub fn get_audio_output_device_list() -> Vec<Device> {
    let host = cpal::default_host();

    let output_devices = host.output_devices().unwrap().collect::<Vec<_>>();

    output_devices
}

pub fn run_stream(
    audio_output_receiver: Receiver<Vec<i16>>,
    device: Device,
    synthesizing: Arc<Mutex<bool>>,
) -> Result<(), Box<dyn Error>> {
    let format = device.default_output_config().unwrap().sample_format();
    let mut config: StreamConfig = device.default_output_config().unwrap().into();
    let (error_sender, error_receiver): (Sender<StreamError>, Receiver<StreamError>) = bounded(1);

    fn error_callback(e: StreamError, error_sender: Sender<StreamError>) {
        error_sender.send(e).ok();
    }

    fn write_audio<T>(output: &mut [T], audio_output_receiver: Receiver<Vec<i16>>)
    where
        T: FromSample<i16> + Sample,
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

    let audio_output_receiver_clone = audio_output_receiver.clone();

    // fix for mac static issue
    #[cfg(target_os = "macos")]
    {
        config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(45000),
            buffer_size: cpal::BufferSize::Default,
        };
    }

    let stream = match format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [f32], _| write_audio(data, audio_output_receiver.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None,
        ),
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [i16], _| write_audio(data, audio_output_receiver.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None,
        ),
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config.clone().into(),
            move |data: &mut [u16], _| write_audio(data, audio_output_receiver.clone()),
            move |e| error_callback(e, error_sender.clone()),
            None,
        ),
        _ => panic!(),
    }
    .expect("Failed to build audio output stream!");

    match stream.play() {
        Ok(_) => println!("Successfully started audio output stream!"),
        Err(error) => println!("Failed to start audio output stream: {}", error),
    }

    loop {
        if let Ok(stream_error) = error_receiver.try_recv() {
            drop(stream);
            return Err(Box::new(stream_error));
        } else if !*synthesizing.lock().unwrap() && audio_output_receiver_clone.is_empty() {
            return Ok(());
        }
    }
}

pub async fn speak(assistant_message: String) -> Result<(), Box<dyn Error>> {
    // create speech sender and receiver
    let (audio_output_sender, audio_output_receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) =
        crossbeam::channel::unbounded::<Vec<i16>>();
    let synthesizing: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // find an output device
    let audio_output_device = get_current_audio_output_device();
    let audio_output_config = audio_output_device.default_output_config().unwrap();

    // spawn create_speech with sender
    *synthesizing.lock().unwrap() = true;
    let create_speech_handle = tauri::async_runtime::spawn(async move {
        assistant::create_speech(
            assistant_message,
            audio_output_sender,
            audio_output_config.sample_rate(),
            audio_output_config.channels(),
        )
        .await
    });

    // spawn output with receiver
    let synthesizing_clone = synthesizing.clone();
    let output_stream_handle = thread::spawn(move || {
        let _ = run_stream(
            audio_output_receiver,
            audio_output_device,
            synthesizing_clone,
        );
    });

    // wait for the threads to finish
    let _ = create_speech_handle.await;
    *synthesizing.lock().unwrap() = false;
    output_stream_handle.join().unwrap();

    return Ok(());
}
