use serde::{Deserialize, Serialize};
use std::{collections::HashSet, env, fs, path::Path, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub lambda_function_name: String,
    #[serde(default)]
    pub lambda_invoke_mode: LambdaInvokeMode,
    #[serde(default)]
    pub api_keys: HashSet<String>,
    #[serde(default)]
    pub auth_mode: AuthMode,
    #[serde(default = "default_addr")]
    pub addr: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lambda_function_name: String::new(),
            lambda_invoke_mode: Default::default(),
            api_keys: HashSet::new(),
            auth_mode: Default::default(),
            addr: default_addr(),
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let mut config = Self::load_from_file(path).unwrap_or_else(|e| {
            tracing::warn!("Failed to load config from file: {}. Using default values.", e);
            Config::default()
        });
        config.apply_env_overrides();
        config
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = env::var("LAMBDA_FUNCTION_NAME") {
            self.lambda_function_name = val;
        }
        if self.lambda_function_name.is_empty() {
            panic!("No lambda_function_name provided. Please set it in the config file or LAMBDA_FUNCTION_NAME environment variable.");
        }
        if let Ok(val) = env::var("LAMBDA_INVOKE_MODE") {
            if let Ok(mode) = val.parse() {
                self.lambda_invoke_mode = mode;
            }
        }
        if let Ok(val) = env::var("API_KEYS") {
            self.api_keys = val.split(',').filter(|s| !s.is_empty()).map(String::from).collect();
        }
        if let Ok(val) = env::var("AUTH_MODE") {
            if let Ok(mode) = val.parse() {
                self.auth_mode = mode;
            }
        }
        if let Ok(val) = env::var("ADDR") {
            self.addr = val;
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

fn default_addr() -> String {
    "0.0.0.0:8000".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AuthMode {
    #[default]
    Open,
    ApiKey,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LambdaInvokeMode {
    #[default]
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

#[cfg(test)]
mod tests;
