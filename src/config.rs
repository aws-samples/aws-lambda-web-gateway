use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub lambda_function_name: String,
    #[serde(default = "default_lambda_invoke_mode")]
    pub lambda_invoke_mode: LambdaInvokeMode,
    #[serde(default)]
    pub api_keys: HashSet<String>,
    #[serde(default = "default_auth_mode")]
    pub auth_mode: AuthMode,
    #[serde(default)]
    pub addr: String,
}

#[cfg(test)]
mod tests {
    include!("config_tests.rs");
}

fn default_auth_mode() -> AuthMode {
    AuthMode::Open
}

fn default_lambda_invoke_mode() -> LambdaInvokeMode {
    LambdaInvokeMode::Buffered
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMode {
    Open,
    ApiKey,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LambdaInvokeMode {
    Buffered,
    ResponseStream,
}

impl FromStr for AuthMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(AuthMode::Open),
            "apikey" => Ok(AuthMode::ApiKey),
            _ => Err(format!("Invalid AuthMode: {}", s)),
        }
    }
}

impl FromStr for LambdaInvokeMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buffered" => Ok(LambdaInvokeMode::Buffered),
            "responsestream" => Ok(LambdaInvokeMode::ResponseStream),
            _ => Err(format!("Invalid LambdaInvokeMode: {}", s)),
        }
    }
}
