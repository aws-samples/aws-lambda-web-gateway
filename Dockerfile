FROM public.ecr.aws/amazonlinux/amazonlinux:2.0.20210427.0
COPY target/x86_64-unknown-linux-musl/release/lambda-gateway /lambda-gateway

CMD ["/lambda-gateway"]