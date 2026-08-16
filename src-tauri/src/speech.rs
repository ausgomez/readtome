use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tts::Tts;

struct TtsWrapper(Tts);

// Tts uses COM on Windows which is thread-local, but we only access it
// from behind a Mutex so at most one thread touches it at a time.
unsafe impl Send for TtsWrapper {}
unsafe impl Sync for TtsWrapper {}

static TTS_ENGINE: Mutex<Option<TtsWrapper>> = Mutex::new(None);
static SPEAKING: AtomicBool = AtomicBool::new(false);

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Tts::default()?;
    let features = engine.supported_features();
    log::info!(
        "TTS initialized: utterance_callbacks={}, stop={}, rate={}, volume={}",
        features.utterance_callbacks,
        features.stop,
        features.rate,
        features.volume,
    );

    if let Ok(voices) = engine.voices() {
        for v in voices.iter().take(5) {
            log::info!("  Voice: {:?}", v);
        }
        if voices.len() > 5 {
            log::info!("  ... and {} more voices", voices.len() - 5);
        }
    }

    let mut lock = TTS_ENGINE.lock().map_err(|e| format!("TTS lock poisoned: {}", e))?;
    *lock = Some(TtsWrapper(engine));
    Ok(())
}

pub fn speak(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = TTS_ENGINE.lock().map_err(|e| format!("TTS lock poisoned: {}", e))?;
    let wrapper = lock.as_mut().ok_or("TTS engine not initialized")?;

    wrapper.0.speak(text, true)?;
    SPEAKING.store(true, Ordering::Relaxed);
    log::info!("TTS speaking: {} chars", text.len());
    Ok(())
}

pub fn stop() -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = TTS_ENGINE.lock().map_err(|e| format!("TTS lock poisoned: {}", e))?;
    if let Some(wrapper) = lock.as_mut() {
        wrapper.0.stop()?;
        SPEAKING.store(false, Ordering::Relaxed);
        log::info!("TTS stopped");
    }
    Ok(())
}

pub fn is_speaking() -> bool {
    SPEAKING.load(Ordering::Relaxed)
}

pub fn set_rate(rate: f32) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = TTS_ENGINE.lock().map_err(|e| format!("TTS lock poisoned: {}", e))?;
    let wrapper = lock.as_mut().ok_or("TTS engine not initialized")?;
    wrapper.0.set_rate(rate)?;
    log::info!("TTS rate set to {}", rate);
    Ok(())
}
