#[derive(Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port1: u16,
    pub port2: u16,
    pub request_methods: Vec<String>,
}

pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

pub struct BaseConfig {
    pub http_protocol: String,
}

pub struct AssetsConfig {
    pub html_base_path: String,
    pub log_dir: String,
    pub log_path: String,
}

pub struct AppConfig {
    pub base: BaseConfig,
    pub server: ServerConfig,
    pub database: DbConfig,
    pub assets: AssetsConfig,
}



#[allow(dead_code)]
impl AppConfig {
    pub fn new_cfg() -> AppConfig {
        AppConfig {
            server: ServerConfig::new_cfg(),
            database: DbConfig::new_cfg(),
            base: BaseConfig::new_cfg(),
            assets: AssetsConfig::new_cfg(),
        }
    }
}

impl BaseConfig {
    pub fn new_cfg() -> BaseConfig {
        match dotenv::dotenv().ok() {
            Some(_envpath) => {},
            None => {
                println!("BaseConfig: Error loading env vars: loading default");
                let _base_cfg : BaseConfig = BaseConfig {
                    http_protocol: String::from("HTTP/1.1").to_string()
                };
            }
        }
        let _base_cfg : BaseConfig = BaseConfig {
            http_protocol: dotenv::var("SERVER.HTTP_PROTOCOL").unwrap()
        };

        _base_cfg
    }
}

impl AssetsConfig {
    pub fn new_cfg() -> AssetsConfig {
        match dotenv::dotenv().ok() {
            Some(_envpath) => {},
            None => {
                println!("AssetsConfig: Error loading env vars: loading default");
                let _assets_cfg : AssetsConfig = AssetsConfig {
                    html_base_path: String::from("src/bin/server/html"),
                    log_dir: String::from("log"),
                    log_path: String::from("logs.log"),
                };
            }
        }
        let _assets_cfg : AssetsConfig = AssetsConfig {
            html_base_path: dotenv::var("APP.HTML_BASE_PATH").unwrap(),
            log_dir: dotenv::var("APP.LOG_DIR").unwrap(),
            log_path: dotenv::var("APP.LOG_PATH").unwrap(),
        };

        _assets_cfg
    }
}

impl ServerConfig {
    pub fn new_cfg() -> ServerConfig {
        match dotenv::dotenv().ok() {
            Some(_envpath) => {},
            None => {
                println!("ServerConfig: Error loading env vars - loading default");
                let request_methods: String = String::from("GET,POST,OPTIONS,HEAD");
                let req_meth: Vec<String> = request_methods.split(",")
                                              .map(str::to_string)
                                              .collect();
                let _server_cfg : ServerConfig = ServerConfig {
                    host: String::from("127.0.0.1").to_string(),
                    port1: 9998_u16,
                    port2: 9999_u16,
                    request_methods: req_meth
                };
            }
        }
        let port1_str: String = dotenv::var("SERVER.PORT1").unwrap();
        let port2_str: String = dotenv::var("SERVER.PORT2").unwrap();
        let port1: u16 = port1_str.trim().parse::<u16>().unwrap();
        let port2: u16 = port2_str.trim().parse::<u16>().unwrap();
        let request_methods: String = dotenv::var("SERVER.REQUEST_METHODS").unwrap();
        let req_meth: Vec<String> = request_methods.split(",")
                                      .map(str::to_string)
                                      .collect();
        let _server_cfg: ServerConfig = ServerConfig {
            host: dotenv::var("SERVER.HOST").unwrap(),
            port1: port1,
            port2: port2,
            request_methods: req_meth
        };
        
        return _server_cfg
    }
}

impl DbConfig {
    pub fn new_cfg() -> DbConfig {
        match dotenv::dotenv().ok() {
            Some(_envpath) => {},
            None => {
                println!("DbConfig: Error loading env vars: loading default");
                let _db_cfg : DbConfig = DbConfig {
                    host: String::from("127.0.0.1").to_string(),
                    port: 3306_u16,
                    user: String::from("user").to_string(),
                    password: String::from("password").to_string(),
                    database: String::from("example_database").to_string(),
                };
            }
        }
        let db_port_str : String = dotenv::var("DATABASE.PORT").unwrap();
        let db_port : u16 = db_port_str.trim().parse::<u16>().unwrap();
        let _db_cfg : DbConfig = DbConfig {
            host: dotenv::var("DATABASE.HOST").unwrap(),
            port: db_port,
            user: dotenv::var("DATABASE.USER").unwrap(),
            password: dotenv::var("DATABASE.PASSWORD").unwrap(),
            database: dotenv::var("DATABASE.DATABASE").unwrap(),
        };

        _db_cfg
    }
}