use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_rustls::server::TlsStream;
use tracing::{error, info, Instrument};

#[tracing::instrument(skip_all)]
pub async fn handle(conn: TlsStream<TcpStream>, addr: SocketAddr) -> Result<()> {
    let raw_fd = conn.get_ref().0.as_raw_fd();
    let mut connection = h2::server::handshake(conn).await?;
    info!("H2 connection bound");
    let ping_pong = Arc::new(Mutex::new(
        connection.ping_pong().context("ping pong error")?,
    ));

    while let Some(result) = connection.accept().await {
        let (mut request, mut respond) = result?;
        let ping_pong = ping_pong.clone();
        // TODO: spawn a ping task
        tokio::spawn(
            async move {
                if let Err(e) =
                    super::http2::handle(&mut request, &mut respond, raw_fd, addr, ping_pong).await
                {
                    error!("error while handling request: {}", e);
                }
            }
            .in_current_span(),
        );
    }

    connection.graceful_shutdown();
    Ok(())
}
