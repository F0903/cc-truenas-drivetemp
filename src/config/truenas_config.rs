use super::normalize::take_non_empty;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TrueNasConfig {
    pub url: String,
    pub username: Option<String>,
    pub api_key: Option<String>,
    pub api_key_file: Option<PathBuf>,
    pub api_key_env: Option<String>,
    pub verify_tls: bool,
    pub timeout_seconds: u64,
}

impl Default for TrueNasConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            username: None,
            api_key: None,
            api_key_file: None,
            api_key_env: Some("TRUENAS_API_KEY".to_string()),
            verify_tls: true,
            timeout_seconds: 20,
        }
    }
}

impl TrueNasConfig {
    pub fn resolve_api_key(&self) -> Result<String> {
        if let Some(api_key) = self.api_key.as_ref().filter(|value| !value.is_empty()) {
            return Ok(api_key.trim().to_string());
        }

        if let Some(api_key_file) = &self.api_key_file {
            return fs::read_to_string(api_key_file)
                .map(|value| value.trim().to_string())
                .with_context(|| {
                    format!("failed to read api_key_file {}", api_key_file.display())
                });
        }

        if let Some(api_key_env) = &self.api_key_env
            && let Ok(value) = env::var(api_key_env)
            && !value.trim().is_empty()
        {
            return Ok(value.trim().to_string());
        }

        bail!("no TrueNAS API key configured");
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    pub(super) fn normalize(&mut self) {
        self.url = self.url.trim().to_string();
        self.username = take_non_empty(self.username.take());
        self.api_key = take_non_empty(self.api_key.take());
        self.api_key_env = take_non_empty(self.api_key_env.take());
    }
}
