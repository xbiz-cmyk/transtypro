use std::sync::{Arc, Mutex};

use crate::db::repositories::SettingsRepository;
use crate::errors::AppError;
use crate::models::InsertionResult;

pub struct InsertionService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl InsertionService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    /// Insert `text` into the previously focused application via clipboard paste simulation.
    ///
    /// Privacy rules enforced here:
    /// - `text` is never logged.
    /// - Saved clipboard contents are never logged.
    /// - No network calls.
    /// - No shell commands.
    /// - No screen or active-app content is read.
    pub fn insert_text(&self, text: String) -> Result<InsertionResult, AppError> {
        // 1. Read clipboard_restore_enabled from settings (release lock immediately).
        let restore_enabled = {
            let conn = self
                .db
                .lock()
                .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
            SettingsRepository::new(&conn)
                .get()?
                .clipboard_restore_enabled
        };

        // 2. Save current clipboard text (non-fatal; errors treated as None).
        //    Contents are never logged.
        let saved_clipboard: Option<String> = arboard::Clipboard::new()
            .ok()
            .and_then(|mut cb| cb.get_text().ok());

        // 3. Write final text to clipboard.
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                if let Err(e) = cb.set_text(&text) {
                    return Ok(InsertionResult {
                        success: false,
                        method: "clipboard_only".to_string(),
                        message: format!("Failed to write to clipboard: {e}"),
                    });
                }
            }
            Err(e) => {
                return Ok(InsertionResult {
                    success: false,
                    method: "clipboard_only".to_string(),
                    message: format!("Could not open clipboard: {e}"),
                });
            }
        }

        // 4. Simulate Ctrl+V / Cmd+V.
        let paste_succeeded = simulate_paste();

        // 5. Brief pause so the target app can read the clipboard before a potential restore.
        std::thread::sleep(std::time::Duration::from_millis(150));

        // 6. Restore old clipboard if enabled and a value was saved.
        if restore_enabled {
            if let Some(saved) = saved_clipboard {
                match arboard::Clipboard::new() {
                    Ok(mut cb) => {
                        if let Err(e) = cb.set_text(&saved) {
                            eprintln!("[insertion] clipboard restore failed: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("[insertion] could not open clipboard for restore: {e}");
                    }
                }
            }
        }

        // 7. Return result.
        if paste_succeeded {
            Ok(InsertionResult {
                success: true,
                method: "clipboard_paste".to_string(),
                message: "Text inserted successfully.".to_string(),
            })
        } else {
            Ok(InsertionResult {
                success: false,
                method: "clipboard_only".to_string(),
                message: "Paste simulation failed. Text is in your clipboard — press Ctrl+V or Cmd+V to paste manually.".to_string(),
            })
        }
    }
}

/// Simulate Ctrl+V (Windows/Linux) or Cmd+V (macOS) using enigo.
///
/// Always releases the modifier key even if pressing V fails.
/// Returns true if the full sequence succeeded.
fn simulate_paste() -> bool {
    use enigo::{Direction, Keyboard, Settings};

    #[cfg(target_os = "macos")]
    let modifier = enigo::Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = enigo::Key::Control;

    let mut enigo = match enigo::Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[insertion] enigo init failed: {e}");
            return false;
        }
    };

    if enigo.key(modifier, Direction::Press).is_err() {
        return false;
    }

    let v_ok = enigo
        .key(enigo::Key::Unicode('v'), Direction::Click)
        .is_ok();

    // Always release modifier — non-fatal if release fails.
    if let Err(e) = enigo.key(modifier, Direction::Release) {
        eprintln!("[insertion] modifier release failed: {e}");
    }

    v_ok
}

#[cfg(test)]
mod tests {
    use crate::models::InsertionResult;

    #[test]
    fn test_insertion_result_serde_success() {
        let r = InsertionResult {
            success: true,
            method: "clipboard_paste".to_string(),
            message: "Text inserted successfully.".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: InsertionResult = serde_json::from_str(&json).unwrap();
        assert!(back.success);
        assert_eq!(back.method, "clipboard_paste");
    }

    #[test]
    fn test_insertion_result_serde_failure() {
        let r = InsertionResult {
            success: false,
            method: "clipboard_only".to_string(),
            message: "Paste failed.".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: InsertionResult = serde_json::from_str(&json).unwrap();
        assert!(!back.success);
        assert_eq!(back.method, "clipboard_only");
    }
}
