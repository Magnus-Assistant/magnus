use lazy_static::lazy_static;
use std::thread;
use tts::Error;
use tts::Tts;

lazy_static! {
    pub static ref TTS_INSTANCE: Result<Tts, Error> = Tts::default();
}

pub fn speak(message: String) {
    match TTS_INSTANCE.as_ref() {
        Ok(tts) => {
            //since we can't make a borrowed value mutable we have to clone it
            let mut cloned_tts = tts.clone();
            thread::spawn(move || {
                let voices = cloned_tts.voices().unwrap();

                #[cfg(target_os = "macos")]
                let _ = cloned_tts.set_voice(&voices[33]);

                #[cfg(target_os = "windows")]
                let _ = cloned_tts.set_voice(&voices[2]);

                cloned_tts.speak(message, true).unwrap();

                while tts.is_speaking().unwrap() {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });
        },
        Err(e) => panic!("Failed to get static tts ref! {e}")
    }
}
