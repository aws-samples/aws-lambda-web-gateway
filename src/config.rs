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
    pub auth_mode: AuthMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthMode {
    Open,
    ApiKey,
}

impl AuthMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "open" => AuthMode::Open,
            "apikey" => AuthMode::ApiKey,
            _ => panic!("Invalid auth mode: {}", s),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LambdaInvokeMode {
    Buffered,
    ResponseStream,
}

impl LambdaInvokeMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "buffered" => LambdaInvokeMode::Buffered,
            "responsestream" => LambdaInvokeMode::ResponseStream,
            _ => panic!("Invalid invoke mode: {}", s),
        }
    }
}

impl Config {
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&config_content)?;
        config.lambda_invoke_mode = config.lambda_invoke_mode.map(|invoke_mode| {
            LambdaInvokeMode::from_str(&invoke_mode.to_string().to_lowercase())
        });
        config.auth_mode = config.auth_mode.map(|auth_mode| {
            AuthMode::from_str(&auth_mode.to_string().to_lowercase())
        });
        Ok(config)
    }

    pub fn from_cli() -> Result<Self, Box<dyn std::error::Error>> {
        let matches = Command::new("lambda-gateway")
            .version("0.1.0")
            .author("Harold Sun <sunhua@amazon.com>")
            .about("A gateway to AWS Lambda functions")
            .arg(
                Arg::new("lambda-function-name")
                    .short('f')
                    .long("lambda-function-name")
                    .value_name("FUNCTION_NAME")
                    .help("Sets the Lambda function name")
                    .required(false)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                Arg::new("lambda-invoke-mode")
                    .short('m')
                    .long("lambda-invoke-mode")
                    .value_name("INVOKE_MODE")
                    .help("Sets the Lambda invoke mode")
                    .required(false)
                    .value_parser(|s: &str| {
                        match s.to_lowercase().as_str() {
                            "buffered" => Ok(LambdaInvokeMode::Buffered),
                            "responsestream" => Ok(LambdaInvokeMode::ResponseStream),
                            _ => Err(format!("Invalid invoke mode: {}", s)),
                        }
                    }),
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
                    .help("Sets the authentication mode")
                    .required(false)
                    .value_parser(|s: &str| {
                        match s.to_lowercase().as_str() {
                            "open" => Ok(AuthMode::Open),
                            "apikey" => Ok(AuthMode::ApiKey),
                            _ => Err(format!("Invalid auth mode: {}", s)),
                        }
                    }),
            )
            .get_matches();

        let lambda_function_name = matches
            .get_one::<String>("lambda-function-name")
            .ok_or("Missing lambda-function-name")?
            .clone();
        let lambda_invoke_mode = match matches
            .get_one::<String>("lambda-invoke-mode")
            .ok_or("Missing lambda-invoke-mode")?
            .as_str()
        {
            "Buffered" => LambdaInvokeMode::Buffered,
            "ResponseStream" => LambdaInvokeMode::ResponseStream,
            _ => return Err("Invalid invoke mode".into()),
        };
        let api_keys: HashSet<String> = matches
            .get_many::<String>("api-keys")
            .ok_or("Missing api-keys")?
            .map(|s| s.clone())
            .collect();

        let auth_mode = match matches
            .get_one::<String>("auth-mode")
            .ok_or("Missing auth-mode")?
            .as_str()
        {
            "Open" => AuthMode::Open,
            "ApiKey" => AuthMode::ApiKey,
            _ => return Err("Invalid auth mode".into()),
        };

        Ok(Config {
            lambda_function_name,
            lambda_invoke_mode,
            api_keys,
            auth_mode,
        })
    }
}
