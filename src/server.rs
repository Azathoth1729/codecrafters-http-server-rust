use crate::connection::Connection;
use crate::response::Response;
use hyper::http::response::Builder;
use hyper::{StatusCode, Uri};

pub struct Handler {
    conn: Connection,
}

impl Handler {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn
        }
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        let req = self.conn.read_req()?;

        let response = Self::handle_request(&req)?;

        self.conn.write_response(response)?;

        Ok(())
    }

    fn handle_request<T>(req: &hyper::Request<T>) -> anyhow::Result<Response<()>> {
        match req.uri().path() {
            "/" => Ok(Response::from(
                Builder::new().status(StatusCode::OK).body(())?,
            )),
            _ => Ok(Response::from(
                Builder::new().status(StatusCode::NOT_FOUND).body(())?,
            )),
        }
    }
}
