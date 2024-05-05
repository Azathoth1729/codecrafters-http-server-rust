use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use crate::connection::Connection;
use crate::response::Response;
use crate::server::Handler;
use anyhow::{Context, Result};
use hyper::http::response::Builder;
use hyper::StatusCode;

pub mod connection;
pub mod response;
pub mod server;

fn handle_stream(stream: TcpStream) -> Result<()> {
    let reader_stream = stream;
    let mut writer_stream = reader_stream.try_clone()?;

    let mut reader = BufReader::new(reader_stream);
    let mut read_buf = String::new();
    loop {
        let mut buf = String::new();
        let size = reader.read_line(&mut buf)?;

        if size == 0 || buf.as_bytes() == b"\r\n" {
            break;
        }
        read_buf.push_str(&buf)
    }
    eprintln!("req_str:\n{:?}", read_buf);

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(&read_buf.as_bytes())
        .context("request parse failed")?;

    if let Some(path) = req.path {
        let response = match path {
            "/" => Response::from(Builder::new().status(StatusCode::OK).body(())?),
            _ => Response::from(Builder::new().status(StatusCode::NOT_FOUND).body(())?),
        };
        let response_str = response.to_string();
        eprintln!("response_str:\n{response_str:?}");

        writer_stream.write_all(response_str.as_bytes())?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let port = 4221;
    let ip_arr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = TcpListener::bind(ip_arr)?;

    for stream in listener.incoming() {
        let mut handler = Handler::new(Connection::try_new(stream?)?);
        handler.run()?;
    }
    Ok(())
}
