use crate::app::response::ok;
use failure::{Error, Fallible};
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::state::{FromState, State};
use headers_ext::{ContentType, HeaderMapExt};
use hyper::{Body, HeaderMap, StatusCode};
use url::form_urlencoded;

fn parse_url_encoded_body(raw_body: &[u8]) -> Fallible<String> {
    Ok(form_urlencoded::parse(&raw_body[..])
        .map(|(key, value)| format!("{} = {}", key, value))
        .collect::<Vec<String>>()
        .join("\n"))
}

enum ContentTypeDecoder {
    UrlEncoded,
    Raw,
}

fn content_type_decoder(state: &State) -> ContentTypeDecoder {
    let content_type = HeaderMap::borrow_from(&state)
        .typed_get::<ContentType>()
        .map(mime::Mime::from)
        .unwrap_or(mime::TEXT_PLAIN);

    match (content_type.type_(), content_type.subtype()) {
        (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
            ContentTypeDecoder::UrlEncoded
        }
        _ => ContentTypeDecoder::Raw,
    }
}

pub fn parse_body(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state).concat2().then(|raw_body| {
        let valid_body = future_try_or_error_response!(state, raw_body);
        let content = future_try_or_error_response!(
            StatusCode::BAD_REQUEST,
            state,
            match content_type_decoder(&state) {
                ContentTypeDecoder::UrlEncoded => {
                    parse_url_encoded_body(&valid_body).map_err(|e| e.compat())
                }
                ContentTypeDecoder::Raw => {
                    String::from_utf8(valid_body.to_vec())
                        .map_err(|e| Error::from(e).compat())
                }
            }
        );
        future::ok(ok(state, content))
    });

    Box::new(f)
}

#[cfg(test)]
mod test {
    use super::{
        content_type_decoder, parse_url_encoded_body, ContentTypeDecoder,
    };

    use gotham::state::State;
    use http::header;
    use hyper::HeaderMap;

    #[test]
    fn test_parse_url_encoded_body() {
        assert_eq!(
            parse_url_encoded_body(
                "first=one&second=two&third=three".as_bytes()
            )
            .unwrap(),
            "first = one\nsecond = two\nthird = three"
        )
    }

    #[test]
    fn test_missing_header() {
        State::with_new(|state| {
            state.put(HeaderMap::new());
            match content_type_decoder(&state) {
                ContentTypeDecoder::Raw => (),
                _ => panic!("Incorrect decoder"),
            };
        });
    }

    #[test]
    fn test_form_encoded_header() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED
                    .to_string()
                    .parse()
                    .unwrap(),
            );
            state.put(headers);

            match content_type_decoder(&state) {
                ContentTypeDecoder::UrlEncoded => (),
                _ => panic!("Incorrect decoder"),
            };
        });
    }

    #[test]
    fn test_form_encoded_with_charset_header() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(
                    "application/x-www-form-urlencoded; charset=utf-8",
                ),
            );
            state.put(headers);

            match content_type_decoder(&state) {
                ContentTypeDecoder::UrlEncoded => (),
                _ => panic!("Incorrect decoder"),
            };
        });
    }
}
