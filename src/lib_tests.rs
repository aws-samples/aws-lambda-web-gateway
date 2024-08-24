use super::*;
use axum::http::StatusCode;
use aws_smithy_types::Blob;
use std::collections::HashMap;
use aws_sdk_lambda::types::InvokeWithResponseStreamResponseEvent;
use aws_sdk_lambda::operation::invoke_with_response_stream::InvokeWithResponseStreamOutput;
use futures_util::stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_util::Stream;

struct MockEventStream {
    events: Vec<Result<InvokeWithResponseStreamResponseEvent, aws_sdk_lambda::Error>>,
}

impl Stream for MockEventStream {
    type Item = Result<InvokeWithResponseStreamResponseEvent, aws_sdk_lambda::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.events.pop())
    }
}

#[tokio::test]
async fn test_health() {
    let response = health().await.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_to_string_map() {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("X-Custom-Header", "test-value".parse().unwrap());

    let result = to_string_map(&headers);

    assert_eq!(result.len(), 2);
    assert_eq!(result.get("content-type"), Some(&"application/json".to_string()));
    assert_eq!(result.get("x-custom-header"), Some(&"test-value".to_string()));
}

#[tokio::test]
async fn test_handle_buffered_response() {
    let lambda_response = LambdaResponse {
        status_code: 200,
        status_description: Some("OK".to_string()),
        is_base64_encoded: Some(false),
        headers: Some(HashMap::from([
            ("Content-Type".to_string(), "text/plain".to_string()),
        ])),
        body: "Hello, World!".to_string(),
    };

    let payload = serde_json::to_vec(&lambda_response).unwrap();
    let invoke_output = aws_sdk_lambda::operation::invoke::InvokeOutput::builder()
        .payload(Blob::new(payload))
        .status_code(200)
        .build();

    let response = handle_buffered_response(invoke_output).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "text/plain"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "Hello, World!");
}

#[tokio::test]
async fn test_detect_metadata() {
    let payload = r#"{"statusCode": 200, "headers": {"Content-Type": "text/plain"}, "body": "Hello"}"#;
    let full_payload = payload.as_bytes().to_vec();
    let chunk = InvokeWithResponseStreamResponseEvent::PayloadChunk(
        aws_sdk_lambda::types::InvokeResponseStreamUpdate::builder()
            .payload(Blob::new(full_payload.clone()))
            .build(),
    );
    let event_stream = MockEventStream {
        events: vec![Ok(chunk)],
    };

    let mut resp = InvokeWithResponseStreamOutput::builder()
        .event_stream(Box::pin(event_stream))
        .build()
        .unwrap();

    let (has_metadata, first_chunk) = detect_metadata(&mut resp).await;

    assert!(has_metadata);
    assert_eq!(first_chunk.unwrap(), full_payload);
}

#[tokio::test]
async fn test_collect_metadata() {
    let payload = r#"{"statusCode": 200, "headers": {"Content-Type": "text/plain"}, "body": "Hello"}"#;
    let null_padding = vec![0u8; 8];
    let remaining_data = b"Remaining data";

    let mut full_payload = payload.as_bytes().to_vec();
    full_payload.extend_from_slice(&null_padding);
    full_payload.extend_from_slice(remaining_data);

    let chunk = InvokeWithResponseStreamResponseEvent::PayloadChunk(
        aws_sdk_lambda::types::InvokeResponseStreamUpdate::builder()
            .payload(Blob::new(full_payload))
            .build(),
    );
    let event_stream = MockEventStream {
        events: vec![Ok(chunk)],
    };

    let mut resp = InvokeWithResponseStreamOutput::builder()
        .event_stream(Box::pin(event_stream))
        .build()
        .unwrap();

    let mut metadata_buffer = Vec::new();
    let (metadata_prelude, remaining) = collect_metadata(&mut resp, &mut metadata_buffer).await;

    assert!(metadata_prelude.is_some());
    let prelude = metadata_prelude.unwrap();
    assert_eq!(prelude.status_code, StatusCode::OK);
    assert_eq!(prelude.headers.get("content-type").unwrap(), "text/plain");
    assert_eq!(remaining, remaining_data);
}

#[tokio::test]
async fn test_process_buffer() {
    let payload = r#"{"statusCode": 200, "headers": {"Content-Type": "text/plain"}, "body": "Hello"}"#;
    let null_padding = vec![0u8; 8];
    let remaining_data = b"Remaining data";

    let mut buffer = payload.as_bytes().to_vec();
    buffer.extend_from_slice(&null_padding);
    buffer.extend_from_slice(remaining_data);

    let (metadata_prelude, remaining) = process_buffer(&buffer);

    assert!(metadata_prelude.is_some());
    let prelude = metadata_prelude.unwrap();
    assert_eq!(prelude.status_code, StatusCode::OK);
    assert_eq!(prelude.headers.get("content-type").unwrap(), "text/plain");
    assert_eq!(remaining, remaining_data);
}
