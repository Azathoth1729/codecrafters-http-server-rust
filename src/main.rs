use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

use anyhow::Result;

fn main() -> Result<()> {
    let ip_arr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4221);
    let listener = TcpListener::bind(ip_arr).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("accepted new connection");

        let http_response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write(http_response.as_bytes())?;
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;
        println!("received:\n{buffer}")
    }
    Ok(())
}
