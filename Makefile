build:
	CC=x86_64-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

package: build
	aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws/awsguru
	docker build -t public.ecr.aws/awsguru/lambda-gateway .

publish: package
	docker push public.ecr.aws/awsguru/lambda-gateway

deploy: publish
	kubectl apply -f k8s