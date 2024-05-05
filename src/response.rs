use std::fmt;
use std::fmt::{Debug, Formatter};

pub trait InComingBody {}

impl InComingBody for () {}

impl InComingBody for str {}

pub struct Response<T> {
    inner: hyper::Response<T>,
}

impl<T> fmt::Display for Response<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.inner.version().fmt(f)?;
        write!(
            f,
            " {} {}\r\n\r\n",
            self.inner.status().as_str(),
            self.inner.status().canonical_reason().unwrap()
        )
    }
}

impl<T> From<hyper::Response<T>> for Response<T> {
    fn from(value: hyper::Response<T>) -> Self {
        Response { inner: value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::http::response;

    #[test]
    fn response_display_test() -> anyhow::Result<()> {
        assert_eq!(
            Response::from(response::Builder::new().status(200).body(())?)
                .to_string()
                .as_bytes(),
            b"HTTP/1.1 200 OK\r\n\r\n"
        );
        assert_eq!(
            Response::from(response::Builder::new().status(404).body(())?)
                .to_string()
                .as_bytes(),
            b"HTTP/1.1 404 Not Found\r\n\r\n"
        );
        Ok(())
    }
}
