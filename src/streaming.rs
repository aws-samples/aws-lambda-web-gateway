use crate::utils::handle_err;
use aws_sdk_lambda::{
    operation::invoke_with_response_stream::InvokeWithResponseStreamOutput,
    types::{InvokeResponseStreamUpdate, InvokeWithResponseStreamResponseEvent},
};
use axum::{
    body::Body,
    http::{response::Builder, HeaderMap, StatusCode},
    response::Response,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use InvokeWithResponseStreamResponseEvent::*;

// TODO: contribute to `lambda_runtime` crate to make this struct derive Deserialize
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataPrelude {
    /// The HTTP status code.
    #[serde(with = "http_serde::status_code")]
    pub status_code: StatusCode,
    /// The HTTP headers.
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
    /// The HTTP cookies.
    pub cookies: Vec<String>,
}

pub(super) async fn handle_streaming_response(mut resp: InvokeWithResponseStreamOutput) -> Response {
    // collect metadata
    let (metadata, buffer) = {
        let mut buffer = vec![];
        loop {
            let next = handle_err!("Receiving response stream", resp.event_stream.recv().await);
            if let Some(PayloadChunk(InvokeResponseStreamUpdate {
                payload: Some(data), ..
            })) = next
            {
                buffer.extend_from_slice(&data.into_inner());

                // actually this is only required for the first chunk
                // but this is cheap, so we call it in the loop to simplify the flow
                if !detect_metadata(&buffer) {
                    break (None, buffer);
                }

                if let Some((prelude, remaining)) = try_parse_metadata(&buffer) {
                    break (Some(prelude), remaining.into());
                }
            } else {
                // no more chunks
                break (None, buffer);
            }
        }
    };

    let builder = create_response_builder(metadata);

    // Spawn task to handle remaining stream
    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        // Send remaining data after metadata first
        if !buffer.is_empty() {
            tx.send(Ok(buffer)).await.ok();
        }

        loop {
            match resp.event_stream.recv().await {
                Err(e) => {
                    tx.send(Err(e)).await.ok();
                }
                Ok(e) => {
                    if let Some(update) = e {
                        match update {
                            PayloadChunk(chunk) => {
                                if let Some(data) = chunk.payload {
                                    let bytes = data.into_inner();
                                    if !bytes.is_empty() {
                                        tx.send(Ok(bytes)).await.ok();
                                    }
                                }
                                // else, no data in the chunk, just ignore
                            }
                            InvokeComplete(_) => {
                                break;
                            }
                            _ => {
                                tracing::warn!("Unhandled event type: {:?}", update);
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    });

    handle_err!(
        "Building response",
        builder.body(Body::from_stream(
            ReceiverStream::new(rx).map(|res| res.map(Bytes::from))
        ))
    )
}

#[inline]
fn detect_metadata(bytes: &[u8]) -> bool {
    bytes.first() == Some(&b'{')
}

/// If metadata prelude is found, return the metadata prelude and the remaining data.
fn try_parse_metadata(buffer: &[u8]) -> Option<(MetadataPrelude, &[u8])> {
    let mut null_count = 0;
    for (i, &byte) in buffer.iter().enumerate() {
        if byte != 0 {
            null_count = 0;
            continue;
        }
        null_count += 1;
        if null_count == 8 {
            // now we have 8 continuous null bytes
            let metadata_str = String::from_utf8_lossy(&buffer[..i - 7]);
            // TODO: handle invalid metadata prelude
            let metadata_prelude = serde_json::from_str(&metadata_str).unwrap_or_default();
            tracing::debug!(metadata_prelude=?metadata_prelude);
            return Some((metadata_prelude, &buffer[i + 1..]));
        }
    }
    None
}

fn create_response_builder(metadata: Option<MetadataPrelude>) -> Builder {
    if let Some(metadata) = metadata {
        let mut builder = Response::builder().status(metadata.status_code);

        // apply all headers except content-length
        {
            let headers = builder.headers_mut().unwrap();
            *headers = metadata.headers;
            headers.remove("content-length");
        }

        for cookie in &metadata.cookies {
            builder = builder.header("set-cookie", cookie);
        }

        builder
    } else {
        // Default response if no metadata
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/octet-stream")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_metadata() {
        assert!(detect_metadata(b"{\"statusCode\":200,\"headers\":{}}"));
        assert!(!detect_metadata(b"Hello, world!"));
    }

    #[test]
    fn test_try_parse_metadata() {
        // incomplete
        assert!(try_parse_metadata(b"{\"statusCod").is_none());
        assert!(try_parse_metadata(b"{\"statusCode\":200,\"headers\":{}}\0\0\0").is_none());

        // complete
        let (metadata_prelude, remaining) =
            try_parse_metadata(b"{\"statusCode\":200,\"headers\":{}}\0\0\0\0\0\0\0\0Hello, world!").unwrap();
        assert_eq!(metadata_prelude.status_code, StatusCode::OK);
        assert_eq!(metadata_prelude.headers.len(), 0);
        assert_eq!(remaining, b"Hello, world!");
    }

    #[test]
    fn test_create_response_builder() {
        let metadata = MetadataPrelude {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            headers: {
                let mut headers = HeaderMap::new();
                headers.insert("content-type", "text/plain".parse().unwrap());
                headers.insert("content-length", "0".parse().unwrap());
                headers
            },
            cookies: vec!["cookie1".to_string(), "cookie2".to_string()],
        };
        let builder = create_response_builder(Some(metadata));
        let response = builder.body(Body::empty()).unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/plain");
        assert_eq!(response.headers().get("content-length"), None);
        assert_eq!(
            response.headers().get_all("set-cookie").iter().collect::<Vec<_>>(),
            &["cookie1", "cookie2"]
        );

        let builder = create_response_builder(None);
        let response = builder.body(Body::empty()).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/octet-stream"
        );
        assert_eq!(response.headers().get("content-length"), None);
    }
}
