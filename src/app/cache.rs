use crate::app::response::{empty_response, ok};
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use headers_ext::{CacheControl, HeaderMapExt};
use http::header;
use hyper::{Body, HeaderMap, Response, StatusCode};
use serde_derive::Deserialize;
use std::time::Duration;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct CacheTimeParams {
    n: u64,
}

pub fn cache(state: State) -> (State, Response<Body>) {
    let headers = HeaderMap::borrow_from(&state);
    if headers.get(header::IF_MODIFIED_SINCE).is_some()
        || headers.get(header::IF_NONE_MATCH).is_some()
    {
        let res = empty_response(&state, StatusCode::NOT_MODIFIED);
        (state, res)
    } else {
        ok(state, vec![])
    }
}

pub fn set_cache(state: State) -> (State, Response<Body>) {
    let n = CacheTimeParams::borrow_from(&state).n;

    let mut res = empty_response(&state, StatusCode::OK);
    {
        let headers = res.headers_mut();
        headers.typed_insert(
            CacheControl::new().with_max_age(Duration::from_secs(n)),
        );
    }
    (state, res)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use headers_ext::{CacheControl, HeaderMapExt, IfModifiedSince};
    use http::header;
    use hyper::StatusCode;
    use std::time::Duration;
    use std::time::SystemTime;

    #[test]
    fn test_cache_no_headers() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_cache_if_modified_since() {
        let test_server = TestServer::new(router()).unwrap();
        let header: IfModifiedSince = SystemTime::now().into();

        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .with_header(
                header::IF_MODIFIED_SINCE,
                crate::test::headers::encode(header),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    #[test]
    fn test_cache_if_none_match() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .with_header(
                header::IF_NONE_MATCH,
                header::HeaderValue::from_static("*"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    #[test]
    fn test_set_cache() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache/30")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().typed_get::<CacheControl>().unwrap(),
            CacheControl::new().with_max_age(Duration::from_secs(30))
        )
    }
}
