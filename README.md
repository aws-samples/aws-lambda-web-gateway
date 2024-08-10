# Lambda Web Gateway

A flexible and efficient web gateway for AWS Lambda functions, written in Rust.

## Features

- Seamless integration with AWS Lambda functions
- Support for both buffered and streaming Lambda invocations
- Configurable authentication (Open or API Key)
- Request transformation from HTTP to Lambda-compatible format
- Automatic handling of base64 encoding/decoding
- Built with Rust for high performance and reliability

## Prerequisites

- Rust (latest stable version)
- AWS account and credentials configured
- AWS Lambda function(s) to be exposed via the gateway

## Configuration

The gateway can be configured using either a YAML file (`config.yaml`) or command-line arguments. Configuration options include:

- Lambda function name
- Lambda invoke mode (Buffered or ResponseStream)
- API keys (for API Key authentication mode)
- Authentication mode (Open or ApiKey)

Example `config.yaml`:

```yaml
lambda_function_name: "my-lambda-function"
lambda_invoke_mode: "ResponseStream"
auth_mode: "ApiKey"
api_keys:
  - "key1"
  - "key2"
```

## Building and Running

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/lambda-web-gateway.git
   cd lambda-web-gateway
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the gateway:
   ```
   ./target/release/lambda-web-gateway
   ```

   Or with command-line arguments:
   ```
   ./target/release/lambda-web-gateway --lambda-function-name my-function --lambda-invoke-mode Buffered --auth-mode ApiKey --api-keys key1,key2
   ```

## Usage

Once running, the gateway will listen for HTTP requests on `0.0.0.0:8000`. All requests (except `/healthz`) will be forwarded to the configured Lambda function.

- Health check: `GET /healthz`
- Lambda invocation: Any method on `/` or `/*path`

For API Key authentication, include the key in the `x-api-key` header or as a Bearer token in the `Authorization` header.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
