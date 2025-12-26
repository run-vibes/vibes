use tracing::info;

/// Start a stub server that binds the port but doesn't accept connections.
/// Full implementation in milestone 1.4.
pub async fn start_stub(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            info!("Vibes server listening on http://{}", addr);
            // Keep the listener alive but don't accept connections
            std::future::pending::<()>().await;
            drop(listener);
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            info!("Port {} already in use (another vibes instance?)", port);
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
