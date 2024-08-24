# Lambda Web Gateway

A high-performance web gateway for AWS Lambda functions, written in Rust.

## Features

- Seamless integration with AWS Lambda functions
- Support for both buffered and streaming Lambda invocations
- Configurable authentication (Open or API Key)
- Request transformation from HTTP to Lambda-compatible format
- Automatic handling of base64 encoding/decoding for request/response bodies
- Built with Rust and Axum for high performance and reliability
- Docker support for easy deployment and scaling

## Prerequisites

- Rust (latest stable version)
- AWS account and credentials configured
- AWS Lambda function(s) to be exposed via the gateway
- Docker (optional, for containerized deployment)

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
   ./target/release/lambda-web-gateway --lambda-function-name my-function --lambda-invoke-mode ResponseStream --auth-mode ApiKey --api-keys key1,key2
   ```

## Docker Deployment

To build and run using Docker:

1. Build the Docker image:
   ```
   docker build -t lambda-web-gateway .
   ```

2. Run the container:
   ```
   docker run -p 8000:8000 -v /path/to/your/config.yaml:/config.yaml lambda-web-gateway
   ```

## Usage

Once running, the gateway listens for HTTP requests on `0.0.0.0:8000`. All requests (except `/healthz`) are forwarded to the configured Lambda function.

- Health check: `GET /healthz`
- Lambda invocation: Any method on `/` or `/*path`

For API Key authentication, include the key in the `x-api-key` header or as a Bearer token in the `Authorization` header.

## Performance Considerations

- The gateway is optimized for high throughput and low latency.
- Streaming responses are supported for improved performance with large payloads.
- The use of Rust and Axum ensures efficient resource utilization.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
