use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use anyhow::{Context, Result};
use crate::response::Response;
use hyper::http::response::Builder;
use hyper::StatusCode;

pub mod response;

fn handle_stream(stream: TcpStream) -> Result<()> {
    let reader_stream = stream;
    let mut writer_steam = reader_stream.try_clone()?;

    let mut reader = BufReader::new(reader_stream);
    let mut req_str = String::new();
    loop {
        let mut buf = String::new();
        let size = reader.read_line(&mut buf)?;

        if size == 0 || buf.as_bytes() == b"\r\n" {
            break;
        }
        req_str.push_str(&buf)
    }
    eprintln!("req_str:\n{:?}", req_str);

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(&req_str.as_bytes()).context("request parse failed")?;


    if let Some(path) = req.path {
        let response = match path {
            "/" => {
                Response::from(
                    Builder::new().status(StatusCode::OK).body(())?
                )
            }
            _ => {
                Response::from(
                    Builder::new().status(StatusCode::NOT_FOUND).body(())?
                )
            }
        };
        let response_str = response.to_string();
        eprintln!("response_str:\n{response_str:?}");
        
        writer_steam.write_all(response_str.as_bytes())?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let port = 4221;
    let ip_arr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let listener = TcpListener::bind(ip_arr).unwrap();

    for stream in listener.incoming() {
        let stream = stream?;
        handle_stream(stream).context("handle stream failed")?;
    }
    Ok(())
}
