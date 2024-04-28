#!/bin/sh
cargo build --bin rust-tcp-http
cargo build --bin client

docker build -t rust-server:1.1 .. -f DockerfileServer 
docker build -t rust-nginx:1.1 .. -f DockerfileNginx 
docker build -t rust-client:1.1 .. -f DockerfileClient
docker build -t rust-mysql:1.1 .. -f DockerfileMysql 

docker tag rust-server:1.1 127.0.0.1:5000/v2/rust-server:1.1
docker tag rust-mysql:1.1 127.0.0.1:5000/v2/rust-mysql:1.1
docker tag rust-client:1.1 127.0.0.1:5000/v2/rust-client:1.1
docker tag rust-nginx:1.1 127.0.0.1:5000/v2/rust-nginx:1.1

docker push 127.0.0.1:5000/v2/rust-server:1.1
docker push 127.0.0.1:5000/v2/rust-mysql:1.1
docker push 127.0.0.1:5000/v2/rust-client:1.1
docker push 127.0.0.1:5000/v2/rust-nginx:1.1

#docker save rust-server:1.1 -o rust-server.tar 
#docker save rust-client:1.1 -o rust-client.tar
#docker save rust-mysql:1.1 -o rust-mysql.tar

#crane push rust-server.tar 127.0.0.1:5000

#sudo k3s ctr images import rust-server.tar 
#sudo k3s ctr images import rust-client.tar
#sudo k3s ctr images import rust-mysql.tar


# rm image dir & run garbage collect
sudo rm -rf /var/lib/docker/volumes/registry-storage/_data/docker/registry/v2/repositories/v2/rust-server && \
sudo rm -rf /var/lib/docker/volumes/registry-storage/_data/docker/registry/v2/repositories/v2/rust-nginx && \
docker exec -it private-registry bin/registry garbage-collect /etc/docker/registry/config.yml

# uninstall
helm uninstall server client base mysql nginx

# install
helm upgrade --install server helm-chart-server
helm upgrade --install client helm-chart-client
helm upgrade --install base helm-chart-base
helm upgrade --install mysql helm-chart-mysql
helm upgrade --install nginx helm-chart-nginx

echo "Stack initialized!"
exit 0