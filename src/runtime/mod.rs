mod args;
mod logging;
mod signals;
mod socket;

use crate::config::PluginConfig;
use crate::polling::{SharedState, poll_loop, poll_once_into_state};
use crate::service::TrueNasDriveTempService;
use crate::{SERVICE_ID, VERSION};
use anyhow::Result;
use args::Args;
use log::{info, warn};
use logging::setup_logging;
use signals::setup_termination_signals;
use socket::serve_device_service;
use std::time::Instant;

pub async fn run() -> Result<()> {
    let args = Args::read();
    setup_logging(args.debug)?;

    let config_path = args.config_path();
    let socket_path = args.socket_path();

    info!("Starting {SERVICE_ID} v{VERSION}");
    info!("Loading config from {}", config_path.display());
    let config = PluginConfig::load(&config_path)?;
    let state = SharedState::new();

    if let Err(err) = poll_once_into_state(&config, &state).await {
        warn!("Initial TrueNAS poll failed: {err:#}");
    }

    let run_token = setup_termination_signals();
    tokio::spawn(poll_loop(config.clone(), state.clone(), run_token.clone()));

    let service = TrueNasDriveTempService::new(config, state, Instant::now());
    serve_device_service(service, socket_path, run_token).await?;
    info!("Stopped {SERVICE_ID}");
    Ok(())
}
