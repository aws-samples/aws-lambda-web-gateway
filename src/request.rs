use aws_lambda_events::query_map::QueryMap;
use axum::http::{request::Parts, HeaderMap};
use std::collections::HashMap;

pub(super) fn build_alb_request_body(
    is_base64_encoded: bool,
    query_string_parameters: QueryMap,
    parts: Parts,
    body: String,
) -> Result<String, serde_json::Error> {
    Ok(serde_json::json!({
        "httpMethod": parts.method.to_string(),
        "path": parts.uri.path(),
        "queryStringParameters": query_map_to_hash_map(query_string_parameters),
        "multiValueQueryStringParameters": {},
        "headers": header_map_to_hash_map(parts.headers),
        "multiValueHeaders": {},
        "requestContext": {
            "elb": {
                "targetGroupArn": Option::<String>::None
            }
        },
        "isBase64Encoded": is_base64_encoded,
        "body": body,
    })
    .to_string())
    // serde_json::to_string(&AlbTargetGroupRequest {
    //     http_method: parts.method,
    //     headers: parts.headers,
    //     path: parts.uri.path().to_string().into(),
    //     query_string_parameters,
    //     body: body.into(),
    //     is_base64_encoded,
    //     request_context: AlbTargetGroupRequestContext {
    //         elb: ElbContext { target_group_arn: None },
    //     },
    //     // TODO: support multi-value-header mode?
    //     multi_value_headers: Default::default(),
    //     multi_value_query_string_parameters: Default::default(),
    // })
}

// TODO: remove this after https://github.com/awslabs/aws-lambda-rust-runtime/pull/955 is merged
fn query_map_to_hash_map(map: QueryMap) -> HashMap<String, String> {
    map.iter()
        .map(|(k, _)| {
            let values = map.all(k).unwrap();
            (k.to_string(), values.iter().last().unwrap().to_string())
        })
        .collect()
}

// TODO: remove this after https://github.com/awslabs/aws-lambda-rust-runtime/pull/955 is merged
fn header_map_to_hash_map(map: HeaderMap) -> HashMap<String, String> {
    map.iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{request::Builder, Method};
    use base64::{prelude::BASE64_STANDARD, Engine};
    use std::collections::HashMap;

    #[test]
    fn test_alb_body() {
        let (parts, body) = Builder::new()
            .method(Method::GET)
            .uri("https://example.com/?k=v")
            .header("key", "value")
            .body("Hello, world!")
            .unwrap()
            .into_parts();
        let query = HashMap::from([("k".to_string(), "v".to_string())]).into();

        let expected = "{\"body\":\"Hello, world!\",\"headers\":{\"key\":\"value\"},\"httpMethod\":\"GET\",\"isBase64Encoded\":false,\"multiValueHeaders\":{},\"multiValueQueryStringParameters\":{},\"path\":\"/\",\"queryStringParameters\":{\"k\":\"v\"},\"requestContext\":{\"elb\":{\"targetGroupArn\":null}}}";
        assert_eq!(
            build_alb_request_body(false, query, parts, body.into()).unwrap(),
            expected
        );
    }

    #[test]
    fn test_alb_body_base64() {
        let (parts, body) = Builder::new()
            .method(Method::GET)
            .uri("https://example.com/?k=v")
            .header("key", "value")
            .body(BASE64_STANDARD.encode("Hello, world!"))
            .unwrap()
            .into_parts();
        let query = HashMap::from([("k".to_string(), "v".to_string())]).into();

        let expected = "{\"body\":\"SGVsbG8sIHdvcmxkIQ==\",\"headers\":{\"key\":\"value\"},\"httpMethod\":\"GET\",\"isBase64Encoded\":true,\"multiValueHeaders\":{},\"multiValueQueryStringParameters\":{},\"path\":\"/\",\"queryStringParameters\":{\"k\":\"v\"},\"requestContext\":{\"elb\":{\"targetGroupArn\":null}}}";
        assert_eq!(build_alb_request_body(true, query, parts, body).unwrap(), expected);
    }
}
