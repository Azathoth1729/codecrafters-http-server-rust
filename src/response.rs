use std::fmt::{Debug};

use bytes::Bytes;
use format_bytes::format_bytes;


pub type BodyData = Option<Bytes>;

#[derive(Debug)]
pub struct Response<T> {
    inner: hyper::Response<T>,
}

impl<T> Response<T> {
    pub fn serialize_without_body(&self) -> Vec<u8> {
        let header_str = self
            .inner
            .headers()
            .iter()
            .flat_map(|(name, value)| {
                format_bytes!(
                    b"{}: {}\r\n",
                    name.to_string().as_bytes().to_vec(),
                    value.as_bytes().to_vec()
                )
            })
            .collect::<Vec<u8>>();
        let start_line = format!(
            "{:?} {} {}\r\n",
            self.inner.version(),
            self.inner.status().as_str(),
            self.inner.status().canonical_reason().unwrap()
        );
        format_bytes!(b"{}{}\r\n", start_line.as_bytes(), header_str)
    }
}

impl Response<()> {
    pub fn serialize(&self) -> Vec<u8> {
        self.serialize_without_body()
    }
}

impl Response<BodyData> {
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialize_str_without_body = self.serialize_without_body();
        let body = self.inner.body();
        let body_str = if let Some(bytes) = body {
            std::str::from_utf8(bytes).unwrap()
        } else {
            ""
        };
        serialize_str_without_body.extend(body_str.as_bytes());
        serialize_str_without_body
    }
}

// impl<T> fmt::Display for Response<T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{:?} {} {}\r\n\r\n",
//             self.inner.version(),
//             self.inner.status().as_str(),
//             self.inner.status().canonical_reason().unwrap()
//         )
//     }
// }

impl<T> From<hyper::Response<T>> for Response<T> {
    fn from(value: hyper::Response<T>) -> Self {
        Response { inner: value }
    }
}

#[cfg(test)]
mod tests {
    use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
    use hyper::http::response;
    use hyper::StatusCode;

    use super::*;

    #[test]
    fn response_serialize_test() -> anyhow::Result<()> {
        assert_eq!(
            Response::from(response::Builder::new().status(StatusCode::OK).body(())?).serialize(),
            b"HTTP/1.1 200 OK\r\n\r\n"
        );
        assert_eq!(
            Response::from(
                response::Builder::new()
                    .status(StatusCode::NOT_FOUND)
                    .body(())?
            )
            .serialize(),
            b"HTTP/1.1 404 Not Found\r\n\r\n"
        );
        assert_eq!(
            Response::from(
                response::Builder::new()
                    .status(StatusCode::NOT_FOUND)
                    .body(Some("abc".into()))?
            )
            .serialize(),
            b"HTTP/1.1 404 Not Found\r\n\r\nabc"
        );
        Ok(())
    }

    #[test]
    fn response_serialize_test2() -> anyhow::Result<()> {
        let body_bytes = Bytes::from("abc");
        let response = Response::from(
            response::Builder::new()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/plain")
                .header(CONTENT_LENGTH, body_bytes.len())
                .body(Some(body_bytes))?,
        );
        let expected =
            b"HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ncontent-length: 3\r\n\r\nabc";
        assert_eq!(response.serialize().as_bytes(), expected);
        Ok(())
    }
}
