use std::future::Future;

use crate::response::BodyData;
use hyper::{Request, Response};

/// An asynchronous function from a `Request` to a `Response`.
///
/// The `Service` trait is a simplified interface making it easy to write
/// network applications in a modular and reusable way, decoupled from the
/// underlying protocol.
///
/// # Functional
///
/// A `Service` is a function of a `Request`. It immediately returns a
/// `Future` representing the eventual completion of processing the
/// request. The actual request processing may happen at any time in the
/// future, on any thread or executor. The processing may depend on calling
/// other services. At some point in the future, the processing will complete,
/// and the `Future` will resolve to a response or error.
///
/// At a high level, the `Service::call` function represents an RPC request. The
/// `Service` value can be a server or a client.
pub trait Service<Request> {
    /// Responses given by the service.
    type Response;

    /// The future response value.
    type Future: Future<Output = anyhow::Result<Self::Response>>;

    /// Process the request and return the response asynchronously.
    /// `call` takes `&self` instead of `mut &self` because:
    /// - It prepares the way for async fn,
    ///   since then the future only borrows `&self`, and thus a Service can concurrently handle
    ///   multiple outstanding requests at once.
    /// - It's clearer that Services can likely be cloned
    /// - To share state across clones, you generally need `Arc<Mutex<_>>`
    ///   That means you're not really using the `&mut self` and could do with a `&self`.
    /// The discussion on this is here: <https://github.com/hyperium/hyper/issues/3040>
    fn call(&self, req: &Request) -> Self::Future;
}

/// Create a `Service` from a function.
///
/// # Example
///
/// ```
/// use bytes::Bytes;
/// use hyper::{body, Request, Response, Version};
/// use http_body_util::Full;
/// use hyper::service::service_fn;
///
/// let service = service_fn(|req: Request<body::Incoming>| async move {
///     if req.version() == Version::HTTP_11 {
///         Ok(Response::new(Full::<Bytes>::from("Hello World")))
///     } else {
///         // Note: it's usually better to return a Response
///         // with an appropriate StatusCode instead of an Err.
///         Err("not HTTP/1.1, abort connection")
///     }
/// });
/// ```
///

impl<F, ReqBody, Ret> Service<Request<ReqBody>> for F
where
    F: Fn(&Request<ReqBody>) -> Ret,
    Ret: Future<Output = anyhow::Result<Response<BodyData>>>,
    // E: Into<Box<dyn StdError + Send + Sync>>,
{
    type Response = hyper::Response<BodyData>;
    // type Error = anyhow::Result<Response<BodyData>>;
    type Future = Ret;

    fn call(&self, req: &Request<ReqBody>) -> Self::Future {
        self(req)
    }
}

// pub fn service_fn<F, R, S>(f: F) -> ServiceFn<F, R>
// where
//     F: Fn(Request<R>) -> S,
//     S: Future,
// {
//     ServiceFn {
//         f,
//         _req: PhantomData,
//     }
// }

// /// Service returned by [`service_fn`]
// pub struct ServiceFn<F, R> {
//     pub(crate) f: F,
//     pub(crate) _req: PhantomData<fn(R)>,
// }
//
// impl<F, ReqBody, Ret, E> Service<Request<ReqBody>> for ServiceFn<F, ReqBody>
// where
//     F: Fn(Request<ReqBody>) -> Ret,
//     Ret: Future<Output = Result<Response<BodyData>, E>>,
//     E: Into<Box<dyn StdError + Send + Sync>>,
// {
//     type Response = hyper::Response<BodyData>;
//     type Error = E;
//     type Future = Ret;
//
//     fn call(&self, req: Request<ReqBody>) -> Self::Future {
//         (self.f)(req)
//     }
// }
//
// impl<F, R> fmt::Debug for ServiceFn<F, R> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("impl Service").finish()
//     }
// }
//
// impl<F, R> Clone for ServiceFn<F, R>
// where
//     F: Clone,
// {
//     fn clone(&self) -> Self {
//         ServiceFn {
//             f: self.f.clone(),
//             _req: PhantomData,
//         }
//     }
// }
//
// impl<F, R> Copy for ServiceFn<F, R> where F: Copy {}
