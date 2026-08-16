use anyhow::{Context, Result};
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum CaptureResult {
    Success(String),
    NoSelection,
}

pub fn capture_selected_text() -> Result<CaptureResult> {
    let start = Instant::now();

    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;

    // 1. Save current clipboard contents
    let t_save = Instant::now();
    let original = clipboard.get_text().ok();
    let save_dur = t_save.elapsed();
    log::debug!(
        "Clipboard saved in {:?} (had text: {})",
        save_dur,
        original.is_some()
    );

    // 2. Clear clipboard so we can detect if copy worked
    let _ = clipboard.clear();

    // 3. Simulate Ctrl+C (Linux/Windows) or Cmd+C (macOS)
    let t_sim = Instant::now();
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to create enigo instance: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press)
            .map_err(|e| anyhow::anyhow!("Key press failed: {}", e))?;
        enigo.key(Key::Unicode('c'), Direction::Click)
            .map_err(|e| anyhow::anyhow!("Key click failed: {}", e))?;
        enigo.key(Key::Meta, Direction::Release)
            .map_err(|e| anyhow::anyhow!("Key release failed: {}", e))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press)
            .map_err(|e| anyhow::anyhow!("Key press failed: {}", e))?;
        enigo.key(Key::Unicode('c'), Direction::Click)
            .map_err(|e| anyhow::anyhow!("Key click failed: {}", e))?;
        enigo.key(Key::Control, Direction::Release)
            .map_err(|e| anyhow::anyhow!("Key release failed: {}", e))?;
    }

    let sim_dur = t_sim.elapsed();
    log::debug!("Copy keystroke simulated in {:?}", sim_dur);

    // 4. Wait for OS to process the copy
    thread::sleep(Duration::from_millis(100));

    // 5. Read clipboard
    let t_read = Instant::now();
    let captured = clipboard.get_text().unwrap_or_default();
    let read_dur = t_read.elapsed();
    log::debug!("Clipboard read in {:?}: {} chars", read_dur, captured.len());

    // 6. Restore original clipboard contents
    let t_restore = Instant::now();
    if let Some(ref orig) = original {
        let _ = clipboard.set_text(orig);
    }
    let restore_dur = t_restore.elapsed();
    log::debug!("Clipboard restored in {:?}", restore_dur);

    let total = start.elapsed();
    log::info!(
        "Capture timing: save={:?} sim={:?} wait=100ms read={:?} restore={:?} total={:?} chars={}",
        save_dur, sim_dur, read_dur, restore_dur, total, captured.len()
    );

    if captured.is_empty() {
        return Ok(CaptureResult::NoSelection);
    }

    Ok(CaptureResult::Success(captured))
}
