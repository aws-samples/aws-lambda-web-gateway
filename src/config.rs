use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
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

impl Config {
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_content)?;
        
        if config.lambda_function_name.is_empty() {
            return Err("lambda_function_name is required in the config file".into());
        }
        
        Ok(config)
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = std::path::PathBuf::from("config.yaml");
        Self::from_yaml_file(&config_path)
    }
}
