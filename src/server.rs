use anyhow::{anyhow, Context};
use bytes::Bytes;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use hyper::http::response::Builder;
use hyper::{Method, StatusCode};
use regex::Regex;
use tokio::net::TcpStream;
use tracing::info;

use crate::connection::Connection;
use crate::response::{BodyData, Response};

pub struct Handler {
    conn: Connection,
}

impl Handler {
    pub fn new(tcp_stream: TcpStream) -> Self {
        Self {
            conn: Connection::new(tcp_stream),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let hyper_req = self.conn.read_req().await?;
        info!("hyper request:\n{:?}", hyper_req);

        let response = Self::handle_request(&hyper_req)?;
        info!("send response:\n{:?}", response);

        self.conn.write_response(response).await?;
        info!("----------write to stream----------");

        Ok(())
    }

    fn handle_request<T>(req: &hyper::Request<T>) -> anyhow::Result<Response<BodyData>> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, path) => match path {
                path if parse_echo_path(path).is_ok() => {
                    let body_bytes = Bytes::from(parse_echo_path(path)?.to_owned());

                    Ok(Response::from(
                        Builder::new()
                            .status(StatusCode::OK)
                            .header(CONTENT_TYPE, "text/plain")
                            .header(CONTENT_LENGTH, body_bytes.len())
                            .body(Some(body_bytes))?,
                    ))
                }
                "/user-agent" => {
                    let body = req
                        .headers()
                        .get(USER_AGENT)
                        .context("req don't have USER_AGENT header key")?;

                    Ok(Response::from(
                        Builder::new()
                            .status(StatusCode::OK)
                            .header(CONTENT_TYPE, "text/plain")
                            .header(CONTENT_LENGTH, body.as_bytes().len())
                            .body(Some(Bytes::from(body.as_bytes().to_owned())))?,
                    ))
                }
                "/" => Ok(Response::from(
                    Builder::new().status(StatusCode::OK).body(None)?,
                )),
                _ => Ok(Response::from(
                    Builder::new().status(StatusCode::NOT_FOUND).body(None)?,
                )),
            },
            _ => Ok(Response::from(
                Builder::new().status(StatusCode::NOT_FOUND).body(None)?,
            )),
        }
    }
}

fn parse_echo_path(path: &str) -> anyhow::Result<&str> {
    let re = Regex::new(r"(?m)^/echo/(.*)$")?;
    let caps = re.captures(path).context("can't capture pattern")?;
    if caps.len() != 2 {
        return Err(anyhow!("caps.len={}", caps.len()));
    }
    Ok(caps.get(1).unwrap().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_echo_path_test() -> anyhow::Result<()> {
        assert_eq!(parse_echo_path("/echo/abc")?, "abc");
        assert_eq!(parse_echo_path("/echo/some/da")?, "some/da");

        assert!(parse_echo_path("/ech").is_err());

        Ok(())
    }
}
