use std::time::SystemTime;

use vosk::{Model, Recognizer};

use crate::audio_stream::InputClip;


pub fn start_model(data_stream: &Vec<i16>) {
    println!("Starting Vosk model with live audio...");
    //grab the stream data so we can dynamically read audio based on what the
    //system assigns for the config
    let stream_data = InputClip::build_config();
    let start = SystemTime::now();

    #[cfg(target_os = "macos")]
    let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";

    #[cfg(target_os = "windows")]
    let model_path = "C:/Users/schre/Projects/vosk-model-small-en-us-0.15";

    let model = Model::new(model_path).unwrap();
    let mut recognizer =
        Recognizer::new(&model, stream_data.config.sample_rate().0 as f32).unwrap();

    recognizer.set_words(true);
    recognizer.set_partial_words(false);

    let stop = SystemTime::now();
    match stop.duration_since(start) {
        Ok(t) => println!("Finished Loading model... Took => {:?}", t),
        Err(t) => println!("Error getting time: {}", t),
    };

    println!("Processing Audio Data...");
    let start = SystemTime::now();
    // prints out the partial results. Often times this prints a LOT
    for sample in data_stream.chunks(100) {
        recognizer.accept_waveform(sample);
    }

    println!("{:#?}", recognizer.final_result().single().unwrap());
    let stop = SystemTime::now();

    match stop.duration_since(start) {
        Ok(t) => println!("Finished Processing... Took => {:?}", t),
        Err(t) => println!("Error getting time: {}", t),
    };
}