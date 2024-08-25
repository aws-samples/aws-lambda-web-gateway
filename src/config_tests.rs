use crate::config::{AuthMode, LambdaInvokeMode};

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
