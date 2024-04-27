# reverseproxy password: Pr1+'13(()oxXx$$ip4s$_136=?$I%#
echo "<PASSWORD>" | base64 
# exec in nginx container
sudo htpasswd -c /etc/nginx/.htpasswd cheki
# test 
curl -v --insecure -L -u cheki:UHIxKycxMygoKW94WHgzODYwaXA0cz0/JSMK https://10.0.2.16/users/
curl -v --insecure -L -u cheki:UHIxKycxMygoKW94WHgzODYwaXA0cz0/JSMK https://10.0.2.16/tokens/
