use crossbeam::channel::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use vosk::{DecodingState, Model, Recognizer};

pub fn run(audio_receiver: Receiver<Vec<i16>>, transcription_sender: Sender<String>) {
    #[cfg(target_os = "macos")]
    //let model_path = "./models/vosk-model-en-us-0.42-gigaspeech/";
    let model_path = "./models/vosk-model-small-en-us-0.15/";

    #[cfg(target_os = "windows")]
    // let model_path = "C:/Users/schre/Projects/vosk-model-en-us-0.42-gigaspeech/";
    let model_path = "C:/Users/schre/Projects/vosk-model-small-en-us-0.15/";

    let model = Model::new(model_path).unwrap();
    let mut recognizer = Recognizer::new(&model, 41000.0).unwrap();
    println!("Vosk model loaded! It hears all...");

    loop {
        match audio_receiver.try_recv() {
            Ok(data) => {
                let decoding_state = recognizer.accept_waveform(data.as_slice());
                if decoding_state == DecodingState::Finalized {
                    // silence detected
                    let transcription = recognizer.final_result().single().unwrap().text.to_string();
                    transcription_sender.send(transcription).unwrap();
                }
                else if decoding_state == DecodingState::Failed {
                    println!("FAILED");
                }
            },
            Err(e) => {
                thread::sleep(Duration::from_secs(1));
            }
        }
        thread::sleep(Duration::from_millis(30));
    }
}
