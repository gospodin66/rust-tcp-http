/*
 * We bring std::io::prelude into scope to get access to certain
 * traits that let us read from and write to the stream
 */
use std::net::{TcpStream,TcpListener,SocketAddr};
use std::sync::{Arc,Mutex,mpsc};

use crate::server::cstmconfig::AssetsConfig;
use crate::server::thrchannel::ThrChannel;

mod threadpool;
mod thrstdin;
mod helpers;
mod database;
mod cstmconfig;
mod cstmfiles;
mod headers;
mod request;
mod response;
mod thrchannel;

pub struct Server {}

impl Server {

    pub fn server(&self) -> Result<(), String>{
        let cfg: cstmconfig::ServerConfig = cstmconfig::ServerConfig::new_cfg();
        let port1 : u16 = cfg.port1;
        let port2 : u16 = cfg.port2;
        let assets_cfg: AssetsConfig = cstmconfig::AssetsConfig::new_cfg();
        let fpath: String = assets_cfg.log_dir+"/"+&assets_cfg.log_path;
        /*
        *  convert ip address from .env file: String => Vec<&str> => Vec<u8> => [u8; 4]
        */
        let ip_str : Vec<&str> = cfg.host.as_str().split('.').collect();
        let ip_vec : Vec<u8> = ip_str.into_iter().map(|val: &str| val.parse::<u8>().unwrap()).collect();
        let ip : [u8; 4] = helpers::vec_to_arr(ip_vec);
        let addrs: [SocketAddr; 2] = [
            SocketAddr::from((ip, port1)),
            SocketAddr::from((ip, port2)),
        ];
        match cstmfiles::f_create(&fpath) {
            Ok(()) => { println!("core: Successfuly created log file at {}", &fpath); }
            Err(_e) => {}
        }
        println!("core: Initializing thread channel.");
        let thrstdin_thrmain_channel: ThrChannel = thrchannel::ThrChannel::new_channel();
        println!("core: Initializing input thread.");
        thrstdin::init_thread(thrstdin_thrmain_channel.rx).unwrap();
        match self::init_server(&addrs) {
            Ok(listener) => {
                println!("core: [{}] listening for connections..", &addrs[0]);
                match self::listen_for_connections(
                    &listener,
                    thrstdin_thrmain_channel.tx
                ) {
                    Ok(()) => { Ok(()) },
                    Err(e) => {
                        return Err(format!("core: Error on listener: {}", e));
                    }
                }
            }, 
            Err(e) => {
                return Err(format!("core: Error initializing server: {}", e));
            }
        }
    }
}


fn init_server(ip_port: &[SocketAddr; 2]) -> Result<TcpListener, String>{
    match TcpListener::bind(format!("{}", ip_port[0])) {
        Ok(listener) => { Ok(listener) },
        _ => {
            println!("core: Error on bind().. trying another ip:port pair..");
            match TcpListener::bind(format!("{}", ip_port[1])) {
                Ok(listener) => { Ok(listener) },
                _ => {
                    return Err(format!("core: Error on bind() on fallback ip:port pair."));
                }
            }
        }
    }
}


fn listen_for_connections(
    listener: &TcpListener,
    thrstdin_thrmain_channel_tx: Arc<Mutex<mpsc::Sender<TcpStream>>>
) -> Result<(), String> {
    match threadpool::handle_in_threadpool(&listener, thrstdin_thrmain_channel_tx) {
        Ok(()) => { println!("core: Worker finsihed the job successfuly."); },
        Err(e) => { println!("core: Error on threadpool handler: {}", e); }
    }
    Ok(())
}