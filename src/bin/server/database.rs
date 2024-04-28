use mysql::*;
use mysql::prelude::*;
#[allow(unused_imports)]
use chrono::{Local, NaiveDateTime, NaiveTime, DateTime, NaiveDate};
use crate::server::cstmconfig;

//Serialize, Queryable
#[derive(Debug, Clone)]
pub struct User {
    role_id: u64,
    username: String,
    email: String,
    password: String,
    config: String,
    active: bool,
    remember_token: String,
    avatar: String,
    created_at: String,
    updated_at: String
}

//Serialize, Queryable
#[derive(Debug, Clone)]
pub struct Token {
    user_id: u64,
    token_type: String,
    access_token: String,
    refresh_token: String,
    token_expire: String,
    created_at: String,
    updated_at: String
}

pub struct Database {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}


impl Database {
    fn init(host: String, port: u16, user: String, password: String, database: String) -> Self {
        Database { host, port, user, password, database }
    }
}


pub fn create_tables() -> Result<(), > {

    /*
    ALTER TABLE users ALTER email_verified_at SET DEFAULT 'N/A';

    INSERT INTO roles (type,config,created_at,updated_at) VALUES ('Admin','{"privileges": 1}', '2024-04-13 11:18:58', '2024-04-13 11:18:58'), ('User','{"privileges": 2}', '2024-04-13 11:18:58', '2024-04-13 11:18:58');
    
    */
    let columns_connected: Vec<String> = vec![
        "id".to_string(),
        "user_id".to_string(),
        "ip".to_string(),
        "port".to_string(),
        "proxy".to_string(),
        "note".to_string(),
        "blacklist".to_string(),              
        "created_at".to_string(),
        "updated_at".to_string(),
        "test".to_string(),        
    ];
    let columns_roles: Vec<String> = vec![
        "id".to_string(),
        "type".to_string(),
        "config".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),        
    ];
    let columns_users: Vec<String> = vec![
        "id".to_string(),
        "role_id".to_string(),
        "username".to_string(),
        "email".to_string(),
        "password".to_string(),
        "config".to_string(),
        "active".to_string(),
        "remember_token".to_string(),
        "avatar".to_string(),
        "email_verified_at".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];
    let columns_tokens: Vec<String> = vec![
        "id".to_string(),
        "user_id".to_string(),
        "token_type".to_string(),
        "access_token".to_string(),
        "refresh_token".to_string(),
        "token_expire".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),        
    ];
    match create_table(String::from("users"), columns_users) {
        Ok(()) => {
            println!("SQL Table users created successfuly.");
        },
        Err(e) => {
            println!("SQL Error creating table: {}", e);
        }
    }
    match create_table(String::from("tokens"), columns_tokens) {
        Ok(()) => {
            println!("SQL Table tokens created successfuly.");
        },
        Err(e) => {
            println!("SQL Error creating table: {}", e);
        }
    }
    match create_table(String::from("roles"), columns_roles) {
        Ok(()) => {
            println!("SQL Table roles created successfuly.");
        },
        Err(e) => {
            println!("SQL Error creating table: {}", e);
        }
    }
    match create_table(String::from("connected"), columns_connected) {
        Ok(()) => {
            println!("SQL Table connected created successfuly.");
        },
        Err(e) => {
            println!("SQL Error creating table: {}", e);
        }
    }

    // custom db query - initialization:
    let mut conn: Conn;
    match init_db() {
        Ok(connection) => {
            conn = connection;
        },
        Err(e) => {
            let errmsg = format!("SQL: Error connecting to db: {}", e);
            println!("{}", errmsg);
            return Err(e);
        }
    }
    let mut tx: Transaction = conn.start_transaction(TxOpts::default()).unwrap();
    let query: &str = "ALTER TABLE users ALTER email_verified_at SET DEFAULT \'N/A\';
    INSERT INTO roles (type,config,created_at,updated_at) VALUES (\'Admin\',\'{\"privileges\": 1}\', \'2024-04-13 11:18:58\', \'2024-04-13 11:18:58\'), (\'User\',\'{\"privileges\": 2}\', \'2024-04-13 11:18:58\', \'2024-04-13 11:18:58\');";
    println!("Running initialization query..");
    match tx.query_drop(query) {
        Ok(_) => {
            return Ok(())
        },
        Err(e) => {
            tx.rollback().unwrap();
            return Err(e)
        }
    }

}

pub fn create_table(table: String, columns: Vec<String>) -> Result<(), > {

    let mut conn: Conn;
    match init_db() {
        Ok(connection) => {
            conn = connection;
        },
        Err(e) => {
            let errmsg = format!("SQL: Error connecting to db: {}", e);
            println!("{}", errmsg);
            return Err(e);
        }
    }

    let mut tx: Transaction = conn.start_transaction(TxOpts::default()).unwrap();
    let mut columns_sql: Vec<String> = Vec::new();
    let mut column_meta: String;

    for (i, column) in columns.iter().enumerate() {
        if column == "id" {
            column_meta = format!("{} BIGINT AUTO_INCREMENT PRIMARY KEY", column);
        } 
        else if column.contains("_id") {
            column_meta = format!("{} BIGINT", column);
        }        
        else if column == "created_at" || column == "updated_at" {
            column_meta = format!("{} TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP()", column);
        } else {
            column_meta = format!("{} VARCHAR(255) NOT NULL", column);
        }
        
        if i < columns.len() -1 {
            column_meta = format!("{},", column_meta);
        }

        columns_sql.push(column_meta);

    }

    let query = format!("CREATE TABLE {} ({})", table, &columns_sql.join(" "));

    println!("DEBUG QUERY: {:?}", query);

    match tx.query_drop(query) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            tx.rollback().unwrap();
            Err(e)
        }
    }
    
}

fn init_db() -> Result<Conn> {
    let db_config : cstmconfig::DbConfig = cstmconfig::DbConfig::new_cfg();
    let db: Database = Database::init(
        db_config.host,
        db_config.port,
        db_config.user,
        db_config.password,
        db_config.database
    );
    let opts: OptsBuilder = OptsBuilder::new()
                .ip_or_hostname(Some(db.host))
                .tcp_port(db.port)
                .user(Some(db.user))
                .pass(Some(db.password))
                .db_name(Some(db.database));
    match Conn::new(opts) {
        Ok(connection) => {
            Ok(connection)
        },
        Err(e) => {
            let errmsg = format!("SQL: Error connecting to database: {}", e);
            println!("{}", &errmsg);
            return Err(e);
        }
    }
}



impl User {
    pub fn user_to_string(user: &User) -> String {
        format!(
            "\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n",
            user.role_id,
            user.username,
            user.email,
            user.password,
            user.config,
            user.active,
            user.remember_token,
            user.avatar,
            user.created_at,
            user.updated_at,
        )
    }

    pub fn select_all() -> Result<Vec<User>>{
        let mut conn : Conn;
        let selected_users : Vec<User>;

        match init_db() {
            Ok(connection) => {
                conn = connection;
            },
            Err(e) => {
                let errmsg = format!("SQL: Error connecting to db: {}", e);
                println!("{}", errmsg);
                return Err(e);
            }
        }
        let stmt = "SELECT
                     role_id,
                     username,
                     email,
                     password,
                     config,
                     active,
                     remember_token,
                     avatar,
                     created_at,
                     updated_at
                    FROM users
                    ORDER BY created_at
                    DESC";
        let select_res = conn.query_map(
            stmt,
            |(role_id,
                username, 
                email, 
                password, 
                config, 
                active, 
                remember_token, 
                avatar, 
                created_at, 
                updated_at)|
            -> User {
                User {
                    role_id,
                    username,
                    email,
                    password,
                    config,
                    active,
                    remember_token,
                    avatar,
                    created_at,
                    updated_at
                }
            },
        );
        match select_res {
            Ok(users) => {
                selected_users = users;
            },
            Err(e) => {
                let errmsg = format!("SQL: Error selecting from db: {}", e);
                println!("{}", errmsg);
                return Err(e);
            }
        }
        Ok(selected_users)
    }

    pub fn create_users(params: std::collections::HashMap<&str, &str>) -> Result<()>{
        let users = vec![
            User { 
                role_id: params["role_id"].parse::<u64>().unwrap(),
                username: String::from(params["username"]),
                email: String::from(params["email"]), 
                password: String::from(params["password"]), 
                config: String::from(params["config"]), 
                active: params["active"].eq("true"),
                remember_token: String::from(params["remember_token"]), 
                avatar: String::from(params["avatar"]), 
                created_at: String::from(params["created_at"]), 
                updated_at: String::from(params["updated_at"]), 
            },
        ];
        /********************************************/
        match insert(users) {
            Ok(()) => {},
            Err(err) => {
                let errmsg = format!("SQL: Error on insert(): {}", err);
                println!("{}", &errmsg);
                return Err(err);
            }
        }
        /********************************************/
        fn insert(users: Vec<User>) -> Result<()> {
            let mut conn : Conn;
            match init_db() {
                Ok(connection) => {
                    conn = connection;
                },
                Err(e) => {
                    let errmsg = format!("SQL: Error connecting to db: {}", e);
                    println!("{}", errmsg);
                    return Err(e);
                }
            }
            let stmt =
            "INSERT INTO users
                (role_id,
                username,
                email,
                password,
                config,
                active,
                remember_token,
                avatar,
                created_at,
                updated_at)
            VALUES
                (:role_id,
                 :username,
                 :email,
                 :password,
                 :config,
                 :active,
                 :remember_token,
                 :avatar,
                 :created_at,
                 :updated_at)";
            
            // Strings are passed by reference!
            let __params = users.iter().map(|u| params!{
                "role_id" => u.role_id,
                "username" => &u.username,
                "email" => &u.email,
                "password" => &u.password,
                "config" => &u.config,
                "active" => u.active,
                "remember_token" => &u.remember_token,
                "avatar" => &u.avatar,
                "created_at" => &u.created_at,
                "updated_at" => &u.updated_at
            });
            let mut tx: Transaction = conn.start_transaction(TxOpts::default())?;
            println!("SQL: Params to insert:\n{:?}\n", __params);
            match tx.exec_batch(stmt, __params) {
                Ok(()) => {
                    println!("SQL: Successfuly inserted users!");
                }
                Err(err) => {
                    tx.rollback().unwrap();
                    let errmsg = format!("SQL: Error on insert(): {}", err);
                    println!("{}", &errmsg);
                    return Err(err);
                }
            }
            tx.commit().unwrap();
            Ok(())
        }
        /********************************************/
        Ok(())
    }
}

impl Token {
    pub fn token_to_string(token: &Token) -> String {
        format!(
            "\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n",
            token.user_id,
            token.token_type,
            token.access_token,
            token.refresh_token,
            token.token_expire,
            token.created_at,
            token.updated_at
        )
    }

    pub fn select_all() -> Result<Vec<Token>> {
        let mut conn : Conn;
        let selected_tokens : Vec<Token>;

        match init_db() {
            Ok(connection) => {
                conn = connection;
            },
            Err(e) => {
                let errmsg: String = format!("SQL: Error connecting to db: {}", e);
                println!("{}", errmsg);
                return Err(e);
            }
        }
        let stmt: &str = "SELECT
                           user_id,
                           token_type,
                           access_token,
                           refresh_token,
                           token_expire,
                           created_at,
                           updated_at
                          FROM tokens
                          ORDER BY created_at
                          DESC";
        let select_res: Result<Vec<Token>> = conn.query_map(
            stmt,
            |(user_id,
                token_type,
                access_token,
                refresh_token,
                token_expire,
                created_at,
                updated_at)| 
            -> Token {
                Token {
                    user_id,
                    token_type,
                    access_token,
                    refresh_token,
                    token_expire,
                    created_at,
                    updated_at
                }
            },
        );
        match select_res {
            Ok(tokens) => {
                selected_tokens = tokens;
            },
            Err(e) => {
                let errmsg = format!("Error selecting from db: {}", e);
                println!("{}", errmsg);
                return Err(e);
            }
        }
        Ok(selected_tokens)
    }

    pub fn create_tokens(params: std::collections::HashMap<&str, &str>) -> Result<()>{
        let tokens: Vec<Token> = vec![
            Token { 
                user_id: params["user_id"].parse::<u64>().unwrap(),
                token_type: String::from(params["token_type"]),
                access_token: String::from(params["access_token"]),
                refresh_token: String::from(params["refresh_token"]),
                token_expire: String::from(params["token_expire"]),
                created_at: String::from(params["created_at"]),
                updated_at: String::from(params["updated_at"]),
            },
        ];
        /********************************************/
        match insert(tokens) {
            Ok(()) => {},
            Err(err) => {
                let errmsg: String = format!("error on insert(): {}", err);
                println!("{}", &errmsg);
                return Err(err);
            }
        }
        /********************************************/
        fn insert(tokens: Vec<Token>) -> Result<()> {
            let mut conn : Conn;
            match init_db() {
                Ok(connection) => {
                    conn = connection;
                },
                Err(e) => {
                    let errmsg = format!("SQL: Error connecting to db: {}", e);
                    println!("{}", errmsg);
                    return Err(e);
                }
            }
            let stmt: &str =
            "INSERT INTO tokens
                (user_id,
                 token_type,
                 access_token,
                 refresh_token,
                 token_expire,
                 created_at,
                 updated_at)
            VALUES
                (:user_id,
                 :token_type,
                 :access_token,
                 :refresh_token,
                 :token_expire,
                 :created_at,
                 :updated_at)";
                
            // Strings are passed by reference!
            let __params = tokens.iter().map(|t| params! {
                "user_id" => t.user_id,
                "token_type" => &t.token_type,
                "access_token" => &t.access_token,
                "refresh_token" => &t.refresh_token,
                "token_expire" => &t.token_expire,
                "created_at" => &t.created_at,
                "updated_at" => &t.updated_at
            });
            let mut tx: Transaction = conn.start_transaction(TxOpts::default())?;
            println!("SQL: Params to insert:\n{:?}\n", __params);
            match tx.exec_batch(stmt, __params) {
                Ok(()) => {
                    println!("SQL: Successfuly inserted tokens!");
                }
                Err(err) => {
                    tx.rollback().unwrap();
                    let errmsg: String = format!("SQL: Error on insert(): {}", err);
                    println!("{}", &errmsg);
                    return Err(err);
                }
            }
            tx.commit().unwrap();
            Ok(())
        }
        /********************************************/
        Ok(())
    }
}