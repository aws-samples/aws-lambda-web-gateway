FROM public.ecr.aws/amazonlinux/amazonlinux:2023-minimal
ADD target/x86_64-unknown-linux-musl/release/lambda-web-gateway /
ADD config.yaml /

ENTRYPOINT [ "/lambda-web-gateway" ]