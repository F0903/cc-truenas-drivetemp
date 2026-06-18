use super::TrueNasConfig;
use super::normalize::normalize_string_vec;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    pub truenas: TrueNasConfig,
    pub poll_interval_seconds: u64,
    pub disks: Vec<String>,
    pub exclude_disks: Vec<String>,
    pub failsafe_aggregate_max: Option<f64>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            truenas: TrueNasConfig::default(),
            poll_interval_seconds: 300,
            disks: Vec::new(),
            exclude_disks: Vec::new(),
            failsafe_aggregate_max: None,
        }
    }
}

impl PluginConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let mut config: PluginConfig = toml::from_str(&text)
            .with_context(|| format!("failed to parse config {}", path.display()))?;

        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        if let Some(api_key_file) = &config.truenas.api_key_file
            && api_key_file.is_relative()
        {
            config.truenas.api_key_file = Some(PathBuf::from(base_dir).join(api_key_file));
        }

        config.normalize();
        config.validate()?;
        Ok(config)
    }

    pub(super) fn normalize(&mut self) {
        normalize_string_vec(&mut self.disks);
        normalize_string_vec(&mut self.exclude_disks);
        self.truenas.normalize();
    }

    pub(super) fn validate(&self) -> Result<()> {
        if !self.truenas.url.starts_with("ws://") && !self.truenas.url.starts_with("wss://") {
            bail!("truenas.url must start with ws:// or wss://");
        }
        if self.poll_interval_seconds == 0 {
            bail!("poll_interval_seconds must be greater than zero");
        }
        if let Some(failsafe_aggregate_max) = self.failsafe_aggregate_max
            && !failsafe_aggregate_max.is_finite()
        {
            bail!("failsafe_aggregate_max must be finite when set");
        }
        if self.truenas.timeout_seconds == 0 {
            bail!("truenas.timeout_seconds must be greater than zero");
        }
        Ok(())
    }
}
