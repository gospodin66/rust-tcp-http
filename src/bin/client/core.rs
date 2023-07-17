use std::borrow::Cow;
/*
 * We bring std::io::prelude into scope to get access to certain
 * traits that let us read from and write to the stream
 */
use std::net::{TcpStream,SocketAddr};
use std::io::Read;
//use std::time::Duration;
use std::env;
use std::thread::{self, JoinHandle};
use std::io;
use std::io::Write;
use chrono::Local;
use crate::helpers;
use crate::thrstdin;
use crate::coreerr::CoreErr;


pub fn client() -> Result<(), CoreErr>{
    let arg_host: String = match env::args().nth(1) {
        Some(host) => host,
        None => return Err(CoreErr { errmsg: format!("No target host ip specified"), errno: 1 })
    };
    let arg_port: u16 = match env::args().nth(2) {
        Some(port) => port.parse::<u16>().unwrap(),
        None => return Err(CoreErr { errmsg: format!("No target host port specified"), errno: 1 })
    };

    let ip_str: Vec<&str> = arg_host.as_str().split('.').collect();
    let ip_vec: Vec<u8> = ip_str.into_iter().map(|val: &str| val.parse::<u8>().unwrap()).collect();
    let ip: [u8; 4] = helpers::vec_to_arr(ip_vec);
    let addr: SocketAddr = SocketAddr::from((ip, arg_port));

    let mut stream: TcpStream = TcpStream::connect(addr).expect("Error connecting to node");
    println!("core: Connected to server");

    println!("core: Initializing thread (stdin)");
    let stream_thr_stdin: TcpStream = stream.try_clone().unwrap();
    thrstdin::init_thread(stream_thr_stdin).unwrap();

    println!("core: Initializing thread (worker)");
    let t: JoinHandle<CoreErr> = thread::Builder::new().name("thr-conn-handler".to_string()).spawn(move || {
        let ip = stream.peer_addr().unwrap().ip();
        let port: u16 = stream.peer_addr().unwrap().port();
        loop {
            let mut client_buffer: [u8; 4096] = [0u8; 4096];
            match stream.read(&mut client_buffer) {
                Ok(n) => {
                    if n == 0 {
                        println!("core: Read 0 bytes -- closing..");
                        std::process::exit(1);
                    }
                    else
                    {
                        let recv: Cow<str> = String::from_utf8_lossy(&client_buffer[..]);
                        let data: &str = recv.trim_matches(char::from(0));
                        let msg: String = format!(
                            "[{time}] -- {ip}:{port} [{bytes} bytes] :: {data}\n",
                            time=Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            ip=ip,
                            port=port,
                            bytes=n,
                            data=&data
                        );
                        io::stdout().write(msg.as_bytes()).unwrap();
                        io::stdout().flush().unwrap();
                    }
                },
                Err(e) => {
                    return CoreErr {
                        errmsg: format!("worker-thread: Error on connection handler: {}", e),
                        errno: 1
                    };
                }
            }
        }
    }).unwrap();
    t.join().unwrap();

    Ok(())
}
