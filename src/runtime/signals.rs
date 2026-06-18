use log::info;
use tokio_util::sync::CancellationToken;

pub(super) fn setup_termination_signals() -> CancellationToken {
    let run_token = CancellationToken::new();
    let token = run_token.clone();
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        token.cancel();
        info!("Shutting down");
    });
    run_token
}

async fn wait_for_shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    let sigterm = async {
        signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    let sigint = async {
        signal(SignalKind::interrupt())
            .expect("failed to install SIGINT handler")
            .recv()
            .await;
    };
    let sigquit = async {
        signal(SignalKind::quit())
            .expect("failed to install SIGQUIT handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = ctrl_c => {},
        () = sigterm => {},
        () = sigint => {},
        () = sigquit => {},
    }
}
