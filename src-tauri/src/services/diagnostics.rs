use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::db::repositories::SettingsRepository;
use crate::errors::AppError;
use crate::models::{DiagnosticCheck, DiagnosticReport};

pub struct DiagnosticsService {
    db: Arc<Mutex<rusqlite::Connection>>,
    audio_dir: PathBuf,
}

impl DiagnosticsService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>, audio_dir: PathBuf) -> Self {
        Self { db, audio_dir }
    }

    /// Run all diagnostic checks and return a full report.
    ///
    /// Each check is independent — a failure in one check does not abort the rest.
    /// Status values: "pass", "fail", "warn", "skip".
    /// Diagnostics data is returned via Tauri IPC only; it is never sent externally.
    pub fn run_diagnostics(&self) -> Result<DiagnosticReport, AppError> {
        // 1–4: checks that are always safe to run unconditionally.
        let mut checks: Vec<DiagnosticCheck> = vec![
            // 1. backend_alive — always passes.
            DiagnosticCheck {
                name: "backend_alive".to_string(),
                status: "pass".to_string(),
                message: "Rust backend is running".to_string(),
            },
            // 2. database_reachable
            self.check_database_reachable(),
            // 3. migrations_current
            self.check_migrations_current(),
            // 4. microphone_available
            self.check_microphone_available(),
        ];

        // 5–8. whisper binary and model (read settings once)
        let settings_result = {
            match self.db.lock() {
                Ok(conn) => SettingsRepository::new(&conn).get(),
                Err(_) => Err(AppError::StorageError("database lock is poisoned".into())),
            }
        };

        for check in self.check_whisper(&settings_result) {
            checks.push(check);
        }

        // 9. providers_configured
        checks.push(self.check_providers_configured());

        // 10. ollama_reachable
        checks.push(self.check_ollama_reachable());

        // 11. shortcut_configured
        checks.push(self.check_shortcut_configured());

        // 12. audio_dir_accessible
        checks.push(self.check_audio_dir_accessible());

        // 13. history_count — informational, always passes.
        checks.push(self.check_history_count());

        // 14. audio_dir_size — informational, always passes.
        checks.push(self.check_audio_dir_size());

        Ok(DiagnosticReport {
            checks,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn check_database_reachable(&self) -> DiagnosticCheck {
        match self.db.lock() {
            Ok(conn) => {
                match conn.query_row("SELECT 1 FROM settings WHERE id = 1", [], |_| Ok(())) {
                    Ok(_) => DiagnosticCheck {
                        name: "database_reachable".to_string(),
                        status: "pass".to_string(),
                        message: "SQLite database is reachable".to_string(),
                    },
                    Err(e) => DiagnosticCheck {
                        name: "database_reachable".to_string(),
                        status: "fail".to_string(),
                        message: format!("Database query failed: {e}"),
                    },
                }
            }
            Err(_) => DiagnosticCheck {
                name: "database_reachable".to_string(),
                status: "fail".to_string(),
                message: "Database lock is poisoned".to_string(),
            },
        }
    }

    fn check_migrations_current(&self) -> DiagnosticCheck {
        const EXPECTED_VERSION: i64 = 6;
        match self.db.lock() {
            Ok(conn) => {
                let result: Result<Option<i64>, _> =
                    conn.query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                        r.get(0)
                    });
                match result {
                    Ok(Some(v)) if v >= EXPECTED_VERSION => DiagnosticCheck {
                        name: "migrations_current".to_string(),
                        status: "pass".to_string(),
                        message: format!("Schema is at version {v} (current)"),
                    },
                    Ok(Some(v)) => DiagnosticCheck {
                        name: "migrations_current".to_string(),
                        status: "warn".to_string(),
                        message: format!("Schema at version {v}, expected {EXPECTED_VERSION}"),
                    },
                    Ok(None) => DiagnosticCheck {
                        name: "migrations_current".to_string(),
                        status: "warn".to_string(),
                        message: "No migrations have been applied".to_string(),
                    },
                    Err(e) => DiagnosticCheck {
                        name: "migrations_current".to_string(),
                        status: "warn".to_string(),
                        message: format!("Could not check migrations: {e}"),
                    },
                }
            }
            Err(_) => DiagnosticCheck {
                name: "migrations_current".to_string(),
                status: "warn".to_string(),
                message: "Database lock unavailable".to_string(),
            },
        }
    }

    fn check_microphone_available(&self) -> DiagnosticCheck {
        use cpal::traits::HostTrait;
        let host = cpal::default_host();
        match host.input_devices() {
            Ok(mut devices) => {
                if devices.next().is_some() {
                    DiagnosticCheck {
                        name: "microphone_available".to_string(),
                        status: "pass".to_string(),
                        message: "At least one input device found".to_string(),
                    }
                } else {
                    DiagnosticCheck {
                        name: "microphone_available".to_string(),
                        status: "warn".to_string(),
                        message: "No input devices found".to_string(),
                    }
                }
            }
            Err(e) => DiagnosticCheck {
                name: "microphone_available".to_string(),
                status: "warn".to_string(),
                message: format!("Could not enumerate input devices: {e}"),
            },
        }
    }

    fn check_whisper(
        &self,
        settings_result: &Result<crate::models::AppSettings, AppError>,
    ) -> Vec<DiagnosticCheck> {
        let mut checks = Vec::new();
        match settings_result {
            Ok(settings) => {
                // Binary configured?
                if let Some(ref path) = settings.whisper_binary_path {
                    checks.push(DiagnosticCheck {
                        name: "whisper_binary_configured".to_string(),
                        status: "pass".to_string(),
                        message: "Whisper binary path is configured".to_string(),
                    });
                    // Binary exists?
                    if std::path::Path::new(path).is_file() {
                        checks.push(DiagnosticCheck {
                            name: "whisper_binary_exists".to_string(),
                            status: "pass".to_string(),
                            message: "Whisper binary file exists".to_string(),
                        });
                    } else {
                        checks.push(DiagnosticCheck {
                            name: "whisper_binary_exists".to_string(),
                            status: "fail".to_string(),
                            message: format!("Whisper binary not found at: {path}"),
                        });
                    }
                } else {
                    checks.push(DiagnosticCheck {
                        name: "whisper_binary_configured".to_string(),
                        status: "warn".to_string(),
                        message: "Whisper binary path is not configured".to_string(),
                    });
                    checks.push(DiagnosticCheck {
                        name: "whisper_binary_exists".to_string(),
                        status: "skip".to_string(),
                        message: "Skipped — no binary path configured".to_string(),
                    });
                }
                // Model configured?
                if let Some(ref path) = settings.whisper_model_path {
                    checks.push(DiagnosticCheck {
                        name: "whisper_model_configured".to_string(),
                        status: "pass".to_string(),
                        message: "Whisper model path is configured".to_string(),
                    });
                    // Model exists?
                    if std::path::Path::new(path).is_file() {
                        checks.push(DiagnosticCheck {
                            name: "whisper_model_exists".to_string(),
                            status: "pass".to_string(),
                            message: "Whisper model file exists".to_string(),
                        });
                    } else {
                        checks.push(DiagnosticCheck {
                            name: "whisper_model_exists".to_string(),
                            status: "fail".to_string(),
                            message: format!("Whisper model not found at: {path}"),
                        });
                    }
                } else {
                    checks.push(DiagnosticCheck {
                        name: "whisper_model_configured".to_string(),
                        status: "warn".to_string(),
                        message: "Whisper model path is not configured".to_string(),
                    });
                    checks.push(DiagnosticCheck {
                        name: "whisper_model_exists".to_string(),
                        status: "skip".to_string(),
                        message: "Skipped — no model path configured".to_string(),
                    });
                }
            }
            Err(e) => {
                for name in &[
                    "whisper_binary_configured",
                    "whisper_binary_exists",
                    "whisper_model_configured",
                    "whisper_model_exists",
                ] {
                    checks.push(DiagnosticCheck {
                        name: name.to_string(),
                        status: "fail".to_string(),
                        message: format!("Could not read settings: {e}"),
                    });
                }
            }
        }
        checks
    }

    fn check_providers_configured(&self) -> DiagnosticCheck {
        match self.db.lock() {
            Ok(conn) => {
                let result: Result<i64, _> = conn.query_row(
                    "SELECT COUNT(*) FROM providers WHERE enabled = 1",
                    [],
                    |r| r.get(0),
                );
                match result {
                    Ok(n) if n > 0 => DiagnosticCheck {
                        name: "providers_configured".to_string(),
                        status: "pass".to_string(),
                        message: format!("{n} enabled provider(s) configured"),
                    },
                    Ok(_) => DiagnosticCheck {
                        name: "providers_configured".to_string(),
                        status: "warn".to_string(),
                        message: "No enabled providers configured".to_string(),
                    },
                    Err(e) => DiagnosticCheck {
                        name: "providers_configured".to_string(),
                        status: "warn".to_string(),
                        message: format!("Could not query providers: {e}"),
                    },
                }
            }
            Err(_) => DiagnosticCheck {
                name: "providers_configured".to_string(),
                status: "warn".to_string(),
                message: "Database lock unavailable".to_string(),
            },
        }
    }

    fn check_ollama_reachable(&self) -> DiagnosticCheck {
        let base_url: Option<String> = match self.db.lock() {
            Ok(conn) => conn
                .query_row(
                    "SELECT base_url FROM providers \
                     WHERE provider_type = 'ollama' AND enabled = 1 LIMIT 1",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .filter(|u: &String| !u.is_empty()),
            Err(_) => None,
        };

        let Some(url_base) = base_url else {
            return DiagnosticCheck {
                name: "ollama_reachable".to_string(),
                status: "skip".to_string(),
                message: "No enabled Ollama provider configured".to_string(),
            };
        };

        let url = format!("{}/api/tags", url_base.trim_end_matches('/'));
        let agent = ureq::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build();

        match agent.get(&url).call() {
            Ok(resp) if resp.status() == 200 => DiagnosticCheck {
                name: "ollama_reachable".to_string(),
                status: "pass".to_string(),
                message: format!("Ollama is reachable at {url_base}"),
            },
            Ok(resp) => DiagnosticCheck {
                name: "ollama_reachable".to_string(),
                status: "warn".to_string(),
                message: format!("Ollama responded with HTTP {}", resp.status()),
            },
            Err(e) => DiagnosticCheck {
                name: "ollama_reachable".to_string(),
                status: "warn".to_string(),
                message: format!("Ollama not reachable: {e}"),
            },
        }
    }

    fn check_shortcut_configured(&self) -> DiagnosticCheck {
        // Read the configured shortcut from DB instead of hardcoding the default.
        let shortcut_str = match self.db.lock() {
            Ok(conn) => SettingsRepository::new(&conn)
                .get()
                .map(|s| s.shortcut)
                .unwrap_or_else(|_| "CommandOrControl+Shift+Space".to_string()),
            Err(_) => {
                return DiagnosticCheck {
                    name: "shortcut_configured".to_string(),
                    status: "warn".to_string(),
                    message: "Database lock unavailable — could not read shortcut".to_string(),
                };
            }
        };
        // The shortcut string (key name) is not a secret and is safe to include here.
        match shortcut_str.parse::<tauri_plugin_global_shortcut::Shortcut>() {
            Ok(_) => DiagnosticCheck {
                name: "shortcut_configured".to_string(),
                status: "pass".to_string(),
                message: format!("Global shortcut '{shortcut_str}' is valid"),
            },
            Err(e) => DiagnosticCheck {
                name: "shortcut_configured".to_string(),
                status: "warn".to_string(),
                message: format!("Shortcut '{shortcut_str}' is not parseable: {e}"),
            },
        }
    }

    fn check_audio_dir_accessible(&self) -> DiagnosticCheck {
        if self.audio_dir.is_dir() {
            DiagnosticCheck {
                name: "audio_dir_accessible".to_string(),
                status: "pass".to_string(),
                message: format!("Audio directory exists: {}", self.audio_dir.display()),
            }
        } else {
            DiagnosticCheck {
                name: "audio_dir_accessible".to_string(),
                status: "warn".to_string(),
                message: format!("Audio directory not found: {}", self.audio_dir.display()),
            }
        }
    }

    fn check_history_count(&self) -> DiagnosticCheck {
        match self.db.lock() {
            Ok(conn) => {
                match conn.query_row("SELECT COUNT(*) FROM history", [], |r| r.get::<_, i64>(0)) {
                    Ok(n) => DiagnosticCheck {
                        name: "history_count".to_string(),
                        status: "pass".to_string(),
                        message: format!("{n} history entries"),
                    },
                    Err(e) => DiagnosticCheck {
                        name: "history_count".to_string(),
                        status: "warn".to_string(),
                        message: format!("Could not count history: {e}"),
                    },
                }
            }
            Err(_) => DiagnosticCheck {
                name: "history_count".to_string(),
                status: "warn".to_string(),
                message: "Database lock unavailable".to_string(),
            },
        }
    }

    fn check_audio_dir_size(&self) -> DiagnosticCheck {
        match std::fs::read_dir(&self.audio_dir) {
            Ok(entries) => {
                let mut file_count: u64 = 0;
                let mut total_bytes: u64 = 0;
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            file_count += 1;
                            total_bytes += meta.len();
                        }
                    }
                }
                DiagnosticCheck {
                    name: "audio_dir_size".to_string(),
                    status: "pass".to_string(),
                    message: format!("{file_count} files, {total_bytes} bytes"),
                }
            }
            Err(e) => DiagnosticCheck {
                name: "audio_dir_size".to_string(),
                status: "warn".to_string(),
                message: format!("Could not read audio directory: {e}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup() -> (Arc<Mutex<Connection>>, PathBuf) {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let audio_dir = std::env::temp_dir().join(format!("tt_diag_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&audio_dir).unwrap();
        (Arc::new(Mutex::new(conn)), audio_dir)
    }

    fn find_check<'a>(report: &'a DiagnosticReport, name: &str) -> &'a DiagnosticCheck {
        report
            .checks
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("check '{name}' not found in report"))
    }

    #[test]
    fn test_backend_alive_always_passes() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "backend_alive");
        assert_eq!(check.status, "pass");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_database_reachable_passes_with_valid_db() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "database_reachable");
        assert_eq!(check.status, "pass");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_migrations_current_passes_after_run() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "migrations_current");
        assert_eq!(check.status, "pass");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_providers_configured_warns_with_empty_db() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "providers_configured");
        assert_eq!(check.status, "warn", "empty providers table should warn");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ollama_reachable_skips_when_no_ollama_provider() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "ollama_reachable");
        assert_eq!(check.status, "skip", "no Ollama provider → must skip");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_shortcut_configured_passes() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "shortcut_configured");
        assert_eq!(check.status, "pass");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_whisper_checks_warn_when_unconfigured() {
        let (db, dir) = setup();
        // Default settings have no whisper paths.
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        assert_eq!(
            find_check(&report, "whisper_binary_configured").status,
            "warn"
        );
        assert_eq!(find_check(&report, "whisper_binary_exists").status, "skip");
        assert_eq!(
            find_check(&report, "whisper_model_configured").status,
            "warn"
        );
        assert_eq!(find_check(&report, "whisper_model_exists").status, "skip");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_report_has_all_14_checks() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        assert_eq!(
            report.checks.len(),
            14,
            "report must contain exactly 14 checks"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_generated_at_is_non_empty() {
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        assert!(!report.generated_at.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Phase 10 diagnostics fix tests ─────────────────────────────────────

    #[test]
    fn test_diagnostics_migrations_version_5_passes() {
        // After all migrations (including 005), MAX(version) = 5 → pass.
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "migrations_current");
        assert_eq!(
            check.status, "pass",
            "migrations_current must pass after migration 005 is applied"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_diagnostics_migrations_low_version_warns() {
        // Insert only version 1 → MAX = 1 < 5 → warn.
        let conn = Connection::open_in_memory().unwrap();
        // Create schema_migrations table manually without running full migrations.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );
            INSERT INTO schema_migrations (version, applied_at) VALUES (1, 'now');
            CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY);
            ",
        )
        .unwrap();
        let db = Arc::new(Mutex::new(conn));
        let dir = std::env::temp_dir().join(format!("tt_diag_lowver_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "migrations_current");
        assert_eq!(
            check.status, "warn",
            "migrations_current must warn when MAX(version) < 5"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_diagnostics_shortcut_default_passes() {
        // Default shortcut in DB is "CommandOrControl+Shift+Space" — parseable → pass.
        let (db, dir) = setup();
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "shortcut_configured");
        assert_eq!(check.status, "pass");
        assert!(
            check.message.contains("CommandOrControl+Shift+Space"),
            "message must show the actual shortcut string"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_diagnostics_shortcut_reads_configured_value() {
        use crate::db::repositories::SettingsRepository;
        let (db, dir) = setup();
        // Save a custom shortcut.
        {
            let conn = db.lock().unwrap();
            let mut s = SettingsRepository::new(&conn).get().unwrap();
            s.shortcut = "CommandOrControl+Shift+D".to_string();
            SettingsRepository::new(&conn).upsert(&s).unwrap();
        }
        let svc = DiagnosticsService::new(db, dir.clone());
        let report = svc.run_diagnostics().unwrap();
        let check = find_check(&report, "shortcut_configured");
        assert_eq!(check.status, "pass");
        assert!(
            check.message.contains("CommandOrControl+Shift+D"),
            "message must show the custom shortcut, not the hardcoded default"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
