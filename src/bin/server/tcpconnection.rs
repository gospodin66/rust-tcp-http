use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr, TcpStream};
use super::cstmconfig::AssetsConfig;
use chrono::format::{DelayedFormat, StrftimeItems};
use chrono::Local;
use crate::server::cstmfiles;
use std::net::Shutdown;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use super::helpers;

const IDENTIFICATOR: &str = "tcpconnection";

/*
 * 
 * client -> server
 * 
 */
pub fn loop_connection(stream: &TcpStream) -> Result<(), String> {
    let (ip, port): (IpAddr, u16) = match stream.peer_addr() {
        Ok(saddr) => (saddr.ip(), saddr.port()),
        Err(e) => return Err(format!("{}: Error fetching ip:port for client: {}", IDENTIFICATOR, e))
    };
    let assets_cfg: AssetsConfig = AssetsConfig::new_cfg();
    let fpath: String = String::from(assets_cfg.log_dir+"/"+&assets_cfg.log_path);
    let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
    loop {
        let mut buffer: [u8; 4096] = [0; 4096];
        match reader.read(&mut buffer) {
            Ok(bytes) => {
                if bytes == 0 {
                    println!("tcp-handler: Empty line");
                    break;
                }
                let recv: Cow<'_, str> = String::from_utf8_lossy(&buffer[..]);
                let data: &str = recv.trim_matches(char::from(0));
                let now: DelayedFormat<StrftimeItems> = Local::now().format("%Y-%m-%d %H:%M:%S");
                let msg: String = format!("[{}] -- [{}:{}] -- [{} bytes]: {}\n", now.to_string(), ip, port, bytes, &data);
                print!("{}", &msg);
                cstmfiles::f_write(&fpath, msg).expect("Error writing file.");
            },
            Err(e) => {
                println!("tcp-handler: Error when reading line: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/*
 * 
 * server -> client
 * 
 */
pub fn send_message(server_input: &String, mut socket: &TcpStream, logfile: &String) -> Result<u8, String> {
    let (ip, port): (String, u16) = match socket.peer_addr() {
        Ok(saddr) => (saddr.ip().to_string(), saddr.port()),
        Err(e) => return Err(format!("{}: Error fetching ip:port for client: {}", IDENTIFICATOR, e))
    };
    match socket.write(server_input.as_bytes()) {
        Ok(bytes) => {
            let now: DelayedFormat<StrftimeItems> = Local::now().format("%Y-%m-%d %H:%M:%S");
            let msg = format!("[{}]: Sent to {}:{} [{} bytes] -- {}", now.to_string(), ip, port, bytes, server_input);
            cstmfiles::f_write(&logfile, msg.clone()).unwrap();
            println!("{}: {}", IDENTIFICATOR, msg);
        }, 
        Err(e) => return Err(format!("{}: Error writing to stream: {:?} -- {} -- removing from streams", IDENTIFICATOR, socket, e))
    }
    /* flush() ensures all data is written on the stream */
    match socket.flush() {
        Ok(()) => {}, 
        Err(e) => return Err(format!("{}: Error flushing stream: {}", IDENTIFICATOR, e))
    }
    Ok(0)

}

pub fn send_file(streams: &Vec<TcpStream>, file_path: &str, ip_port: Vec<&str>) -> Result<(), String>{
    let mut file: File = File::open(file_path).expect("Error opening file");
    let file_extension: &str = cstmfiles::get_extension_from_filename(file_path).unwrap();
    let init_payload: String = format!(">>>FILE_START>>>:{}\r\n", file_extension);
    let stream_completed_flag: &str = "<<<FILE_END<<<";
    let bytes_to_read_per_attempt: usize = 1024;
    for (_, mut s) in streams.iter().enumerate() {
        let (ip, port): (String, u16) = match s.peer_addr() {
            Ok(saddr) => (saddr.ip().to_string(), saddr.port()),
            Err(e) => return Err(format!("{}: Error fetching ip:port for client: {}", IDENTIFICATOR, e))
        };
        if ip == ip_port[0] && port == ip_port[1].parse::<u16>().unwrap() {
            println!("{}: Sending file [{}] to {}:{}", IDENTIFICATOR, file_path, ip_port[0], ip_port[1]);
            /* Send file-incomming flag and file extension to client */
            let mut total_bytes_read: Vec<u8> = init_payload.as_bytes().to_vec();
            let mut read_attempt_nr: i32 = 0;
            loop {
                read_attempt_nr += 1;
                println!("Read cycle {read_attempt_nr}");
                let mut cur_buffer: Vec<u8> = vec![0; bytes_to_read_per_attempt];
                let nr_of_bytes_read: usize = file.read(&mut cur_buffer).unwrap();
                if nr_of_bytes_read == 0 {
                    println!("EOF reached.");
                    break;
                }
                cur_buffer.truncate(nr_of_bytes_read);
                total_bytes_read.append(&mut cur_buffer);
                println!("Read {nr_of_bytes_read} bytes in cycle {read_attempt_nr}");
            }
            total_bytes_read.append(&mut stream_completed_flag.as_bytes().to_vec());
            let fcontents: String = String::from_utf8_lossy(&total_bytes_read[..]).to_string();
            match s.write_all(fcontents.as_bytes()) {
                Ok(()) => println!("{}: File sent to {}:{} - size: {}", IDENTIFICATOR, ip_port[0], ip_port[1], total_bytes_read.len()),
                Err(e) => println!("thrstdin: Error writing to stream: {:?} -- {}", s, e),
            }
            s.flush().unwrap();
        }
    }   
    Ok(())
}

pub fn connect_client(ip: &str, port: u16) -> Result<TcpStream, String> {
    let ip_str: Vec<&str> = ip.split('.').collect();
    let ip_vec: Vec<u8> = ip_str.into_iter().map(|val: &str| val.parse::<u8>().unwrap()).collect();
    let _ip: [u8; 4] = helpers::vec_to_arr(ip_vec);
    let addr: SocketAddr = SocketAddr::from((_ip, port));
    let stream: TcpStream = TcpStream::connect(addr).expect("Error connecting to node");
    Ok(stream)
}

pub fn process_stdin() -> String {
    let mut contents: String = String::new();
    io::stdin()
        .read_line(&mut contents)
        .expect(format!("{}: Error reading stdin", IDENTIFICATOR).as_str());
    let server_input: String = format!("{}", contents.trim());
    if server_input.is_empty() {
        let err: String = format!("{}: empty input", IDENTIFICATOR);
        println!("{}", err);
        return String::new()
    }
    return server_input
}

pub fn print_connected(streams: &Vec<TcpStream>) -> Vec<usize> {
    println!("\nConnected streams:");
    let mut streams_to_rm: Vec<usize> = Vec::new();
    for (i, s) in streams.iter().enumerate() {
        let (ip, port): (String, u16) = match s.peer_addr() {
            Ok(saddr) => (saddr.ip().to_string(), saddr.port()),
            Err(e) => {
                println!("{}: Error fetching ip:port for client: {}", IDENTIFICATOR, e);
                (String::from(""), 0)
            }
        };
        if ip == String::from("") || port == 0 {
            println!("> Garbage-collector: index {} added for removal.", i);
            streams_to_rm.push(i);
        } else {
            println!("{} -- {}:{}", i, ip, port);
        }
    }
    println!("");
    streams_to_rm
}

pub fn dc_all_nodes(streams: &Vec<TcpStream>) {
    for (_, stream) in streams.iter().enumerate() {
        stream.shutdown(Shutdown::Both).unwrap();
    }
}

pub fn dc_node(streams: &Vec<TcpStream>, ip_port: Vec<&str>) -> Result<usize, String> {
    let mut idx: usize = 0;
    for (i, s) in streams.iter().enumerate() {
        let (ip, port): (String, u16) = match s.peer_addr() {
            Ok(saddr) => (saddr.ip().to_string(), saddr.port()),
            Err(e) => return Err(format!("{}: Error fetching ip:port for client: {}", IDENTIFICATOR, e))
        };
        if ip == ip_port[0] && port == ip_port[1].parse::<u16>().unwrap() {
            s.shutdown(Shutdown::Both).unwrap();
            idx = i;
            break;
        }
    }
    Ok(idx)
}
