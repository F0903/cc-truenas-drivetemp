use crate::SERVICE_ID;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(super) struct Args {
    #[clap(short, long)]
    pub(super) debug: bool,

    #[clap(long)]
    config: Option<PathBuf>,

    #[clap(long)]
    socket: Option<PathBuf>,
}

impl Args {
    pub(super) fn read() -> Self {
        Self::parse()
    }

    pub(super) fn config_path(&self) -> PathBuf {
        self.config.clone().unwrap_or_else(default_config_path)
    }

    pub(super) fn socket_path(&self) -> PathBuf {
        self.socket.clone().unwrap_or_else(default_socket_path)
    }
}

fn default_config_path() -> PathBuf {
    PathBuf::from(format!(
        "/etc/coolercontrol/plugins/{SERVICE_ID}/config.toml"
    ))
}

fn default_socket_path() -> PathBuf {
    PathBuf::from(format!("/tmp/{SERVICE_ID}.sock"))
}
