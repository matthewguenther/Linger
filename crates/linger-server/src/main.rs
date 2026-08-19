//! The stoop's entry point: read env config, open the database, serve.

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = linger_server::config::Config::from_env()?;
    tracing::info!(data_dir = %config.data_dir.display(), bind = %config.bind, "starting linger-server");

    let db = linger_server::db::init(&config.db_path()).await?;
    tokio::fs::create_dir_all(config.objects_dir()).await?;

    // M1 adds the first-run flow here: if no host account exists, print a
    // one-time host-setup URL with a token to stdout (ARCHITECTURE §9).

    let bind = config.bind;
    let state = linger_server::AppState::new(db, config);
    let app = linger_server::app(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("listening on {bind}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    // SIGINT or SIGTERM: finish in-flight requests, then stop. Docker sends
    // SIGTERM on `compose down`; without this, every restart looks like a crash.
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
