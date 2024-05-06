use crate::response::{BodyData, Response};
use anyhow::{anyhow, Context};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use tracing::info;

pub struct Connection {
    reader_stream: BufReader<TcpStream>,
    writer_stream: TcpStream,
    read_buf: String,
}

impl Connection {
    const MAX_HEADER_NUM: usize = 1 << 10;

    pub fn try_new(tcp_stream: TcpStream) -> anyhow::Result<Self> {
        Ok(Connection {
            reader_stream: BufReader::new(tcp_stream.try_clone()?),
            writer_stream: tcp_stream,
            read_buf: String::new(),
        })
    }

    pub fn read_req(&mut self) -> anyhow::Result<hyper::Request<()>> {
        loop {
            let mut buf = String::new();
            let size = self.reader_stream.read_line(&mut buf)?;

            if size == 0 || buf.as_bytes() == b"\r\n" {
                break;
            }
            self.read_buf.push_str(&buf)
        }
        info!("----------read from stream----------");
        info!("parsed http_request: {:?}", &self.read_buf);
        let mut headers = [httparse::EMPTY_HEADER; Self::MAX_HEADER_NUM];
        let mut httparse_req = httparse::Request::new(&mut headers);
        httparse_req
            .parse(self.read_buf.as_bytes())
            .context("request parse failed")?;
        info!(
            "more parsed request:\n{}",
            debug_httparse_request(&httparse_req)
        );

        hyper_request_try_from_httparse(httparse_req, ())
            .context("hyper_request_try_from_httparse failed")
    }

    pub fn write_response(&mut self, response: Response<BodyData>) -> anyhow::Result<()> {
        self.writer_stream
            .write_all(&response.serialize())
            .context("write response to stream")
    }
}

pub fn hyper_request_try_from_httparse<T>(
    parse_req: httparse::Request,
    body: T,
) -> anyhow::Result<hyper::Request<T>> {
    fn version_from_u8(version: u8) -> anyhow::Result<hyper::Version> {
        match version {
            0 => Ok(hyper::Version::HTTP_10),
            1 => Ok(hyper::Version::HTTP_11),
            _ => Err(anyhow!("wrong version")),
        }
    }

    let mut hyper_req = hyper::http::request::Builder::new()
        .method(parse_req.method.unwrap())
        .uri(parse_req.path.unwrap())
        .version(version_from_u8(parse_req.version.unwrap())?);

    let headers = hyper_req
        .headers_mut()
        .context("hyper request failed to build from httparse req")?;
    for parse_header in parse_req
        .headers
        .iter()
        .filter(|header| !header.name.is_empty())
    {
        headers.insert(
            hyper::http::header::HeaderName::from_bytes(parse_header.name.as_bytes())
                .context(format!("parse HeaderName failed: {:?}", parse_header.name))?,
            hyper::http::header::HeaderValue::from_bytes(parse_header.value)?,
        );
    }

    hyper_req
        .body(body)
        .context("hyper request failed to build from httparse req")
}

fn debug_httparse_request(parse_req: &httparse::Request) -> String {
    let headers: Vec<httparse::Header> = parse_req
        .headers
        .iter()
        .filter(|header| !header.name.is_empty())
        .map(httparse::Header::to_owned)
        .collect();
    format!(
        "Request {{ method: {:?} path: {:?} version: {:?}\nheaders: {:?} }}",
        parse_req.method, parse_req.path, parse_req.version, headers
    )
}
