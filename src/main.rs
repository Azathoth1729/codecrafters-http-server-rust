use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::server::Handler;

pub mod connection;
pub mod response;
pub mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let tracing_subscriber = tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_level(true)
        .finish();

    tracing::subscriber::set_global_default(tracing_subscriber)?;

    let port = 4221;
    let ip_arr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = TcpListener::bind(ip_arr).await?;

    info!("serving on {:?}", ip_arr);
    loop {
        let (stream, peer_addr) = listener.accept().await?;

        let mut handler = Handler::new(stream);

        tokio::spawn(async move {
            info!(peer_addr = ?peer_addr, "new connection");
            if let Err(err) = handler.run().await {
                error!(cause = ?err, "connection error");
            }
        });
    }
}
