use crossbeam::channel::{Receiver, Sender};
use vosk::{DecodingState, Model, Recognizer};
use cpal::SampleRate;
use std::sync::{Arc, Mutex};

pub fn run(input_stream_running: Arc<Mutex<bool>>, audio_input_receiver: Receiver<Vec<i16>>, transcription_sender: Sender<String>, sample_rate: SampleRate) {
    // let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";
    let model_path = "./models/vosk-model-small-en-us-0.15/";

    let model = Model::new(model_path).unwrap();
    let mut recognizer = Recognizer::new(&model, sample_rate.0 as f32).unwrap();
    println!("Vosk model loaded! It hears all...");

    while *input_stream_running.lock().unwrap() {
        if let Ok(data) = audio_input_receiver.try_recv() {
            let decoding_state = recognizer.accept_waveform(data.as_slice());
            if decoding_state == DecodingState::Finalized {
                // silence detected
                let transcription = recognizer.final_result().single().unwrap().text.to_string();

                if !transcription.is_empty() && transcription != "huh".to_string() {
                    transcription_sender.try_send(transcription).ok();
                }
            }
        }
    }
}
