# Lambda Web Gateway

A high-performance web gateway for AWS Lambda functions, written in Rust.

## Features

- Seamless integration with AWS Lambda functions
- Support for both buffered and streaming Lambda invocations
- Configurable authentication (Open or API Key)
- Request transformation from HTTP to Lambda-compatible format
- Automatic handling of base64 encoding/decoding for request/response bodies
- Built with Rust and Axum for high performance and reliability
- Health check endpoint for monitoring
- Flexible configuration via YAML file or environment variables

## Prerequisites

- Rust (latest stable version)
- AWS account and credentials configured
- AWS Lambda function(s) to be exposed via the gateway

## Configuration

The gateway can be configured using a YAML file (`config.yaml`) or environment variables. Configuration options include:

- Lambda function name (required)
- Lambda invoke mode (Buffered or ResponseStream, default: Buffered)
- API keys (for API Key authentication mode)
- Authorization mode (Open or ApiKey, default: Open)
- Bind address (default: "0.0.0.0:8000")

Example `config.yaml`:

```yaml
lambda_function_name: "my-lambda-function"
lambda_invoke_mode: "ResponseStream"
auth_mode: "ApiKey"
api_keys:
  - "key1"
  - "key2"
addr: "0.0.0.0:8000"
```

Alternatively, you can use environment variables:

- `LAMBDA_FUNCTION_NAME`
- `LAMBDA_INVOKE_MODE`
- `API_KEYS` (comma-separated list)
- `AUTH_MODE` (default: Open)
- `ADDR`

Environment variables take precedence over the configuration file when both are present.

## Building and Running

1. Clone the repository:
   ```
   git clone https://github.com/aws-samples/aws-lambda-web-gateway.git
   cd aws-lambda-web-gateway
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Create a `config.yaml` file in the project root or set the necessary environment variables.

4. Run the gateway:
   ```
   ./target/release/lambda-web-gateway
   ```

## Usage

Once running, the gateway listens for HTTP requests on the configured address (default: `0.0.0.0:8000`). All requests (except `/healthz`) are forwarded to the configured Lambda function.

- Health check: `GET /healthz`
- Lambda invocation: Any method on `/` or `/*path`

For API Key authentication, include the key in the `x-api-key` header or as a Bearer token in the `Authorization` header.

## Performance Considerations

- The gateway is optimized for high throughput and low latency.
- Streaming responses are supported for improved performance with large payloads.
- The use of Rust and Axum ensures efficient resource utilization.

## Development

To contribute to the project:

1. Fork the repository
2. Create a new branch for your feature
3. Implement your changes
4. Write tests for your new functionality
5. Submit a pull request

Please refer to [CONTRIBUTING.md](CONTRIBUTING.md) for more details on the contribution process.

## Security

See [CONTRIBUTING.md](CONTRIBUTING.md#security-issue-notifications) for more information on reporting security issues.

## License

This project is licensed under the MIT-0 License. See the [LICENSE](LICENSE) file for details.
