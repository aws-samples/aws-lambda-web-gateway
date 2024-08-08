mod config;
use crate::config::{Config, LambdaInvokeMode};
use std::path::PathBuf;
use std::collections::HashMap;

struct ApplicationState {
    client: aws_sdk_lambda::Client,
    config: Config,
}

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
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config_path = PathBuf::from("config.yaml");
    let config = Config::from_yaml_file(&config_path).expect("Failed to load configuration");
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&aws_config);

    let app_state = ApplicationState {
        client,
        config,
    };

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/", any(handler))
        .route("/*path", any(handler))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on {}", addr);
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
        .and_then(|v| v.to_str().ok())
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

    if !Config::api_keys(config).contains(headers.get("x-api-key").and_then(|v| v.to_str().ok()).unwrap_or_default()) {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap();
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

    let resp = match Config::lambda_invoke_mode(config) {
        LambdaInvokeMode::Buffered => {
            let resp = client
                .invoke()
                .function_name(&Config::lambda_function_name(config))
                .payload(Blob::new(lambda_request_body))
                .send()
                .await
                .unwrap();
            handle_buffered_response(resp).await
        }
        LambdaInvokeMode::ResponseStreaming => {
            let mut resp = client
                .invoke_with_response_stream()
                .function_name(&Config::lambda_function_name(&config))
                .invocation_type(ResponseStreamingInvocationType::RequestResponse)
                .payload(Blob::new(lambda_request_body))
                .send()
                .await
                .unwrap();
            handle_streaming_response(&mut resp).await
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
    headers: HashMap<String, String>,
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

async fn handle_buffered_response(_resp: aws_sdk_lambda::operation::invoke::InvokeOutput) -> Response {
    // Handle buffered response
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

async fn handle_streaming_response(resp: &mut aws_sdk_lambda::operation::invoke_with_response_stream::InvokeWithResponseStreamOutput) -> Response {
    // Handle streaming response
    let mut metadata_prelude_buffer = Vec::new();
    let mut remain_buffer = Vec::new();
    'outer: while let Some(event) = resp.event_stream.recv().await.unwrap() {
        match event {
            PayloadChunk(chunk) => match chunk.payload() {
                None => {}
                Some(data) => {
                    let mut null_count = 0;
                    let bytes = data.clone().into_inner();
                    let bytes_len = bytes.len();
                    for i in 0..bytes_len {
                        if bytes[i] != 0 {
                            metadata_prelude_buffer.push(bytes[i]);
                        } else {
                            null_count += 1;
                            if null_count == 8 {
                                if i != bytes_len {
                                    remain_buffer = bytes[i + 1..bytes_len].to_vec()
                                }
                                break 'outer;
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
    let metadata_prelude_string = String::from_utf8(metadata_prelude_buffer).unwrap();
    let metadata_prelude: MetadataPrelude =
        serde_json::from_str(metadata_prelude_string.as_str()).unwrap_or_default();
    info!(metadata_prelude=?metadata_prelude);

    let (tx, rx) = mpsc::channel(1);

    let resp_clone = resp.as_mut().clone();
    tokio::spawn(async move {
        if remain_buffer.len() != 0 {
            let stream_update = InvokeResponseStreamUpdate::builder()
                .payload(Blob::new(remain_buffer))
                .build();

            let _ = tx.send(PayloadChunk(stream_update)).await;
        }

        while let Some(event) = resp_clone.event_stream().unwrap().recv().await.unwrap() {
            let _ = tx.send(event).await;
        }
    });

    let stream = ReceiverStream::new(rx).map(|event| match event {
        InvokeComplete(_) => Ok(Bytes::default()),
        PayloadChunk(chunk) => match chunk.payload() {
            Some(data) => {
                let bytes = data.clone().into_inner();
                info!(data = ?String::from_utf8_lossy(&*bytes));
                Ok(Bytes::from(bytes))
            }
            None => Ok(Bytes::default()),
        },
        _ => Err("unknown events"),
    });

    let resp_builder = Response::builder().status(metadata_prelude.status_code);

    let resp_builder = metadata_prelude
        .headers
        .iter()
        .filter(|(k, _)| *k != "content-length")
        .fold(resp_builder, |builder, (k, v)| builder.header(k, v));

    let resp_builder = metadata_prelude
        .cookies
        .iter()
        .fold(resp_builder, |builder, cookie| builder.header("set-cookie", cookie));

    resp_builder.body(Body::from_stream(stream)).unwrap()
}
