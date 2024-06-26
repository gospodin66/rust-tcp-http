/*
 * We bring std::io::prelude into scope to get access to certain
 * traits that let us read from and write to the stream
 */
use std::net::{SocketAddr, TcpListener};
use crate::server::cstmconfig::AssetsConfig;
mod threadpool;
mod thrstdin;
mod helpers;
mod database;
mod cstmconfig;
mod cstmfiles;
mod httpconnection;
mod tcpconnection;
mod thrchannel;
mod validator;

const IDENTIFICATOR: &str = "core";

pub struct Server {}

impl Server {

    pub fn server(&self) -> Result<(), String> {
        let cfg: cstmconfig::ServerConfig = cstmconfig::ServerConfig::new_cfg();
        let assets_cfg: AssetsConfig = cstmconfig::AssetsConfig::new_cfg();
        let log_path: String = format!("{}/{}", &assets_cfg.log_dir, &assets_cfg.log_path);
        match cstmfiles::f_create_dir(&assets_cfg.log_dir) {
            Ok(()) => {
                println!("cstmfiles: Parent dir created: {}", assets_cfg.log_dir);
            },
            Err(e) => {
                println!("cstmfiles: Parent dir already exists: {}", e);
            }
        }
        /*
        *  convert ip address from .env file: &str => Vec<&str> => Vec<u8> => [u8; 4]
        */
        let ip_vec: Vec<u8> = 
            cfg.host.as_str()
                    .split('.')
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .map(|val: &str| match val.parse::<u8>() {
                            Ok(v) => v,
                            Err(e) => {
                                println!("Error parsing port: {}", e);
                                0
                            }
                        }
                    ).collect::<Vec<u8>>();
        let ip: [u8; 4] = helpers::vec_to_arr(ip_vec);
        let addrs: [SocketAddr; 2] = [
            SocketAddr::from((ip, cfg.port1)),
            SocketAddr::from((ip, cfg.port2)),
        ];
        match cstmfiles::f_create(&log_path) {
            Ok(()) => { println!("{}: Successfuly created log file at {}", IDENTIFICATOR, &log_path); }
            Err(_e) => {}
        }
        match self::init_server(&addrs) {
            Ok(listener) => {
                println!("{}: [{}] listening for connections..", IDENTIFICATOR, &addrs[0]);
                match self::listen_for_connections(&listener) {
                    Ok(()) => Ok(()),
                    Err(e) => return Err(format!("{}: Error on listener: {}", IDENTIFICATOR, e))
                }
            }, 
            Err(e) => return Err(format!("{}: Error initializing server: {}", IDENTIFICATOR, e))
        }
    }
}


fn init_server(ip_port: &[SocketAddr; 2]) -> Result<TcpListener, String>{
    match TcpListener::bind(format!("{}", ip_port[0])) {
        Ok(listener) => Ok(listener),
        _ => {
            println!("{}: Error on bind().. Trying fallback ip:port pair..", IDENTIFICATOR);
            match TcpListener::bind(format!("{}", ip_port[1])) {
                Ok(listener) => Ok(listener),
                _ => return Err(format!("{}: Error on bind() on fallback ip:port pair.", IDENTIFICATOR))
            }
        }
    }
}


fn listen_for_connections(listener: &TcpListener) -> Result<(), String> {
    match threadpool::handle_in_threadpool(&listener) {
        Ok(()) => { println!("{}: Worker finsihed the job successfuly.", IDENTIFICATOR); },
        Err(e) => { println!("{}: Error on threadpool handler: {}", IDENTIFICATOR, e); }
    }
    Ok(())
}