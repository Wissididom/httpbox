extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate serde;

use crate::app::response::empty_response;
use gotham::state::{FromState, State};

use hyper::{Response, StatusCode};

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct StatusCodeParams {
    code: u16,
}

pub fn status_code(mut state: State) -> (State, Response) {
    let params = StatusCodeParams::take_from(&mut state);

    let status_code =
        try_or_error_response!(state, StatusCode::try_from(params.code));
    let res = empty_response(&state, status_code);
    (state, res)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_status_code() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/status/429")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::TooManyRequests);
    }

    #[test]
    fn test_bad_status_code() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/status/999")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::BadRequest);
    }
}
