mod auth;
mod buffered;
mod config;
mod request;
mod streaming;
mod utils;

pub use config::*;

use auth::is_authorized;
use aws_lambda_events::query_map::QueryMap;
use aws_sdk_lambda::Client;
use aws_smithy_types::Blob;
use axum::{
    body::{Body, Bytes},
    extract::{Query, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use buffered::handle_buffered_response;
use request::build_alb_request_body;
use std::sync::Arc;
use streaming::handle_streaming_response;
use utils::{handle_err, transform_body, whether_should_base64_encode};

#[derive(Clone)]
pub struct ApplicationState {
    pub client: Client,
    pub config: Arc<Config>,
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn invoke_lambda(
    State(state): State<ApplicationState>,
    Query(query): Query<QueryMap>,
    parts: Parts,
    body: Bytes,
) -> Response {
    if !is_authorized(&parts.headers, &state.config) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let should_base64_encode = whether_should_base64_encode(&parts.headers);
    let body = transform_body(should_base64_encode, body);

    let lambda_request_body = handle_err!(
        "Building lambda request",
        build_alb_request_body(should_base64_encode, query, parts, body)
    );

    macro_rules! call_lambda {
        ($action:ident) => {
            handle_err!(
                "Invoking lambda",
                state
                    .client
                    .$action()
                    .function_name(state.config.lambda_function_name.as_str())
                    .payload(Blob::new(lambda_request_body))
                    .send()
                    .await
            )
        };
    }

    match state.config.lambda_invoke_mode {
        LambdaInvokeMode::Buffered => handle_buffered_response(call_lambda!(invoke)),
        LambdaInvokeMode::ResponseStream => handle_streaming_response(call_lambda!(invoke_with_response_stream)).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health() {
        let response = health().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
