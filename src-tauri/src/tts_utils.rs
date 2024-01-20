use tts::Tts;
use std::thread;

pub fn speak(message: String) {
    thread::spawn(move || {
        let mut tts = Tts::default().unwrap();
        let voices = tts.voices().unwrap();

        #[cfg(target_os = "macos")]
        let _ = tts.set_voice(&voices[33]);
        #[cfg(target_os = "windows")]
        let _ = tts.set_voice(&voices[2]);

        tts.speak(message, true).unwrap();
        
        while tts.is_speaking().unwrap() {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
