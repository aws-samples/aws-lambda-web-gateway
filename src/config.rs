use clap::{value_parser, Arg, Command};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub lambda_function_name: String,
    pub lambda_invoke_mode: LambdaInvokeMode,
    pub api_keys: HashSet<String>,
    #[serde(default = "default_auth_mode")]
    pub auth_mode: AuthMode,
    pub addr: String,
}

fn default_auth_mode() -> AuthMode {
    AuthMode::Open
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMode {
    Open,
    ApiKey,
}

impl FromStr for AuthMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(AuthMode::Open),
            "apikey" => Ok(AuthMode::ApiKey),
            _ => Err(format!("Invalid auth mode: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LambdaInvokeMode {
    Buffered,
    ResponseStream,
}

impl FromStr for LambdaInvokeMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buffered" => Ok(LambdaInvokeMode::Buffered),
            "responsestream" => Ok(LambdaInvokeMode::ResponseStream),
            _ => Err(format!("Invalid invoke mode: {}", s)),
        }
    }
}

impl Config {
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&config_content)?;
        
        if config.lambda_function_name.is_empty() {
            return Err("lambda_function_name is required in the config file".into());
        }
        
        // Set default value for lambda_invoke_mode if not specified
        if config.lambda_invoke_mode == LambdaInvokeMode::Buffered {
            config.lambda_invoke_mode = LambdaInvokeMode::Buffered;
        }
        
        Ok(config)
    }

    pub fn from_cli() -> Result<Self, Box<dyn std::error::Error>> {
        let matches = Command::new("lambda-web-gateway")
            .version("0.1.0")
            .author("Harold Sun <sunhua@amazon.com>")
            .about("A gateway to AWS Lambda functions")
            .arg(
                Arg::new("addr")
                    .short('b')
                    .long("bind-address")
                    .value_name("ADDRESS")
                    .help("Sets the bind address (default: 0.0.0.0:8000)")
                    .required(false)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                Arg::new("lambda-function-name")
                    .short('f')
                    .long("lambda-function-name")
                    .value_name("FUNCTION_NAME")
                    .help("Sets the Lambda function name")
                    .required(true)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                Arg::new("lambda-invoke-mode")
                    .short('m')
                    .long("lambda-invoke-mode")
                    .value_name("INVOKE_MODE")
                    .help("Sets the Lambda invoke mode (default: Buffered)")
                    .required(false)
                    .value_parser(value_parser!(LambdaInvokeMode)),
            )
            .arg(
                Arg::new("api-keys")
                    .short('k')
                    .long("api-keys")
                    .value_name("API_KEYS")
                    .help("Sets the API keys")
                    .required(false)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                Arg::new("auth-mode")
                    .short('a')
                    .long("auth-mode")
                    .value_name("AUTH_MODE")
                    .help("Sets the authentication mode (default: Open)")
                    .required(false)
                    .value_parser(value_parser!(AuthMode)),
            )
            .get_matches();

        let lambda_function_name = matches
            .get_one::<String>("lambda-function-name")
            .expect("lambda-function-name is required")
            .clone();
        let lambda_invoke_mode = matches
            .get_one::<LambdaInvokeMode>("lambda-invoke-mode")
            .cloned()
            .unwrap_or(LambdaInvokeMode::Buffered);
        let api_keys: HashSet<String> = matches
            .get_many::<String>("api-keys")
            .ok_or("Missing api-keys")?
            .map(|s| s.clone())
            .collect();

        let auth_mode = matches
            .get_one::<AuthMode>("auth-mode")
            .cloned()
            .unwrap_or(AuthMode::Open);

        let addr = matches
            .get_one::<String>("addr")
            .cloned()
            .unwrap_or_else(|| "0.0.0.0:8000".to_string());

        Ok(Config {
            lambda_function_name,
            lambda_invoke_mode,
            api_keys,
            auth_mode,
            addr,
        })
    }
}
impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_cli().or_else(|_| {
            let config_path = std::path::PathBuf::from("config.yaml");
            Self::from_yaml_file(&config_path)
        })
    }
}
