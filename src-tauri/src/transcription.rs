use crossbeam::channel::{Receiver, Sender};
use vosk::{DecodingState, Model, Recognizer};
use cpal::SampleRate;

pub fn run(audio_receiver: Receiver<Vec<i16>>, transcription_sender: Sender<String>, sample_rate: SampleRate) {
    #[cfg(target_os = "macos")]
    //let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";
    let model_path = "./models/vosk-model-small-en-us-0.15/";

    #[cfg(target_os = "windows")]
    // let model_path = "C:/Users/schre/Projects/vosk-model-en-us-0.42-gigaspeech/";
    let model_path = "C:/Users/schre/Projects/vosk-model-small-en-us-0.15/";

    let model = Model::new(model_path).unwrap();
    let mut recognizer = Recognizer::new(&model, sample_rate.0 as f32).unwrap();
    println!("Vosk model loaded! It hears all...");

    loop {
        if let Ok(data) = audio_receiver.try_recv() {
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
