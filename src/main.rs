use aws_config::BehaviorVersion;
use aws_sdk_lambda::Client;
use axum::{
    routing::{any, get},
    Router,
};
use lambda_web_gateway::{health, invoke_lambda, ApplicationState, Config};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = {
        let config = Arc::new(Config::load("config.yaml"));
        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = Client::new(&aws_config);
        ApplicationState { client, config }
    };

    let addr = state.config.addr.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Listening on {}", addr);

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/", any(invoke_lambda))
        .route("/*path", any(invoke_lambda))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    axum::serve(listener, app).await.unwrap();
}
