use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

use anyhow::Result;
use tracing::info;

use crate::connection::Connection;
use crate::server::Handler;

pub mod connection;
pub mod response;
pub mod server;

fn main() -> Result<()> {
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
    let listener = TcpListener::bind(ip_arr)?;

    info!("connected to {:?}", ip_arr);
    
    for stream in listener.incoming() {
        let mut handler = Handler::new(Connection::try_new(stream?)?);
        handler.run()?;
    }
    Ok(())
}
