use std::sync::{Arc, Mutex};

use crate::db::repositories::SettingsRepository;
use crate::errors::AppError;
use crate::models::{PrivacyDecision, PrivacyOperation, PrivacySummary};

/// Operation types that are explicitly permitted when local-only mode is active.
const ALLOWED_IN_LOCAL_MODE: &[&str] = &[
    "local_transcription",
    "local_cleanup",
    "read_settings",
    "write_settings",
    "read_history",
    "write_history",
    "read_vocabulary",
    "write_vocabulary",
    "read_modes",
    "write_modes",
];

/// Operation types that are explicitly cloud/provider-related and must be blocked.
const CLOUD_OPS: &[&str] = &[
    "cloud_transcription",
    "cloud_cleanup",
    "provider_test",
    "provider_call",
    "openai",
    "anthropic",
    "remote_model",
];

pub struct PrivacyService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl PrivacyService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    /// Returns the current privacy state derived from persisted settings.
    pub fn get_privacy_status(&self) -> Result<PrivacySummary, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        let settings = SettingsRepository::new(&conn).get()?;

        let audio_retention_days = if settings.audio_history_enabled {
            settings.retention_days
        } else {
            0
        };

        let (cloud_allowed, reason) = if settings.local_only_mode {
            (
                false,
                "local-only mode is active; no data may leave this device".to_string(),
            )
        } else {
            (
                true,
                "cloud providers are enabled; audio and text may leave this device \
                 if a provider is configured"
                    .to_string(),
            )
        };

        Ok(PrivacySummary {
            local_only_mode: settings.local_only_mode,
            audio_retention_days,
            history_retention_days: settings.retention_days,
            cloud_allowed,
            reason,
        })
    }

    /// Evaluates whether an operation is permitted under the current privacy settings.
    ///
    /// Fails closed: unknown operations are blocked when local-only mode is on.
    pub fn enforce_privacy_preview(
        &self,
        op: PrivacyOperation,
    ) -> Result<PrivacyDecision, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        let settings = SettingsRepository::new(&conn).get()?;

        if !settings.local_only_mode {
            return Ok(PrivacyDecision {
                allowed: true,
                reason: "local-only mode is off; cloud operations are permitted".to_string(),
            });
        }

        // local_only_mode is true — evaluate the operation type.
        let op_type = op.operation_type.as_str();

        if CLOUD_OPS.contains(&op_type) {
            return Ok(PrivacyDecision {
                allowed: false,
                reason: format!("operation '{op_type}' is not allowed in local-only mode"),
            });
        }

        if ALLOWED_IN_LOCAL_MODE.contains(&op_type) {
            return Ok(PrivacyDecision {
                allowed: true,
                reason: format!("operation '{op_type}' is permitted in local-only mode"),
            });
        }

        // Unknown operation — fail closed.
        Ok(PrivacyDecision {
            allowed: false,
            reason: format!("unknown operation '{op_type}' blocked by default in local-only mode"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::SettingsRepository;
    use crate::models::AppSettings;

    fn make_service(local_only: bool) -> PrivacyService {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        if local_only {
            let mut s = SettingsRepository::new(&conn).get().unwrap();
            s.local_only_mode = true;
            SettingsRepository::new(&conn).upsert(&s).unwrap();
        }
        PrivacyService::new(Arc::new(Mutex::new(conn)))
    }

    fn op(operation_type: &str) -> PrivacyOperation {
        PrivacyOperation {
            operation_type: operation_type.to_string(),
            provider_id: None,
        }
    }

    #[test]
    fn test_status_cloud_allowed_by_default() {
        let svc = make_service(false);
        let status = svc.get_privacy_status().unwrap();
        assert!(status.cloud_allowed);
        assert!(!status.local_only_mode);
    }

    #[test]
    fn test_status_blocked_in_local_only_mode() {
        let svc = make_service(true);
        let status = svc.get_privacy_status().unwrap();
        assert!(!status.cloud_allowed);
        assert!(status.local_only_mode);
    }

    #[test]
    fn test_status_audio_retention_zero_when_disabled() {
        let svc = make_service(false);
        let status = svc.get_privacy_status().unwrap();
        // audio_history_enabled defaults to false → audio_retention_days = 0
        assert_eq!(status.audio_retention_days, 0);
    }

    #[test]
    fn test_status_audio_retention_uses_settings_when_enabled() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let mut s = SettingsRepository::new(&conn).get().unwrap();
        s.audio_history_enabled = true;
        s.retention_days = 14;
        SettingsRepository::new(&conn).upsert(&s).unwrap();
        let svc = PrivacyService::new(Arc::new(Mutex::new(conn)));
        let status = svc.get_privacy_status().unwrap();
        assert_eq!(status.audio_retention_days, 14);
    }

    #[test]
    fn test_enforce_allows_cloud_when_not_local_mode() {
        let svc = make_service(false);
        let decision = svc
            .enforce_privacy_preview(op("cloud_transcription"))
            .unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn test_enforce_blocks_cloud_transcription_in_local_only() {
        let svc = make_service(true);
        let decision = svc
            .enforce_privacy_preview(op("cloud_transcription"))
            .unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn test_enforce_blocks_openai_in_local_only() {
        let svc = make_service(true);
        let decision = svc.enforce_privacy_preview(op("openai")).unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn test_enforce_blocks_provider_test_in_local_only() {
        let svc = make_service(true);
        let decision = svc.enforce_privacy_preview(op("provider_test")).unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn test_enforce_allows_local_transcription_in_local_only() {
        let svc = make_service(true);
        let decision = svc
            .enforce_privacy_preview(op("local_transcription"))
            .unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn test_enforce_blocks_unknown_op_in_local_only() {
        let svc = make_service(true);
        let decision = svc.enforce_privacy_preview(op("some_future_op")).unwrap();
        assert!(!decision.allowed);
        assert!(decision.reason.contains("unknown operation"));
    }

    #[test]
    fn test_enforce_allows_unknown_op_when_not_local_only() {
        let svc = make_service(false);
        let decision = svc.enforce_privacy_preview(op("some_future_op")).unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn test_history_retention_days_from_settings() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let mut s = AppSettings {
            active_mode: "smart".to_string(),
            local_only_mode: false,
            theme: "dark".to_string(),
            retention_days: 90,
            audio_history_enabled: false,
            clipboard_restore_enabled: false,
            whisper_binary_path: None,
            whisper_model_path: None,
        };
        SettingsRepository::new(&conn).upsert(&s).unwrap();
        let svc = PrivacyService::new(Arc::new(Mutex::new(conn)));
        let status = svc.get_privacy_status().unwrap();
        assert_eq!(status.history_retention_days, 90);
        // suppress unused warning
        s.retention_days = 90;
    }
}
