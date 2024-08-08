use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub lambda_function_name: String,
    pub lambda_invoke_mode: LambdaInvokeMode,
    pub api_keys: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LambdaInvokeMode {
    Buffered,
    ResponseStreaming,
}
use std::fs;
use std::path::Path;
use serde_yaml;

impl Config {
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
}
