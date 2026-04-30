use std::sync::{Arc, Mutex};

/// Shared application state managed by Tauri.
///
/// Holds the single SQLite connection wrapped for thread-safe access.
/// Initialised in the setup hook before any commands are callable.
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
}
