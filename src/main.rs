mod config;
use crate::config::{Config, LambdaInvokeMode};
use aws_config::BehaviorVersion;
use aws_sdk_lambda::types::InvokeWithResponseStreamResponseEvent::{InvokeComplete, PayloadChunk};
use aws_sdk_lambda::types::{InvokeResponseStreamUpdate, ResponseStreamingInvocationType};
use aws_sdk_lambda::Client;
use aws_smithy_types::Blob;
use axum::body::Body;
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    routing::get,
    Router,
};
use base64::Engine;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct ApplicationState {
    client: Client,
    config: Config,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load().expect("Failed to load configuration");
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&aws_config);

    let app_state = ApplicationState { client, config };

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/", any(handler))
        .route("/*path", any(handler))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}

async fn handler(
    path: Option<Path<String>>,
    Query(query_string_parameters): Query<HashMap<String, String>>,
    State(state): State<ApplicationState>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let client = &state.client;
    let config = &state.config;
    let path = "/".to_string() + path.map(|p| p.0).unwrap_or_default().as_str();

    let http_method = method.to_string();

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok().map(Some).flatten())
        .unwrap_or_default();

    let is_base64_encoded = match content_type {
        "application/json" => false,
        "application/xml" => false,
        "application/javascript" => false,
        _ if content_type.starts_with("text/") => false,
        _ => true,
    };

    let body = if is_base64_encoded {
        base64::engine::general_purpose::STANDARD.encode(body)
    } else {
        String::from_utf8_lossy(&body).to_string()
    };

    match config.auth_mode {
        config::AuthMode::Open => {}
        config::AuthMode::ApiKey => {
            let api_key = headers
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .or_else(|| {
                    headers
                        .get("authorization")
                        .and_then(|v| v.to_str().ok().and_then(|s| s.strip_prefix("Bearer ")))
                })
                .unwrap_or_default();

            if !config.api_keys.contains(api_key) {
                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap();
            }
        }
    }

    let lambda_request_body = json!({
        "httpMethod": http_method,
        "headers": to_string_map(&headers),
        "path": path,
        "queryStringParameters": query_string_parameters,
        "isBase64Encoded": is_base64_encoded,
        "body": body,
        "requestContext": {
            "elb": {
                "targetGroupArn": "",
            },
        },
    })
    .to_string();

    let resp = match config.lambda_invoke_mode {
        LambdaInvokeMode::Buffered => {
            let resp = client
                .invoke()
                .function_name(config.lambda_function_name.as_str())
                .payload(Blob::new(lambda_request_body))
                .send()
                .await
                .unwrap();
            handle_buffered_response(resp).await
        }
        LambdaInvokeMode::ResponseStream => {
            let resp = client
                .invoke_with_response_stream()
                .function_name(config.lambda_function_name.as_str())
                .invocation_type(ResponseStreamingInvocationType::RequestResponse)
                .payload(Blob::new(lambda_request_body))
                .send()
                .await
                .unwrap();
            handle_streaming_response(resp).await
        }
    };

    resp
}

fn to_string_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_owned(),
                String::from_utf8_lossy(v.as_bytes()).into_owned(),
            )
        })
        .collect()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LambdaResponse {
    status_code: u16,
    status_description: Option<String>,
    is_base64_encoded: Option<bool>,
    headers: Option<HashMap<String, String>>,
    body: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataPrelude {
    #[serde(with = "http_serde::status_code")]
    /// The HTTP status code.
    pub status_code: StatusCode,
    #[serde(with = "http_serde::header_map")]
    /// The HTTP headers.
    pub headers: HeaderMap,
    /// The HTTP cookies.
    pub cookies: Vec<String>,
}

async fn handle_buffered_response(resp: aws_sdk_lambda::operation::invoke::InvokeOutput) -> Response {
    // Parse the InvokeOutput payload to extract the LambdaResponse
    let payload = resp.payload().unwrap().as_ref().to_vec();
    tracing::debug!(payload=?String::from_utf8(payload.clone()));
    let lambda_response: LambdaResponse = serde_json::from_slice(&payload).unwrap();

    // Build the response using the extracted information
    let mut resp_builder = Response::builder().status(StatusCode::from_u16(lambda_response.status_code).unwrap());

    if let Some(headers) = lambda_response.headers {
        for (key, value) in headers {
            resp_builder = resp_builder.header(key, value);
        }
    }

    let body = if lambda_response.is_base64_encoded.unwrap_or(false) {
        base64::engine::general_purpose::STANDARD.decode(lambda_response.body).unwrap()
    } else {
        lambda_response.body.into_bytes()
    };
    resp_builder.body(Body::from(body)).unwrap()
}

async fn handle_streaming_response(
    mut resp: aws_sdk_lambda::operation::invoke_with_response_stream::InvokeWithResponseStreamOutput,
) -> Response {
    let (tx, rx) = mpsc::channel(1);
    let mut metadata_buffer = Vec::new();
    let mut in_metadata = false;
    let mut null_count = 0;
    let mut metadata_prelude: Option<MetadataPrelude> = None;
    let mut remaining_data = Vec::new();

    // Process metadata
    while let Some(event) = resp.event_stream.recv().await.unwrap() {
        match event {
            PayloadChunk(chunk) => {
                if let Some(data) = chunk.payload() {
                    let bytes = data.clone().into_inner();
                    for (i, &byte) in bytes.iter().enumerate() {
                        if !in_metadata && byte == b'{' {
                            in_metadata = true;
                        }
                        if in_metadata {
                            metadata_buffer.push(byte);
                            if byte == 0 {
                                null_count += 1;
                                if null_count == 8 {
                                    let metadata_str = String::from_utf8_lossy(&metadata_buffer[..metadata_buffer.len() - 8]);
                                    metadata_prelude = Some(serde_json::from_str(&metadata_str).unwrap_or_default());
                                    tracing::debug!(metadata_prelude=?metadata_prelude);
                                    // Save remaining data after metadata
                                    remaining_data = bytes[i+1..].to_vec();
                                    break;
                                }
                            } else {
                                null_count = 0;
                            }
                        }
                    }
                    if metadata_prelude.is_some() {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    // Spawn task to handle remaining stream
    tokio::spawn(async move {
        // Send remaining data after metadata first
        if !remaining_data.is_empty() {
            let stream_update = InvokeResponseStreamUpdate::builder()
                .payload(Blob::new(remaining_data))
                .build();
            let _ = tx.send(PayloadChunk(stream_update)).await;
        }

        while let Some(event) = resp.event_stream.recv().await.unwrap() {
            match event {
                PayloadChunk(chunk) => {
                    if let Some(data) = chunk.payload() {
                        let stream_update = InvokeResponseStreamUpdate::builder()
                            .payload(data.clone())
                            .build();
                        let _ = tx.send(PayloadChunk(stream_update)).await;
                    }
                }
                InvokeComplete(_) => {
                    let _ = tx.send(event).await;
                }
                _ => {}
            }
        }
    });

    let stream = ReceiverStream::new(rx).map(|event| match event {
        InvokeComplete(_) => Ok(Bytes::default()),
        PayloadChunk(chunk) => match chunk.payload() {
            Some(data) => {
                let bytes = data.clone().into_inner();
                tracing::debug!(data = ?String::from_utf8_lossy(&*bytes));
                Ok(Bytes::from(bytes))
            }
            None => Ok(Bytes::default()),
        },
        _ => Err("unknown events"),
    });

    let metadata_prelude = metadata_prelude.unwrap_or_default();
    let mut resp_builder = Response::builder().status(metadata_prelude.status_code);

    for (k, v) in metadata_prelude.headers.iter() {
        if k != "content-length" {
            resp_builder = resp_builder.header(k, v);
        }
    }

    for cookie in &metadata_prelude.cookies {
        resp_builder = resp_builder.header("set-cookie", cookie);
    }

    resp_builder.body(Body::from_stream(stream)).unwrap()
}
