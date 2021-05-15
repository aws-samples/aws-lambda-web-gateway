use std::{collections::HashMap, env, str};

use actix_web::dev::HttpResponseBuilder;
use actix_web::{
    http, http::HeaderMap, http::StatusCode, middleware, web, App, Error, HttpRequest,
    HttpResponse, HttpServer,
};
use bytes::Bytes;
use env_logger::Env;
use futures::StreamExt;
use log::*;
use rusoto_core::Region;
use rusoto_lambda::{InvocationRequest, Lambda, LambdaClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::form_urlencoded;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = LambdaClient::new(Region::default());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(client.clone())
            .route("/healthz", web::get().to(|| HttpResponse::Ok()))
            .route("/*", web::to(handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn handler(req: HttpRequest, mut payload: web::Payload, client: web::Data<LambdaClient>) -> Result<HttpResponse, Error> {
    // extract request data
    let http_method = req.method();
    let headers = req.headers();
    let path = req.path().to_string();
    let query_string_parameters: HashMap<String, String> =
        form_urlencoded::parse(req.query_string().as_bytes())
            .into_owned()
            .collect();

    // get Content-Type
    let content_type;
    if let Some(value) = headers.get(http::header::CONTENT_TYPE) {
        content_type = value.to_str().unwrap_or_default();
    } else {
        content_type = "application/json";
    }

    // determine if the body should be based64 encoded
    let is_base64_encoded;
    match content_type {
        "application/json" => is_base64_encoded = false,
        "application/xml" => is_base64_encoded = false,
        "application/javascript" => is_base64_encoded = false,
        _ if content_type.starts_with("text/") => is_base64_encoded = false,
        _ => is_base64_encoded = true,
    }

    // retrieve the request body as byte stream
    let mut bytes = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        let item = item?;
        bytes.extend_from_slice(&item);
    }

    // convert the byte stream to String and base64 encoded if required
    let body;
    if is_base64_encoded {
        body = base64::encode(bytes);
    } else {
        body = str::from_utf8(&*bytes).unwrap_or_default().to_string();
    }

    // Create an event which is the same as if it comes from ALB
    let lambda_request_body = json!({
        "httpMethod": http_method.to_string(),
        "headers": to_string_map(headers),
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

    debug!("lambda_request_body = {:#?}", lambda_request_body);

    let request = InvocationRequest {
        function_name: env::var("LAMBDA_FUNCTION_NAME").unwrap(),
        payload: Some(Bytes::from(lambda_request_body.to_owned())),
        ..Default::default()
    };

    match client.invoke(request).await {
        Ok(result) => {
            debug!("invocation response = {:#?}", result);
            let lambda_response: LambdaResponse =
                serde_json::from_slice(result.payload.unwrap().as_ref()).unwrap();
            debug!("lambda_response = {:#?}", lambda_response);

            let mut http_response = HttpResponseBuilder::new(
                StatusCode::from_u16(lambda_response.status_code).unwrap(),
            )
            .body(lambda_response.body);

            lambda_response.headers.iter().map(|(k, v)| {
                http_response
                    .headers_mut()
                    .insert(k.parse().unwrap(), v.parse().unwrap())
            }).for_each(drop);

            debug!("http_response = {:#?}", http_response);

            Ok(http_response)
        }
        Err(error) => {
            debug!("invocation error = {:#?}", error);
            Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
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
