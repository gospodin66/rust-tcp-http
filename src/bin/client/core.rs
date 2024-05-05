#![allow(unused_labels)]
use std::borrow::Cow;
use std::fs::File;
/*
 * We bring std::io::prelude into scope to get access to certain
 * traits that let us read from and write to the stream
 */
use std::net::{TcpStream,SocketAddr};
use std::net::IpAddr;
use std::env;
use std::thread::{self, JoinHandle};
use std::io::{self, Read};
use std::io::Write;
use chrono::Local;
extern crate base64;
use crate::helpers;
use crate::thrstdin;
use crate::coreerr::CoreErr;

#[allow(dead_code)]
fn connect_server(ip: &str, port: u16) -> Result<TcpStream, String> {
    let ip_str: Vec<&str> = ip.split('.').collect();
    let ip_vec: Vec<u8> = ip_str.into_iter().map(|val: &str| val.parse::<u8>().unwrap()).collect();
    let _ip: [u8; 4] = helpers::vec_to_arr(ip_vec);
    let addr: SocketAddr = SocketAddr::from((_ip, port));
    let stream: TcpStream = TcpStream::connect(addr).expect("Error connecting to node");
    Ok(stream)
}

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
        let ip: IpAddr = stream.peer_addr().unwrap().ip();
        let port: u16 = stream.peer_addr().unwrap().port();
        let stream_start_flag: &str = ">>>FILE_START>>>:";
        let stream_completed_flag: &str = "<<<FILE_END<<<";
        let bytes_to_read_per_attempt: usize = 1024;
        loop {
            let mut client_buffer: [u8; 1024] = [0u8; 1024];
            let now: String = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            match stream.read(&mut client_buffer) {
                Ok(n) => {
                    if n == 0 {
                        println!("core: Read 0 bytes -- closing..");
                        std::process::exit(1);
                    } else {
                        let recv: Cow<str> = String::from_utf8_lossy(&client_buffer[..]);
                        let mut data: String = recv.trim_matches(char::from(0)).to_string();
                        // File transfer block
                        if data.contains(stream_start_flag) {
                            data = data.replace(stream_start_flag, "");
                            let file_extension: &str = &data[0..data.find("\r\n").unwrap()];
                            let f_path: String = format!("recv.{file_extension}");
                            let mut total_bytes_read: Vec<u8> = vec![];
                            let mut read_attempt_nr: i32 = 0;
                            println!("> File transfer initiated!");
                            println!("> Downloading file: {}", f_path.as_str());
                            stream.flush().unwrap();
                            loop {
                                read_attempt_nr += 1;
                                println!("Read cycle {read_attempt_nr}");
                                let mut cur_buffer: Vec<u8> = vec![0; bytes_to_read_per_attempt];
                                let nr_of_bytes_read: usize = match stream.read(&mut cur_buffer) {
                                    Ok(nr_of_bytes_read) => nr_of_bytes_read,
                                    Err(err) => {
                                        if err.kind() == io::ErrorKind::WouldBlock || err.kind() == io::ErrorKind::TimedOut {
                                            println!("Read attempt timed out");
                                            break;
                                        } else {
                                            println!("Error: failed to read bytes from stream: {}", err);
                                            break;
                                        }
                                    }
                                };
                                if nr_of_bytes_read == 0 {
                                    println!("Read zero bytes - Connection seems closed");
                                    break;
                                }
                                cur_buffer.truncate(nr_of_bytes_read);
                                let mut __data: String = String::from_utf8_lossy(&cur_buffer).to_string();
                                if __data.contains(stream_completed_flag) {
                                    println!(">>> File transfer completed: Reached EOF flag: {}", stream_completed_flag);
                                    // Drop stream_completed_flag from input
                                    __data = __data.as_str().replace(stream_completed_flag, "");
                                    total_bytes_read.append(&mut __data.as_bytes().to_vec());
                                    break;
                                } else {
                                    total_bytes_read.append(&mut cur_buffer);
                                }
                                println!("Read {nr_of_bytes_read} bytes in cycle {read_attempt_nr}");
                            }
                            println!("Total bytes read: {}", total_bytes_read.len());
                            let fcontents: String = String::from_utf8_lossy(&total_bytes_read[..]).to_string();
                            let fc_with_nl: String = fcontents + "\r\n";
                            let mut __file: File = match File::create(f_path.as_str()) {
                                Ok(file) => file,
                                Err(e) => { 
                                    let err = format!("Error: Failed to create file: {}", e);
                                    println!("{}", err);
                                    return CoreErr { errmsg: err, errno: 1 }; 
                                }
                            };
                            __file.write_all(fc_with_nl.as_bytes()).unwrap();
                            __file.sync_all().unwrap();
                            __file.flush().unwrap();
                            println!("Successfuly written data to file.");
                        } else {
                            // Default write to stdout
                            let msg: String = format!("[{}] -- {}:{} [{} bytes] :: {}\n", now, ip, port, n, &data);
                            io::stdout().write(msg.as_bytes()).unwrap();
                            io::stdout().flush().unwrap();
                        }
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
