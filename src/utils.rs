use axum::http::HeaderMap;
use base64::{prelude::BASE64_STANDARD, Engine};
use bytes::Bytes;

macro_rules! handle_err {
    ($name:expr, $result:expr) => {{
        match $result {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{}: {:?}", $name, e);
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap();
            }
        }
    }};
}
pub(crate) use handle_err;

pub(super) fn whether_should_base64_encode(headers: &HeaderMap) -> bool {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    match content_type {
        "application/json" => false,
        "application/xml" => false,
        "application/javascript" => false,
        _ if content_type.starts_with("text/") => false,
        _ => true,
    }
}

pub(super) fn transform_body(should_base64_encode: bool, body: Bytes) -> String {
    if should_base64_encode {
        BASE64_STANDARD.encode(body)
    } else {
        String::from_utf8_lossy(&body).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whether_base64_encoded() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        assert!(!whether_should_base64_encode(&headers));

        headers.insert("content-type", "application/xml".parse().unwrap());
        assert!(!whether_should_base64_encode(&headers));

        headers.insert("content-type", "application/javascript".parse().unwrap());
        assert!(!whether_should_base64_encode(&headers));

        headers.insert("content-type", "text/html".parse().unwrap());
        assert!(!whether_should_base64_encode(&headers));

        headers.insert("content-type", "image/png".parse().unwrap());
        assert!(whether_should_base64_encode(&headers));
    }

    #[test]
    fn test_transform_body() {
        let body = Bytes::from("Hello, world!");
        assert_eq!(transform_body(false, body.clone()), "Hello, world!");

        let base64_body = Bytes::from(BASE64_STANDARD.encode(&body));
        assert_eq!(transform_body(true, body), base64_body);
    }
}
