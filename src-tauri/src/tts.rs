use tts::Tts;

pub async fn speak(message: String) {
    let mut tts = Tts::default().unwrap();
    let voices = tts.voices().unwrap();
    let _ = tts.set_voice(&voices[2]);

    tts.speak(message, true).unwrap();
    
    while tts.is_speaking().unwrap() {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
