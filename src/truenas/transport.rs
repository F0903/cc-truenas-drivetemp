use crate::config::TrueNasConfig;
use anyhow::{Context, Result};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub(super) type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(super) async fn connect_websocket(config: &TrueNasConfig) -> Result<WsStream> {
    tokio::time::timeout(config.request_timeout(), connect_websocket_inner(config))
        .await
        .with_context(|| {
            format!(
                "TrueNAS websocket connection timed out after {:?}",
                config.request_timeout()
            )
        })?
}

async fn connect_websocket_inner(config: &TrueNasConfig) -> Result<WsStream> {
    if config.url.starts_with("wss://") && !config.verify_tls {
        let connector = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .context("failed to build insecure TLS connector")?;
        let connector = tokio_tungstenite::Connector::NativeTls(connector);
        let (stream, _) = tokio_tungstenite::connect_async_tls_with_config(
            &config.url,
            None,
            false,
            Some(connector),
        )
        .await
        .with_context(|| format!("failed to connect to {}", config.url))?;
        return Ok(stream);
    }

    let (stream, _) = tokio_tungstenite::connect_async(&config.url)
        .await
        .with_context(|| format!("failed to connect to {}", config.url))?;
    Ok(stream)
}
