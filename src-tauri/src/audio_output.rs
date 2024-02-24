use crate::assistant;
use crossbeam::channel::{bounded, Receiver, Sender};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Device, Sample, StreamError, FromSample};
use std::{collections::VecDeque, error::Error, sync::{Arc, Mutex}, thread, time::Duration};

pub fn get_audio_output_device() -> Device {
    let host = cpal::default_host();

    let audio_output_device: Device = loop {
        match host.default_output_device() {
            Some(device) => break device,
            None => thread::sleep(Duration::from_secs(1))
        }
        println!("Looking for output device.")
    };
    println!("Found output device! -> {:?}", audio_output_device.name().unwrap());

    audio_output_device
}

pub fn run_stream(audio_output_receiver: Receiver<Vec<i16>>, device: Device, synthesizing: Arc<Mutex<bool>>) -> Result<(), Box<dyn Error>> {
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

    let audio_output_receiver_clone = audio_output_receiver.clone();

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
    }.expect("Failed to build audio output stream!");

    match stream.play() {
        Ok(_) => println!("Successfully started audio output stream!"),
        Err(error) => println!("Failed to start audio output stream: {}", error)
    }

    loop {
        if let Ok(stream_error) = error_receiver.try_recv() {
            drop(stream);
            return Err(Box::new(stream_error))
        }
        else if !*synthesizing.lock().unwrap() && audio_output_receiver_clone.is_empty() {
            return Ok(())
        }
    }
}
/* 
pub fn run(transcription_receiver: Receiver<String>) {
    let output_stream_running: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let (audio_output_sender, audio_output_receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = crossbeam::channel::unbounded::<Vec<i16>>();

    loop {
        *output_stream_running.lock().unwrap() = true;

        // find an output device
        let audio_output_device = get_audio_output_device();
        let audio_output_config = audio_output_device.default_output_config().unwrap();

        // must create clones for these types to be moved to the async runtime
        let output_stream_running_clone = output_stream_running.clone();
        let transcription_receiver_clone = transcription_receiver.clone();
        let audio_output_sender_clone = audio_output_sender.clone();

        // spawn assistant async runtime
        // thread::spawn(move || {
        //     tauri::async_runtime::spawn(async move {
        //         assistant::run(output_stream_running_clone, transcription_receiver_clone, audio_output_sender_clone, audio_output_config).await;
        //     });
        // });

        // run output stream until there is some error
        // let error = run_stream(audio_output_receiver.clone(), audio_output_device);

        // once there is an error with the output stream, stop the assistant runtime
        *output_stream_running.lock().unwrap() = false;

        // many different potential errors can occur, maybe we handle them each differently??
        if let Some(stream_error) = error.downcast_ref::<StreamError>() {
            match stream_error {
                StreamError::DeviceNotAvailable => println!("Output device disconnected!"),
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
*/
pub async fn speak(assistant_message: String) -> Result<(), Box<dyn Error>> {
    // create speech sender and receiver
    let (audio_output_sender, audio_output_receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = crossbeam::channel::unbounded::<Vec<i16>>();
    let synthesizing: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // find an output device
    let audio_output_device = get_audio_output_device();
    let audio_output_config = audio_output_device.default_output_config().unwrap();    

    // spawn create_speech with sender
    *synthesizing.lock().unwrap() = true;
    let create_speech_handle = tauri::async_runtime::spawn(async move {
        assistant::create_speech(assistant_message, audio_output_sender, audio_output_config.sample_rate(), audio_output_config.channels()).await
    });

    // spawn output with receiver
    let synthesizing_clone = synthesizing.clone();
    let output_stream_handle = thread::spawn(move || {
        let _ = run_stream(audio_output_receiver, audio_output_device, synthesizing_clone);
    });

    // wait for the threads to finish
    let asdf = create_speech_handle.await;
    *synthesizing.lock().unwrap() = false;
    output_stream_handle.join().unwrap();

    return Ok(())
}