use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&contents)?;
        config.override_from_env();
        Ok(config)
    }

    fn override_from_env(&mut self) {
        if let Ok(val) = std::env::var("LAMBDA_FUNCTION_NAME") {
            self.lambda_function_name = val;
        }
        if let Ok(val) = std::env::var("LAMBDA_INVOKE_MODE") {
            if let Ok(mode) = val.parse() {
                self.lambda_invoke_mode = mode;
            }
        }
        if let Ok(val) = std::env::var("API_KEYS") {
            self.api_keys = val.split(',').map(String::from).collect();
        }
        if let Ok(val) = std::env::var("AUTH_MODE") {
            if let Ok(mode) = val.parse() {
                self.auth_mode = mode;
            }
        }
        if let Ok(val) = std::env::var("ADDR") {
            self.addr = val;
        }
    }
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
