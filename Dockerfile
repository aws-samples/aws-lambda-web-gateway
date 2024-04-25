FROM public.ecr.aws/amazonlinux/amazonlinux:2023-minimal
COPY target/x86_64-unknown-linux-musl/release/lambda-gateway /lambda-gateway

CMD ["/lambda-gateway"]