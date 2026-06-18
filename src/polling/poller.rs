use super::channels::to_channels;
use super::poll_snapshot::PollSnapshot;
use super::selection::select_disk_names;
use super::state::SharedState;
use crate::config::PluginConfig;
use crate::truenas::{DiskInfo, TrueNasClient};
use anyhow::{Result, bail};
use log::{info, warn};
use std::collections::HashMap;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn poll_loop(config: PluginConfig, state: SharedState, run_token: CancellationToken) {
    let interval = Duration::from_secs(config.poll_interval_seconds);
    while !run_token.is_cancelled() {
        tokio::select! {
            () = run_token.cancelled() => break,
            () = tokio::time::sleep(interval) => {
                if let Err(err) = poll_once_into_state(&config, &state).await {
                    warn!("TrueNAS poll failed: {err:#}");
                }
            }
        }
    }
}

pub async fn poll_once_into_state(config: &PluginConfig, state: &SharedState) -> Result<()> {
    match poll_once(config).await {
        Ok(PollSnapshot { temps, missing }) => {
            let count = temps.len();
            state.set_success(temps, missing.clone()).await;
            if missing.is_empty() {
                info!("Updated {count} TrueNAS temperature channels");
            } else {
                warn!(
                    "Updated {count} TrueNAS temperature channels; missing temperatures for: {}",
                    missing.join(", ")
                );
            }
            Ok(())
        }
        Err(err) => {
            let error = format!("{err:#}");
            state.set_error(error.clone(), config).await;
            Err(anyhow::anyhow!(error))
        }
    }
}

async fn poll_once(config: &PluginConfig) -> Result<PollSnapshot> {
    let mut client = TrueNasClient::connect(config.truenas.clone()).await?;
    let metadata = load_disk_metadata(&mut client, config).await?;
    let disk_names = select_disk_names(&metadata, config);
    if disk_names.is_empty() {
        bail!("no disks selected for temperature polling");
    }

    let (temperatures, missing) = client.query_temperatures(&disk_names, &metadata).await?;
    Ok(PollSnapshot {
        temps: to_channels(&temperatures),
        missing,
    })
}

async fn load_disk_metadata(
    client: &mut TrueNasClient,
    config: &PluginConfig,
) -> Result<HashMap<String, DiskInfo>> {
    if !config.disks.is_empty() {
        return Ok(config
            .disks
            .iter()
            .map(|name| (name.clone(), DiskInfo::from_name(name)))
            .collect());
    }

    Ok(client
        .query_disks()
        .await?
        .into_iter()
        .map(|disk| (disk.name.clone(), disk))
        .collect())
}
