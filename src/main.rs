use anyhow::Result;
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::server::Handler;

pub mod common;
pub mod connection;
pub mod response;
pub mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path of file to send on http server
    #[arg(short, long)]
    directory: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let directory = args.directory;

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

        let directory = directory.clone();

        tokio::spawn(async move {
            info!(peer_addr = ?peer_addr, "new connection");
            if let Err(err) = handler.run(Handler::handle_request, directory).await {
                error!(cause = ?err, "connection error");
            }
        });
    }
}
