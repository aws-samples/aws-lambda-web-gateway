use crate::config::{AuthMode, Config, LambdaInvokeMode};
use std::collections::HashSet;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_from_yaml_file() {
    let config_yaml = r#"
lambda_function_name: test-function
lambda_invoke_mode: ResponseStream
api_keys:
  - key1
  - key2
auth_mode: ApiKey
addr: 127.0.0.1:8080
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_yaml).unwrap();

    let config = Config::from_yaml_file(temp_file.path()).unwrap();

    assert_eq!(config.lambda_function_name, "test-function");
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::ResponseStream);
    assert_eq!(config.api_keys, HashSet::from(["key1".to_string(), "key2".to_string()]));
    assert_eq!(config.auth_mode, AuthMode::ApiKey);
    assert_eq!(config.addr, "127.0.0.1:8080");
}

#[test]
fn test_from_yaml_file_default_values() {
    let config_yaml = r#"
lambda_function_name: test-function
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_yaml).unwrap();

    let config = Config::from_yaml_file(temp_file.path()).unwrap();

    assert_eq!(config.lambda_function_name, "test-function");
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::Buffered);
    assert!(config.api_keys.is_empty());
    assert_eq!(config.auth_mode, AuthMode::Open);
    assert_eq!(config.addr, "");
}

#[test]
fn test_from_yaml_file_missing_required_field() {
    let config_yaml = r#"
api_keys:
  - key1
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_yaml).unwrap();

    let result = Config::from_yaml_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_auth_mode_from_str() {
    assert_eq!("open".parse::<AuthMode>().unwrap(), AuthMode::Open);
    assert_eq!("apikey".parse::<AuthMode>().unwrap(), AuthMode::ApiKey);
    assert!("invalid".parse::<AuthMode>().is_err());
}

#[test]
fn test_lambda_invoke_mode_from_str() {
    assert_eq!("buffered".parse::<LambdaInvokeMode>().unwrap(), LambdaInvokeMode::Buffered);
    assert_eq!("responsestream".parse::<LambdaInvokeMode>().unwrap(), LambdaInvokeMode::ResponseStream);
    assert!("invalid".parse::<LambdaInvokeMode>().is_err());
}
