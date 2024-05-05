# reverseproxy password: Pr1+'13(()oxXx$$ip4s$_136=?$I%#
echo "<PASSWORD>" | base64 
# exec in nginx container
sudo htpasswd -c /etc/nginx/.htpasswd cheki
# test get
curl -v --insecure -L -u cheki:UHIxKycxMygoKW94WHgzODYwaXA0cz0/JSMK https://10.0.2.16/users/
# test insert 
curl -v \
    --insecure \
    -L \
    -u cheki:UHIxKycxMygoKW94WHgzODYwaXA0cz0/JSMK \
    -X POST \
    -d "user_id=92" \
    -d "token_type=Bearer" \
    -d "access_token=9jojOELU1YcWq1sh3dRHLdn+GjA7e/Hn" \
    -d "refresh_token=OGaQHohcJ4skNBulc5KPCMywyNB4JB7UvSS8isvsMTo=" \
    -d "token_expire=2024-04-17 23:51:40" \
    -d "created_at=2024-04-17 23:51:40" \
    -d "updated_at=2024-04-17 23:51:40" \
    https://10.0.2.16/proxy/tokens

curl -v \
    --insecure \
    -L \
    -u cheki:UHIxKycxMygoKW94WHgzODYwaXA0cz0/JSMK \
    -X POST \
    -d "role_id=1" \
    -d "username=admin" \
    -d "email=admin@no-existing.com" \
    -d "password=admin@123" \
    -d "config={\"test1\": \"test11\", \"test22\": \"testval2\"}" \
    -d "active=true" \
    -d "remember_token=apisdvv3uzz453b4" \
    -d "avatar=/img/default/user-avatar.png" \
    -d "created_at=2024-04-17 23:51:40" \
    -d "updated_at=2024-04-17 23:51:40" \
    https://10.0.2.16/proxy/users

curl -v --insecure -L https://10.0.2.16