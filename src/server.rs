use anyhow::{anyhow, Context};
use bytes::Bytes;
use hyper::{
    header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
    http::response::Builder,
    Method, Request, StatusCode,
};
use nom::AsBytes;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::pin::pin;
use std::{fmt::Debug, future::Future};
use tokio::net::TcpStream;
use tracing::info;

use crate::common::Serializable;
use crate::connection::Connection;
use crate::response::{BodyData, Response};
use crate::service::Service;

type ServeFn<ReqBody, State, ResBody> =
    fn(&hyper::Request<ReqBody>, State) -> anyhow::Result<ResBody>;

// type Service<R, S> = dyn Fn(&hyper::Request<R>) -> anyhow::Result<S>;
trait PathFilter = Fn(&str) -> bool;

pub struct Handler {
    conn: Connection,
}

// pub struct Router<F, R, S> {
//     filter: ServiceFn<F, R>,
//     _req: PhantomData<fn(R)>,
//     _ret: PhantomData<S>,
// }
//
// impl<F, R, S> Router<F, R, S>
// where
//     F: Fn(hyper::Request<R>) -> S,
//     S: Future,
// {
//     fn route(self, path_filter: impl PathFilter, serve_fn: ServiceFn<F, R>) -> Self {
//         let filter_fn = self.filter.f;
//         let closure: fn(Request<R>) -> S = |req: hyper::Request<R>| filter_fn(req);
//         Router {
//             filter: service_fn::<F, R, S>(closure),
//
//             _req: PhantomData,
//             _ret: PhantomData,
//         }
//     }
// }

// fn compose_two_service<ReqBody, S>(
//     path_filter: impl Fn(&str) -> bool,
//     service_1: S,
//     service_2: S,
// ) -> impl Service<Request<ReqBody>>
// where
//     S: Service<Request<ReqBody>, Response = hyper::Response<BodyData>>,
// {
//     async move |req| {
//         let response: anyhow::Result<hyper::Response<BodyData>> = service_1.call(req).await;
//         if response.is_ok() {
//             async { response }
//         } else if path_filter(req.uri().path()) {
//             service_2.call(req)
//         } else {
//             // let fut = ;
//             // fut
//             // async { Err(anyhow!("error")) }
//         }
//     }
// }

impl Handler {
    pub fn new(tcp_stream: TcpStream) -> Self {
        Self {
            conn: Connection::new(tcp_stream),
        }
    }

    pub async fn run<State, ResBody>(
        &mut self,
        server_fn: ServeFn<(), State, ResBody>,
        state: State,
    ) -> anyhow::Result<()>
    where
        ResBody: Serializable + Debug,
    {
        let hyper_req = self.conn.read_req().await?;
        info!("hyper request:\n{:?}", hyper_req);

        let response = server_fn(&hyper_req, state)?;
        info!("send response:\n{:?}", response);

        self.conn.write_response(response).await?;
        info!("----------write to stream----------");

        Ok(())
    }

    pub fn handle_request<T>(
        req: &hyper::Request<T>,
        directory: Option<PathBuf>,
    ) -> anyhow::Result<Response<BodyData>> {
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
                path if parse_file_path(path).is_ok() => {
                    let file_path = parse_file_path(path)?;
                    let file_abs_path = directory
                        .map(|dir| dir.join(file_path))
                        .context("need to provide directory option")?;
                    if let Ok(mut f) = File::open(file_abs_path) {
                        let mut data = vec![];
                        f.read_to_end(&mut data)?;
                        Ok(Response::from(
                            Builder::new()
                                .status(StatusCode::OK)
                                .header(CONTENT_TYPE, "application/octet-stream")
                                .header(CONTENT_LENGTH, data.len())
                                .body(Some(data.into()))?,
                        ))
                    } else {
                        Ok(Response::from(
                            Builder::new().status(StatusCode::NOT_FOUND).body(None)?,
                        ))
                    }
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

fn parse_file_path(path: &str) -> anyhow::Result<&str> {
    let re = Regex::new(r"(?m)^/files/(.*)$")?;
    let caps = re.captures(path).context("can't capture pattern")?;
    if caps.len() != 2 {
        return Err(anyhow!("caps.len expected 2, but got {}", caps.len()));
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
