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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LambdaInvokeMode {
    Buffered,
    ResponseStream,
}

impl Config {
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_content)?;
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
                    .value_parser(["Buffered", "ResponseStream"]),
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
            .get_matches();

        let lambda_function_name = matches.get_one::<String>("lambda-function-name").unwrap().clone();
        let lambda_invoke_mode = match matches.get_one::<String>("lambda-invoke-mode").unwrap().as_str() {
            "Buffered" => LambdaInvokeMode::Buffered,
            "ResponseStream" => LambdaInvokeMode::ResponseStream,
            _ => return Err("Invalid invoke mode".into()),
        };
        let api_keys: HashSet<String> = matches
            .get_many::<String>("api-keys")
            .unwrap()
            .map(|s| s.clone())
            .collect();

        Ok(Config {
            lambda_function_name,
            lambda_invoke_mode,
            api_keys,
        })
    }
}
