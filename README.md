# rust-tcp-http
Multithreaded TCP/HTTP server with input thread to pass messages. 
Each request is processed by worker thread from threadpool which max number is pre-defined in the program. 

### Info

Max number of workers in threadpool is defined in: 
```
server::threadpool::static THREAD_LIMIT : usize = 10;
```
Server & database configuration is loaded from `.env`:
```
# Fresh deployment
$ mv .env.example .env && vi .env
```

HTTP requests are used for database queries (`GET = SELECT` & `POST = INSERT`). \
Non-HTTP TCP request opens persistent connection to server (exchange messages). \
Encrypter (in development)

### Run

```bash
# run server:
cargo run --bin rust-tcp-http


### tcp requests:
cargo run --bin client <server_ip> <server_port>


### Http requests:
# insert user (pre-defined values in database::create_users_from_vec()):
curl -X POST <server_ip>:<server_port>/users
# insert token (pre-defined values in database::create_tokens_from_vec()):
curl -X POST <server_ip>:<server_port>/tokens

# get users:
curl <server_ip>:<server_port>/users
# get tokens:
curl <server_ip>:<server_port>/tokens
  
  
# run encrypter:
cargo run \
      --bin encrypter \
      <algo(RSA|AES)> \
      <data> \
      <mode(private|public)> \
      <passphrase> \
      <keypair_id(creates new id if non-existing)>
  
```


### SSH tunnel
```bash
ssh -v -L 127.0.0.1:47111:<rust_server_ip>:<rust_server_port> localhost

# client requests get forwarded to rust server
cargo run --bin client 127.0.0.1 47111
```


### Database
```sql
-- 'rust_db' database:
+-------------------+
| Tables_in_rust_db |
+-------------------+
| connected         |
| roles             |
| tokens            |
| users             |
+-------------------+

-- table 'connected':
+------------+------------------+------+-----+-------------------+-----------------------------------------------+
| Field      | Type             | Null | Key | Default           | Extra                                         |
+------------+------------------+------+-----+-------------------+-----------------------------------------------+
| id         | bigint unsigned  | NO   | PRI | NULL              | auto_increment                                |
| user_id    | bigint unsigned  | NO   |     | NULL              |                                               |
| ip         | varchar(15)      | NO   |     | NULL              |                                               |
| port       | int unsigned     | NO   |     | NULL              |                                               |
| proxy      | json             | YES  |     | NULL              |                                               |
| note       | json             | YES  |     | NULL              |                                               |
| blacklist  | tinyint unsigned | YES  |     | 0                 |                                               |
| created_at | timestamp        | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED                             |
| updated_at | timestamp        | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED on update CURRENT_TIMESTAMP |
| test       | json             | YES  |     | NULL              |                                               |
+------------+------------------+------+-----+-------------------+-----------------------------------------------+

-- table 'roles':
+------------+-----------------+------+-----+-------------------+-----------------------------------------------+
| Field      | Type            | Null | Key | Default           | Extra                                         |
+------------+-----------------+------+-----+-------------------+-----------------------------------------------+
| id         | bigint unsigned | NO   | PRI | NULL              | auto_increment                                |
| type       | varchar(255)    | NO   |     | basic             |                                               |
| config     | json            | YES  |     | NULL              |                                               |
| created_at | timestamp       | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED                             |
| updated_at | timestamp       | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED on update CURRENT_TIMESTAMP |
+------------+-----------------+------+-----+-------------------+-----------------------------------------------+

-- table 'tokens':
+---------------+-----------------+------+-----+-------------------+-----------------------------------------------+
| Field         | Type            | Null | Key | Default           | Extra                                         |
+---------------+-----------------+------+-----+-------------------+-----------------------------------------------+
| id            | bigint unsigned | NO   | PRI | NULL              | auto_increment                                |
| user_id       | bigint unsigned | NO   |     | NULL              |                                               |
| token_type    | varchar(255)    | NO   |     | Bearer            |                                               |
| access_token  | varchar(255)    | NO   |     | NULL              |                                               |
| refresh_token | varchar(255)    | NO   |     | NULL              |                                               |
| token_expire  | timestamp       | YES  |     | NULL              |                                               |
| created_at    | timestamp       | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED                             |
| updated_at    | timestamp       | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED on update CURRENT_TIMESTAMP |
+---------------+-----------------+------+-----+-------------------+-----------------------------------------------+

-- table 'users':
+-------------------+------------------+------+-----+-------------------+-----------------------------------------------+
| Field             | Type             | Null | Key | Default           | Extra                                         |
+-------------------+------------------+------+-----+-------------------+-----------------------------------------------+
| id                | bigint unsigned  | NO   | PRI | NULL              | auto_increment                                |
| role_id           | bigint unsigned  | NO   |     | NULL              |                                               |
| username          | varchar(255)     | NO   |     | NULL              |                                               |
| email             | varchar(255)     | NO   |     | NULL              |                                               |
| password          | varchar(255)     | NO   |     | NULL              |                                               |
| config            | json             | YES  |     | NULL              |                                               |
| active            | tinyint unsigned | YES  |     | 0                 |                                               |
| remember_token    | varchar(100)     | YES  |     | NULL              |                                               |
| avatar            | varchar(255)     | NO   |     | default.jpg       |                                               |
| email_verified_at | timestamp        | YES  |     | NULL              |                                               |
| created_at        | timestamp        | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED                             |
| updated_at        | timestamp        | YES  |     | CURRENT_TIMESTAMP | DEFAULT_GENERATED on update CURRENT_TIMESTAMP |
+-------------------+------------------+------+-----+-------------------+-----------------------------------------------+
```
