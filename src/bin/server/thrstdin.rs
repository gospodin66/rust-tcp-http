use std::net::{TcpStream, SocketAddr};
use std::thread;
use chrono::Local;
use crate::server::{cstmfiles, cstmconfig};
use std::sync::{Mutex, Arc, mpsc};
use std::sync::mpsc::TryRecvError;
use std::net::Shutdown;
use std::fs::File;
use std::io::{self, Read, Write};
use super::cstmconfig::AssetsConfig;
use super::helpers;

const IDENTIFICATOR: &str = "thrstdin";


pub fn init_thread(
    thrstdin_thrmain_channel_rx: Arc<Mutex<mpsc::Receiver<TcpStream>>>
) -> Result<(), String> {

    match loop_user_stdin(thrstdin_thrmain_channel_rx) {
        Ok(()) => { Ok(()) }
        Err(e) => {
            let err: String = format!("{}: Write thread error: {}", IDENTIFICATOR, e);
            println!("{}", err);
            return Err(err);
        }
    }
}
/*
 * Uses thrstdin_thrmain_channel to communicate with stdin thread
 *      -- send new connections
 *      -- send closed connections
 *      -- synchronize with input thread
 */
pub fn loop_user_stdin(
    thrstdin_thrmain_channel_rx: Arc<Mutex<mpsc::Receiver<TcpStream>>>
) -> Result<(), String> {
    /*
     * Using scopes guarantees to terminate before the scope exits,
     * allowing it to reference variables outside the scope.
     *    -- move -> give ownership to a thread
     */
    let mut streams: Vec<TcpStream> = Vec::new();
    let assets_cfg: AssetsConfig = cstmconfig::AssetsConfig::new_cfg();
    let fpath: String = String::from(assets_cfg.log_dir+"/"+&assets_cfg.log_path);

    thread::Builder::new()
      .name("thr-stdin".to_string())
      .spawn(move || loop {
        /*
         * Fetch new connections from thrstdin_thrmain_channel here 
         *    -- rx.try_recv() -> for non-blocking
         */
        match thrstdin_thrmain_channel_rx.lock().unwrap().try_recv() {
            Ok(new_stream) => {
                println!("thrstdin-threadchannel_rx: New stream received: {:?}", new_stream);
                streams.push(new_stream);
            }, 
            Err(e) => {
                if e != TryRecvError::Empty {
                    println!("thrstdin-threadchannel_rx: Error receiving new streams: {}", e);
                }
            }
        }
        let response = process_stdin();
        if response.is_empty() {
            continue;
        }
        if response == "exit:" {
            dc_all_nodes(&streams);
            streams.clear();
            break;
        }
        else if response.starts_with("dc:") {
            /*
             * format: dc:127.0.0.1:9999
             */
            // fetch stream by ip:port from streams
            let ip_port = response[3..].split(":").collect::<Vec<_>>();
            let idx_to_remove = dc_node(&streams, ip_port).unwrap();
            if idx_to_remove != 0 {
                print!("REMOVING IDX/IPPORT: {}", idx_to_remove);
                streams.remove(idx_to_remove);
            }
        }
        else if response.starts_with("conn:") {
            /*
             * format: conn:127.0.0.1:46999
             */
            let (ip, port): (&str, &str) = match response[5..].split_once(":") {
                Some(ip_port) => ip_port,
                None => ("", "")
            };
            if ip == "" || port == "" {
                println!("No target ip:port specified..");
                continue;
            }

            let port_u16: u16 = port.parse::<u16>().expect("Error parsing port to u16");

            match connect_client(ip, port_u16) {
                Ok(stream) => {
                    println!("Connected to node: {:?}", stream);
                    streams.push(stream);
                },
                Err(e) => {
                    println!("Error connecting to client: {}", e);
                }
            }
        }
        else if response == "conns:" {
            print_connected(&streams);
        }
        else if response.starts_with("sendf:") {
            /*
             * format: sendf:/home/cheki/workspace/rust-tcp-http/README.md:127.0.0.1:47074
             */
            let path_ip_port = response[6..].split_once(":").unwrap();
            let file_path = path_ip_port.0;
            let ip_port = path_ip_port.1.split(":").collect::<Vec<_>>();

            send_file(&streams, file_path, ip_port).unwrap();
        }
        else {
            let mut i: usize = 0;
            while i < streams.iter().enumerate().len() {
                if send_response(&response,  &streams[i], &fpath) != 0 {
                    streams.remove(i);
                }
                i = i+1;
            }
        }
    }).unwrap();
    Ok(())
}




fn connect_client(ip: &str, port: u16) -> Result<TcpStream, String> {
    let ip_str: Vec<&str> = ip.split('.').collect();
    let ip_vec: Vec<u8> = ip_str.into_iter().map(|val: &str| val.parse::<u8>().unwrap()).collect();
    let _ip: [u8; 4] = helpers::vec_to_arr(ip_vec);
    let addr: SocketAddr = SocketAddr::from((_ip, port));
    let stream: TcpStream = TcpStream::connect(addr).expect("Error connecting to node");
    Ok(stream)
}




fn send_response(
    response: &String, 
    mut socket: &TcpStream, 
    logfile: &String,
) -> u8 {
    let ip: String = match socket.peer_addr() {
        Ok(ip) => ip.ip().to_string(),
        Err(e) => {
            println!("{}: Error fetching client IP address: {}", IDENTIFICATOR, e);
            String::from("")
        }
    };
    let port = match socket.peer_addr() {
        Ok(port) => port.port(),
        Err(e) => {
            println!("{}: Error fetching client port: {}", IDENTIFICATOR, e);
            0
        }
    };
    if ip == String::from("") || port == 0 {
        return 1
    }

    match socket.write(response.as_bytes()) {
        Ok(bytes) => {
            let msg = format!(
                "[{time}]: Sent to {ip}:{port} [{bytes} bytes] -- {response}",
                time=Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                ip=ip,
                port=port,
                bytes=bytes,
                response=response
            );
            cstmfiles::f_write(&logfile, msg.clone()).unwrap();
            println!("{}: {}", IDENTIFICATOR, msg);
        }, 
        Err(e) => { 
            println!("{}: Error writing to stream: {:?} -- {} -- removing from streams", IDENTIFICATOR, socket, e);
            return 1
        }
    }
    // flush() ensures all data is written on the stream
    match socket.flush() {
        Ok(()) => {}, 
        Err(e) => {
            println!("{}: http-response: Error flushing stream: {}", IDENTIFICATOR, e);
        }
    }
    return 0

}


fn dc_all_nodes(streams: &Vec<TcpStream>) {
    for (_, stream) in streams.iter().enumerate() {
        stream.shutdown(Shutdown::Both).unwrap();
    }
}


fn dc_node(streams: &Vec<TcpStream>, ip_port: Vec<&str>) -> Result<usize, String> {
    let mut idx = 0;
    for (i, s) in streams.iter().enumerate() {
        if s.peer_addr().unwrap().ip().to_string() == ip_port[0] 
        && s.peer_addr().unwrap().port() == ip_port[1].parse::<u16>().unwrap() {
            s.shutdown(Shutdown::Both).unwrap();
            idx = i;
        }
    }
    Ok(idx)
}


fn send_file(streams: &Vec<TcpStream>, file_path: &str, ip_port: Vec<&str>) -> Result<(), String>{
    let mut file = File::open(file_path).expect("Error opening file");

    for (_, mut s) in streams.iter().enumerate() {
        if s.peer_addr().unwrap().ip().to_string() == ip_port[0] 
        && s.peer_addr().unwrap().port() == ip_port[1].parse::<u16>().unwrap()
        {
            println!("{}: Sending file [{}] to {}:{}", IDENTIFICATOR, file_path, ip_port[0], ip_port[1]);
            let mut buf = [0; 4096];
            loop {
                let n = file.read(&mut buf).unwrap();
                if n == 0 {
                    // reached end of file
                    break;
                }
                s.write_all(&buf[..n]).unwrap();
            }
            println!("{}: File sent to {}:{}", IDENTIFICATOR, ip_port[0], ip_port[1]);
        }
    }   
    Ok(())
}


fn process_stdin() -> String {
    let mut contents: String = String::new();
    io::stdin()
        .read_line(&mut contents)
        .expect(format!("{}: Error reading stdin", IDENTIFICATOR).as_str());
    let response: String = format!("{}", contents.trim());
    if response.is_empty() {
        let err: String = format!("{}: empty response", IDENTIFICATOR);
        println!("{}", err);
        return String::new()
    }
    return response
}


fn print_connected(streams: &Vec<TcpStream>) {
    println!("\nConnected streams:");
    for (i, s) in streams.iter().enumerate() {

        let ip: String = match s.peer_addr() {
            Ok(ip) => ip.ip().to_string(),
            Err(e) => {
                println!("{}: Error fetching client IP address: {}", IDENTIFICATOR, e);
                String::from("")
            }
        };
        let port = match s.peer_addr() {
            Ok(port) => port.port(),
            Err(e) => {
                println!("{}: Error fetching client port: {}", IDENTIFICATOR, e);
                0
            }
        };

        if ip == String::from("") || port == 0 {
            println!("Client [{}-{}:{}] not connected?", i, ip, port);
        } else {
            println!("{} -- {}:{}", i, ip, port);
        }

    }
    println!("");
}