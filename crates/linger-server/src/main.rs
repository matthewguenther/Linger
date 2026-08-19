//! The server's entry point: read env config, open the database, serve.

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

    let bind = config.bind;
    let domain = config.domain.clone();
    let state = linger_server::AppState::build(db, config).await?;

    // First run: no users yet, so hand the host their one-time setup URL.
    // Printed to stdout on purpose — `docker compose logs linger` is the flow.
    if let Some(token) = state.setup.peek() {
        let host = domain.unwrap_or_else(|| bind.to_string());
        println!("\n  ┌─────────────────────────────────────────────────");
        println!("  │  This server isn't set up yet.");
        println!("  │  Open:  http://{host}/setup?token={token}");
        println!("  │  (the link works once, then never again)");
        println!("  └─────────────────────────────────────────────────\n");
    }

    let app = linger_server::app(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("listening on {bind}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
