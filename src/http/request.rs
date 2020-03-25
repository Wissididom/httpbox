use crate::headers::{Header, HeaderMapExt};
use hyper::http::Request as HTTPRequest;
pub use hyper::http::{HeaderMap, StatusCode, Uri};
use std::collections::HashMap;

pub use hyper::{body::Bytes, Body};
use std::net::SocketAddr;

pub struct Request {
    pub req: HTTPRequest<Body>,
    pub client_addr: Option<SocketAddr>,
    params: HashMap<&'static str, String>,
}

impl Request {
    pub fn new(
        req: HTTPRequest<Body>,
        client_addr: Option<SocketAddr>,
        params: Option<HashMap<&'static str, String>>,
    ) -> Self {
        Self {
            req,
            client_addr,
            params: params.unwrap_or_else(HashMap::new),
        }
    }

    pub fn param<T: std::str::FromStr>(&self, key: &'static str) -> Option<T> {
        let str = self.params.get(key)?;
        T::from_str(str).ok()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.req.headers()
    }

    pub fn uri(&self) -> &Uri {
        self.req.uri()
    }

    pub fn body(&mut self) -> Body {
        std::mem::replace(self.req.body_mut(), Body::empty())
    }

    pub fn typed_header<H: Header>(&self) -> Option<H> {
        self.req.headers().typed_get::<H>()
    }

    pub fn query<T: serde::de::DeserializeOwned>(
        &self,
    ) -> std::result::Result<T, serde_urlencoded::de::Error> {
        let query_string = self.req.uri().query().unwrap_or("");
        serde_urlencoded::from_str(query_string)
    }
}