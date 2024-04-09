#!/bin/sh
cargo build --bin rust-tcp-http
cargo build --bin client

docker build -t rust-server:1.0 .. -f DockerfileServer 
docker build -t rust-client:1.0 .. -f DockerfileClient
docker build -t rust-mysql:1.0 .. -f DockerfileMysql 

docker save rust-server:1.0 -o rust-server.tar 
docker save rust-client:1.0 -o rust-client.tar
docker save rust-mysql:1.0 -o rust-mysql.tar

sudo k3s ctr images import rust-server.tar 
sudo k3s ctr images import rust-client.tar
sudo k3s ctr images import rust-mysql.tar

helm upgrade --install server helm-chart-server
helm upgrade --install client helm-chart-client
helm upgrade --install base helm-chart-base
helm upgrade --install mysql helm-chart-mysql

echo "Stack initialized!"
exit 0
