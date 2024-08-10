build:
	CC=x86_64-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

package: build
	docker build -t lambda-web-gateway:latest 048972532408.dkr.ecr.us-west-2.amazonaws.com/lambda-web-gateway:latest .

publish: package
	aws ecr get-login-password --region us-west-2 | docker login --username AWS --password-stdin 048972532408.dkr.ecr.us-west-2.amazonaws.com
	docker push 048972532408.dkr.ecr.us-west-2.amazonaws.com/lambda-web-gateway:latest