use crate::response::Response;
use anyhow::{anyhow, Context};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub struct Connection {
    reader_stream: BufReader<TcpStream>,
    writer_stream: TcpStream,
    read_buf: String,
}

pub fn hyper_request_try_from_httparse<T>(
    parse_req: httparse::Request<'_, '_>,
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
    for parse_header in parse_req.headers.iter().filter(|header| !header.name.is_empty()) {
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
        eprintln!("read_buf:\n{:?}", self.read_buf);

        let mut headers = [httparse::EMPTY_HEADER; Self::MAX_HEADER_NUM];
        let mut httparse_req = httparse::Request::new(&mut headers);
        httparse_req
            .parse(&self.read_buf.as_bytes())
            .context("request parse failed")?;
        eprintln!("httparse_req.headers:\n{:?}", httparse_req.headers);

        hyper_request_try_from_httparse(httparse_req, ())
            .context("hyper_request_try_from_httparse failed")
    }

    pub fn write_response<T>(&mut self, response: Response<T>) -> anyhow::Result<()> {
        self.writer_stream
            .write_all(response.to_string().as_bytes())
            .context("write response to stream")
    }
}
