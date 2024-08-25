use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub lambda_function_name: String,
    #[serde(default = "default_lambda_invoke_mode")]
    pub lambda_invoke_mode: LambdaInvokeMode,
    #[serde(default)]
    pub api_keys: HashSet<String>,
    #[serde(default = "default_auth_mode")]
    pub auth_mode: AuthMode,
    #[serde(default = "default_addr")]
    pub addr: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lambda_function_name: String::new(),
            lambda_invoke_mode: default_lambda_invoke_mode(),
            api_keys: HashSet::new(),
            auth_mode: default_auth_mode(),
            addr: default_addr(),
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::load_from_file(path)?;
        config.apply_env_overrides();
        Ok(config)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
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

fn default_addr() -> String {
    "0.0.0.0:8000".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMode {
    Open,
    ApiKey,
}

impl Default for AuthMode {
    fn default() -> Self {
        AuthMode::Open
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LambdaInvokeMode {
    Buffered,
    ResponseStream,
}

impl Default for LambdaInvokeMode {
    fn default() -> Self {
        LambdaInvokeMode::Buffered
    }
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
