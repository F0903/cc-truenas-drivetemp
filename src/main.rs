#[cfg(not(target_os = "linux"))]
compile_error!("cc-truenas-drivetemp is Linux-only like CoolerControl currently is.");

mod config;
mod polling;
mod proto;
mod runtime;
mod service;
mod truenas;

use anyhow::Result;

pub const SERVICE_ID: &str = env!("CARGO_PKG_NAME");
pub const DEVICE_NAME: &str = "TrueNAS Drive Temperatures";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    runtime::run().await
}
