use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Deserialize;

use crate::db::repositories::SettingsRepository;
use crate::errors::AppError;
use crate::models::{AiProvider, CleanupResult, PrivacyOperation};
use crate::services::{PrivacyService, ProvidersService};

const DEFAULT_SYSTEM_PROMPT: &str =
    "You are a text cleanup assistant. Fix grammar, punctuation, and formatting. \
     Return only the cleaned text with no explanation.";

pub struct CleanupService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl CleanupService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    pub fn cleanup(&self, raw_text: &str, provider_id: &str) -> Result<CleanupResult, AppError> {
        let providers_svc = ProvidersService::new(self.db.clone());
        let provider = providers_svc.get_provider(provider_id)?;

        // Reject disabled providers before any further processing.
        if !provider.enabled {
            return Err(AppError::CleanupError(format!(
                "Provider '{}' is disabled",
                provider.name
            )));
        }

        // Privacy check before any HTTP call — must happen before network access.
        let operation_type = if provider.provider_type == "ollama" {
            "local_cleanup"
        } else {
            "cloud_cleanup"
        };
        let decision =
            PrivacyService::new(self.db.clone()).enforce_privacy_preview(PrivacyOperation {
                operation_type: operation_type.to_string(),
                provider_id: Some(provider_id.to_string()),
            })?;
        if !decision.allowed {
            return Err(AppError::PrivacyBlocked(decision.reason));
        }

        let system_prompt = self.load_system_prompt();
        let start = Instant::now();

        let cleaned_text = match provider.provider_type.as_str() {
            "ollama" => self.ollama_cleanup(&provider, raw_text, &system_prompt)?,
            "openai_compatible" => {
                let api_key = providers_svc.get_api_key(provider_id)?;
                self.openai_cleanup(&provider, raw_text, &system_prompt, &api_key)?
            }
            t => {
                return Err(AppError::CleanupError(format!(
                    "Unknown provider type: {t}"
                )))
            }
        };

        Ok(CleanupResult {
            cleaned_text,
            provider_id: provider_id.to_string(),
            provider_name: provider.name,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Load the system prompt from the active dictation mode.
    /// Falls back to DEFAULT_SYSTEM_PROMPT when the mode has an empty prompt.
    fn load_system_prompt(&self) -> String {
        let conn = match self.db.lock() {
            Ok(c) => c,
            Err(_) => return DEFAULT_SYSTEM_PROMPT.to_string(),
        };
        let settings = match SettingsRepository::new(&conn).get() {
            Ok(s) => s,
            Err(_) => return DEFAULT_SYSTEM_PROMPT.to_string(),
        };
        let mode_prompt: rusqlite::Result<String> = conn.query_row(
            "SELECT system_prompt FROM modes WHERE id = ?1",
            rusqlite::params![settings.active_mode],
            |row| row.get(0),
        );
        match mode_prompt {
            Ok(p) if !p.is_empty() => p,
            _ => DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    fn ollama_cleanup(
        &self,
        provider: &AiProvider,
        raw_text: &str,
        system_prompt: &str,
    ) -> Result<String, AppError> {
        let url = format!("{}/api/generate", provider.base_url.trim_end_matches('/'));
        let prompt = Self::build_ollama_prompt(system_prompt, raw_text);

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(serde_json::json!({
                "model": provider.model,
                "prompt": prompt,
                "stream": false,
            }))
            .map_err(|e| AppError::CleanupError(format!("Ollama request failed: {e}")))?;

        let body: OllamaResponse = response
            .into_json()
            .map_err(|e| AppError::CleanupError(format!("Ollama response parse failed: {e}")))?;

        Ok(body.response)
    }

    fn openai_cleanup(
        &self,
        provider: &AiProvider,
        raw_text: &str,
        system_prompt: &str,
        api_key: &str,
    ) -> Result<String, AppError> {
        let url = format!(
            "{}/chat/completions",
            provider.base_url.trim_end_matches('/')
        );

        #[derive(Deserialize)]
        struct Message {
            content: String,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<Choice>,
        }

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {api_key}"))
            .set("Content-Type", "application/json")
            .send_json(serde_json::json!({
                "model": provider.model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user",   "content": raw_text},
                ],
            }))
            .map_err(|e| AppError::CleanupError(format!("OpenAI request failed: {e}")))?;

        let body: OpenAIResponse = response
            .into_json()
            .map_err(|e| AppError::CleanupError(format!("OpenAI response parse failed: {e}")))?;

        body.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| AppError::CleanupError("Provider response had no choices".to_string()))
    }

    /// Build the Ollama prompt string. Public for unit testing.
    pub fn build_ollama_prompt(system_prompt: &str, raw_text: &str) -> String {
        format!("{system_prompt}\n\nText to clean:\n{raw_text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::SettingsRepository;
    use rusqlite::Connection;

    fn migrated_db() -> Arc<Mutex<rusqlite::Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn insert_provider(
        db: &Arc<Mutex<rusqlite::Connection>>,
        id: &str,
        provider_type: &str,
        enabled: bool,
    ) {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT INTO providers \
             (id, name, provider_type, base_url, model, \
              enabled, use_for_cleanup, use_for_transcription, api_key_set) \
             VALUES (?1, ?2, ?3, 'http://localhost:1', 'model', ?4, 1, 0, 0)",
            rusqlite::params![
                id,
                format!("{provider_type}-name"),
                provider_type,
                enabled as i64,
            ],
        )
        .unwrap();
    }

    // ── Prompt construction ───────────────────────────────────────────────────

    #[test]
    fn test_prompt_construction_default_when_mode_prompt_empty() {
        // smart mode seeds with empty system_prompt (migration 001)
        let db = migrated_db();
        let svc = CleanupService::new(db);
        let prompt = svc.load_system_prompt();
        assert_eq!(prompt, DEFAULT_SYSTEM_PROMPT);
    }

    #[test]
    fn test_prompt_construction_uses_active_mode_prompt_when_non_empty() {
        let db = migrated_db();
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "UPDATE modes SET system_prompt = 'Custom prompt.' WHERE id = 'smart'",
                [],
            )
            .unwrap();
        }
        let svc = CleanupService::new(db);
        let prompt = svc.load_system_prompt();
        assert_eq!(prompt, "Custom prompt.");
    }

    #[test]
    fn test_build_ollama_prompt_combines_system_and_raw() {
        let result = CleanupService::build_ollama_prompt("Fix grammar.", "hello world");
        assert_eq!(result, "Fix grammar.\n\nText to clean:\nhello world");
    }

    // ── Provider validation ───────────────────────────────────────────────────

    #[test]
    fn test_cleanup_rejects_disabled_provider() {
        let db = migrated_db();
        insert_provider(&db, "disabled-p", "ollama", false);
        let svc = CleanupService::new(db);
        let err = svc.cleanup("hello", "disabled-p").unwrap_err();
        assert!(
            matches!(err, AppError::CleanupError(_)),
            "expected CleanupError for disabled provider, got {err:?}"
        );
    }

    #[test]
    fn test_cleanup_unknown_provider_type_returns_cleanup_error() {
        // custom_http is not "ollama" or "openai_compatible"
        let db = migrated_db();
        insert_provider(&db, "custom-p", "custom_http", true);
        let svc = CleanupService::new(db);
        let err = svc.cleanup("hello", "custom-p").unwrap_err();
        assert!(
            matches!(err, AppError::CleanupError(_)),
            "expected CleanupError for unknown type, got {err:?}"
        );
    }

    // ── Privacy enforcement ───────────────────────────────────────────────────

    #[test]
    fn test_privacy_cloud_cleanup_blocked_in_local_only_mode() {
        let db = migrated_db();
        {
            let conn = db.lock().unwrap();
            let mut s = SettingsRepository::new(&conn).get().unwrap();
            s.local_only_mode = true;
            SettingsRepository::new(&conn).upsert(&s).unwrap();
        }
        insert_provider(&db, "oai-p", "openai_compatible", true);
        let svc = CleanupService::new(db);
        let err = svc.cleanup("hello", "oai-p").unwrap_err();
        assert!(
            matches!(err, AppError::PrivacyBlocked(_)),
            "expected PrivacyBlocked for cloud cleanup in local-only mode, got {err:?}"
        );
    }

    #[test]
    fn test_privacy_local_cleanup_allowed_in_local_only_mode() {
        // Ollama is local_cleanup — allowed even when local_only_mode = true.
        // The HTTP call to port 1 will fail, but the error must NOT be PrivacyBlocked.
        let db = migrated_db();
        {
            let conn = db.lock().unwrap();
            let mut s = SettingsRepository::new(&conn).get().unwrap();
            s.local_only_mode = true;
            SettingsRepository::new(&conn).upsert(&s).unwrap();
        }
        insert_provider(&db, "ollama-p", "ollama", true);
        let svc = CleanupService::new(db);
        let err = svc.cleanup("hello", "ollama-p").unwrap_err();
        assert!(
            !matches!(err, AppError::PrivacyBlocked(_)),
            "Ollama must NOT be blocked by privacy in local-only mode, got {err:?}"
        );
        assert!(
            matches!(err, AppError::CleanupError(_)),
            "Expected CleanupError from network failure on port 1, got {err:?}"
        );
    }
}
