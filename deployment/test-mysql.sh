#!/bin/sh

kubectl delete po mysql-client >2/dev/null

if [ -z "$1" ]; then
    ip=$(hostname -I | cut -d ' ' -f1)
else
    ip="$1"
fi
if [ -z "$2" ]; then
    port=3306
else
    port=$(($2 + 0))
fi

echo "mysql -h $ip -P $port -e 'SELECT * FROM rust_db.basic;'"

kubectl run mysql-client \
    --image=mysql:8.0 \
    -i \
    --rm \
    --restart=Never \
    -- mysql -h $ip -P $port -e 'SELECT * FROM rust_db.basic;'

