use super::CachedState;
use crate::config::PluginConfig;
use crate::polling::TempChannel;
use crate::polling::channels::{aggregate_id, failsafe_aggregate_max_channel};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SharedState {
    inner: Arc<RwLock<CachedState>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CachedState {
                temps: Vec::new(),
                last_updated: None,
                last_error: None,
                missing: Vec::new(),
            })),
        }
    }

    pub async fn snapshot(&self) -> CachedState {
        self.inner.read().await.clone()
    }

    pub async fn set_success(&self, temps: Vec<TempChannel>, missing: Vec<String>) {
        let mut guard = self.inner.write().await;
        guard.temps = temps;
        guard.last_updated = Some(SystemTime::now());
        guard.last_error = None;
        guard.missing = missing;
    }

    pub async fn set_error(&self, error: String, config: &PluginConfig) {
        let mut guard = self.inner.write().await;
        guard.last_error = Some(error);
        guard.missing.clear();

        if let Some(celsius) = config.failsafe_aggregate_max {
            let id = aggregate_id("max");
            if let Some(channel) = guard.temps.iter_mut().find(|temp| temp.id == id) {
                *channel = failsafe_aggregate_max_channel(celsius, channel.number);
            } else {
                let number = guard.temps.len() as u32 + 1;
                guard
                    .temps
                    .push(failsafe_aggregate_max_channel(celsius, number));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> PluginConfig {
        toml::from_str(
            r#"
                failsafe_aggregate_max = 72.5

                [truenas]
                url = "wss://nas/api/current"
                api_key = "secret"
            "#,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn set_error_adds_failsafe_aggregate_max_channel() {
        let state = SharedState::new();
        state.set_error("boom".to_string(), &test_config()).await;

        let snapshot = state.snapshot().await;
        assert_eq!(snapshot.last_error.as_deref(), Some("boom"));
        assert_eq!(snapshot.temps.len(), 1);
        assert_eq!(snapshot.temps[0].id, "aggregate_max");
        assert_eq!(snapshot.temps[0].celsius, 72.5);
    }
}
