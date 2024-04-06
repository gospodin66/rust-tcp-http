#!/bin/sh

cd ..

cargo build --bin rust_tcp_http
cargo build --bin client

docker build -t rust-server:1.0 . -f deployment/DockerfileServer
docker build -t rust-client:1.0 . -f deployment/DockerfileClient
docker build -t rust-mysql:1.0 . -f deployment/DockerfileMysql

helm upgrade --install base helm-chart-base
sleep 5
helm upgrade --install base helm-chart-mysql
sleep 5
helm upgrade --install base helm-chart-server
sleep 5
helm upgrade --install base helm-chart-client

echo "Stack initialized!"
exit 0
