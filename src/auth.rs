use crate::config::{AuthMode, Config};
use axum::http::HeaderMap;

pub(super) fn is_authorized(headers: &HeaderMap, config: &Config) -> bool {
    match config.auth_mode {
        AuthMode::Open => true,
        AuthMode::ApiKey => {
            let api_key = headers
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .or_else(|| {
                    headers
                        .get("authorization")
                        .and_then(|v| v.to_str().ok().and_then(|s| s.strip_prefix("Bearer ")))
                })
                .unwrap_or_default();

            config.api_keys.contains(api_key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_open_auth() {
        let headers = HeaderMap::new();
        let config = Config {
            auth_mode: AuthMode::Open,
            ..Default::default()
        };
        assert!(is_authorized(&headers, &config));
    }

    #[test]
    fn test_api_key_auth() {
        let config = Config {
            auth_mode: AuthMode::ApiKey,
            api_keys: HashSet::from(["test".to_string()]),
            ..Default::default()
        };

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "test".parse().unwrap());
        assert!(is_authorized(&headers, &config));

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "invalid".parse().unwrap());
        assert!(!is_authorized(&headers, &config));

        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test".parse().unwrap());
        assert!(is_authorized(&headers, &config));

        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer invalid".parse().unwrap());
        assert!(!is_authorized(&headers, &config));
    }

    #[test]
    fn test_multi_api_keys() {
        let config = Config {
            auth_mode: AuthMode::ApiKey,
            api_keys: HashSet::from(["test1".to_string(), "test2".to_string()]),
            ..Default::default()
        };

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "test1".parse().unwrap());
        assert!(is_authorized(&headers, &config));

        headers.insert("x-api-key", "test2".parse().unwrap());
        assert!(is_authorized(&headers, &config));

        headers.insert("x-api-key", "invalid".parse().unwrap());
        assert!(!is_authorized(&headers, &config));
    }
}
