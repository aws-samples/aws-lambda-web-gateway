use super::*;
use serial_test::serial;
use std::collections::HashSet;
use std::env;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_auth_mode_from_str() {
    assert_eq!("open".parse::<AuthMode>().unwrap(), AuthMode::Open);
    assert_eq!("apikey".parse::<AuthMode>().unwrap(), AuthMode::ApiKey);
    assert_eq!("OPEN".parse::<AuthMode>().unwrap(), AuthMode::Open);
    assert_eq!("APIKEY".parse::<AuthMode>().unwrap(), AuthMode::ApiKey);
    assert!("invalid".parse::<AuthMode>().is_err());
}

#[test]
fn test_lambda_invoke_mode_from_str() {
    assert_eq!(
        "buffered".parse::<LambdaInvokeMode>().unwrap(),
        LambdaInvokeMode::Buffered
    );
    assert_eq!(
        "responsestream".parse::<LambdaInvokeMode>().unwrap(),
        LambdaInvokeMode::ResponseStream
    );
    assert_eq!(
        "BUFFERED".parse::<LambdaInvokeMode>().unwrap(),
        LambdaInvokeMode::Buffered
    );
    assert_eq!(
        "RESPONSESTREAM".parse::<LambdaInvokeMode>().unwrap(),
        LambdaInvokeMode::ResponseStream
    );
    assert!("invalid".parse::<LambdaInvokeMode>().is_err());
}

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.lambda_function_name, "");
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::Buffered);
    assert!(config.api_keys.is_empty());
    assert_eq!(config.auth_mode, AuthMode::Open);
    assert_eq!(config.addr, "0.0.0.0:8000");
}

#[test]
#[serial]
#[should_panic(expected = "No lambda_function_name provided")]
fn test_config_panic_on_empty_lambda_function_name() {
    let mut config = Config::default();
    config.apply_env_overrides();
}

#[test]
#[serial]
fn test_config_apply_env_overrides() {
    env::set_var("LAMBDA_FUNCTION_NAME", "test-function");
    env::set_var("LAMBDA_INVOKE_MODE", "responsestream");
    env::set_var("API_KEYS", "key1,key2");
    env::set_var("AUTH_MODE", "apikey");
    env::set_var("ADDR", "127.0.0.1:3000");

    let mut config = Config::default();
    config.apply_env_overrides();

    assert_eq!(config.lambda_function_name, "test-function");
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::ResponseStream);
    assert_eq!(
        config.api_keys,
        vec!["key1", "key2"]
            .into_iter()
            .map(String::from)
            .collect::<HashSet<String>>()
    );
    assert_eq!(config.auth_mode, AuthMode::ApiKey);
    assert_eq!(config.addr, "127.0.0.1:3000");

    // Clean up environment variables
    env::remove_var("LAMBDA_FUNCTION_NAME");
    env::remove_var("LAMBDA_INVOKE_MODE");
    env::remove_var("API_KEYS");
    env::remove_var("AUTH_MODE");
    env::remove_var("ADDR");
}

#[test]
fn test_config_load() {
    let config_content = r#"
lambda_function_name: test-function
lambda_invoke_mode: ResponseStream
api_keys:
  - key1
  - key2
auth_mode: ApiKey
addr: 127.0.0.1:3000
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", config_content).unwrap();

    let config = Config::load_from_file(temp_file.path()).unwrap();

    assert_eq!(config.lambda_function_name, "test-function");
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::ResponseStream);
    assert_eq!(
        config.api_keys,
        vec!["key1", "key2"]
            .into_iter()
            .map(String::from)
            .collect::<HashSet<String>>()
    );
    assert_eq!(config.auth_mode, AuthMode::ApiKey);
    assert_eq!(config.addr, "127.0.0.1:3000");
}

#[test]
#[serial]
fn test_config_load_with_env_override() {
    let config_content = r#"
lambda_function_name: file-function
lambda_invoke_mode: Buffered
api_keys:
  - file-key
auth_mode: Open
addr: 0.0.0.0:8000
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", config_content).unwrap();

    // Test with environment variables set
    env::set_var("LAMBDA_FUNCTION_NAME", "env-function");
    env::set_var("AUTH_MODE", "apikey");
    env::set_var("LAMBDA_INVOKE_MODE", "responsestream");

    let config = Config::load(temp_file.path());

    assert_eq!(config.lambda_function_name, "env-function");
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::ResponseStream);
    assert_eq!(
        config.api_keys,
        vec!["file-key"]
            .into_iter()
            .map(String::from)
            .collect::<HashSet<String>>()
    );
    assert_eq!(config.auth_mode, AuthMode::ApiKey);
    assert_eq!(config.addr, "0.0.0.0:8000");

    // Clean up environment variables
    env::remove_var("LAMBDA_FUNCTION_NAME");
    env::remove_var("AUTH_MODE");
    env::remove_var("LAMBDA_INVOKE_MODE");

    // Test with no environment variables set
    let config = Config::load(temp_file.path());
    assert_eq!(config.lambda_function_name, "file-function");
    assert_eq!(config.auth_mode, AuthMode::Open);
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::Buffered);

    // Test with empty LAMBDA_FUNCTION_NAME
    env::set_var("LAMBDA_FUNCTION_NAME", "");
    assert!(std::panic::catch_unwind(|| Config::load(temp_file.path())).is_err());
    env::remove_var("LAMBDA_FUNCTION_NAME");
}

#[test]
#[serial]
fn test_config_load_invalid_file() {
    env::set_var("LAMBDA_FUNCTION_NAME", "env-function");
    env::set_var("AUTH_MODE", "apikey");
    env::set_var("LAMBDA_INVOKE_MODE", "responsestream");

    let config = Config::load("non_existent_file.yaml");

    assert_eq!(config.lambda_function_name, "env-function");
    assert_eq!(config.auth_mode, AuthMode::ApiKey);
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::ResponseStream);
    assert!(config.api_keys.is_empty());
    assert_eq!(config.addr, "0.0.0.0:8000");

    // Clean up environment variables
    env::remove_var("LAMBDA_FUNCTION_NAME");
    env::remove_var("AUTH_MODE");
    env::remove_var("LAMBDA_INVOKE_MODE");
}

#[test]
#[serial]
fn test_config_load_invalid_yaml() {
    let config_content = "invalid: yaml: content";

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", config_content).unwrap();

    env::set_var("LAMBDA_FUNCTION_NAME", "env-function");
    env::set_var("AUTH_MODE", "apikey");
    env::set_var("LAMBDA_INVOKE_MODE", "responsestream");

    let config = Config::load(temp_file.path());

    assert_eq!(config.lambda_function_name, "env-function");
    assert_eq!(config.auth_mode, AuthMode::ApiKey);
    assert_eq!(config.lambda_invoke_mode, LambdaInvokeMode::ResponseStream);
    assert!(config.api_keys.is_empty());
    assert_eq!(config.addr, "0.0.0.0:8000");

    // Clean up environment variables
    env::remove_var("LAMBDA_FUNCTION_NAME");
    env::remove_var("AUTH_MODE");
    env::remove_var("LAMBDA_INVOKE_MODE");
}

#[test]
#[serial]
fn test_config_load_empty_api_keys() {
    env::set_var("API_KEYS", "");
    env::set_var("LAMBDA_FUNCTION_NAME", "test-function"); // Add this line

    let config = Config::load("non_existent_file.yaml");

    assert!(config.api_keys.is_empty());

    env::remove_var("API_KEYS");
    env::remove_var("LAMBDA_FUNCTION_NAME"); // Add this line
}
