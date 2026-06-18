use crate::proto::device_service::v1::device_service_server::DeviceServiceServer;
use crate::service::TrueNasDriveTempService;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;

async fn cleanup_uds(path: &PathBuf) {
    if let Err(err) = tokio::fs::remove_file(path).await
        && err.kind() != std::io::ErrorKind::NotFound
    {
        log::error!("Failed to remove stale socket {}: {err}", path.display());
    }
}

pub(super) async fn serve_device_service(
    service: TrueNasDriveTempService,
    socket_path: PathBuf,
    run_token: CancellationToken,
) -> Result<()> {
    cleanup_uds(&socket_path).await;
    let uds = UnixListener::bind(&socket_path)
        .with_context(|| format!("failed to bind UDS {}", socket_path.display()))?;
    let uds_stream = UnixListenerStream::new(uds);

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<DeviceServiceServer<TrueNasDriveTempService>>()
        .await;

    Server::builder()
        .add_service(DeviceServiceServer::new(service))
        .add_service(health_service)
        .serve_with_incoming_shutdown(uds_stream, run_token.cancelled())
        .await?;

    cleanup_uds(&socket_path).await;
    Ok(())
}
