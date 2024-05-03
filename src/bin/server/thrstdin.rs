use std::net::TcpStream;
use std::thread;
use crate::server::cstmconfig;
use crate::server::tcpconnection;
use std::sync::{Mutex, Arc, mpsc};
use std::sync::mpsc::TryRecvError;
use super::cstmconfig::AssetsConfig;

const IDENTIFICATOR: &str = "thrstdin";

pub fn init_thread(thrstdin_thrmain_channel_rx: Arc<Mutex<mpsc::Receiver<TcpStream>>>) -> Result<(), String> {
    match loop_user_stdin(thrstdin_thrmain_channel_rx) {
        Ok(()) => Ok(()),
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
pub fn loop_user_stdin(thrstdin_thrmain_channel_rx: Arc<Mutex<mpsc::Receiver<TcpStream>>>) -> Result<(), String> {
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
                println!("{}-threadchannel_rx: New stream received: {:?}", IDENTIFICATOR, new_stream);
                streams.push(new_stream);
            }, 
            Err(e) => {
                if e != TryRecvError::Empty {
                    println!("{}-threadchannel_rx: Error receiving new streams: {}", IDENTIFICATOR, e);
                }
            }
        }
        let server_input: String = tcpconnection::process_stdin();
        if server_input.is_empty() { continue; }
        if server_input == "exit:" {
            tcpconnection::dc_all_nodes(&streams);
            streams.clear();
            break;
        }
        else if server_input.starts_with("dc:") {
            /*
             * fetch stream by ip:port from streams - format: dc:127.0.0.1:9999
             */
            let ip_port: Vec<&str> = server_input[3..].split(":").collect::<Vec<_>>();
            let idx_to_remove: usize = tcpconnection::dc_node(&streams, ip_port).unwrap();
            if idx_to_remove != 0 {
                println!("> Garbage-collector: removing index: {}", idx_to_remove);
                streams.remove(idx_to_remove);
            }
        }
        else if server_input.starts_with("conn:") {
            /*
             * format: conn:127.0.0.1:46999
             */
            let (ip, port): (&str, &str) = match server_input[5..].split_once(":") {
                Some(ip_port) => ip_port,
                None => ("", "")
            };
            if ip == "" || port == "" {
                println!("No target ip:port specified..");
                continue;
            }
            let port_u16: u16 = port.parse::<u16>().expect("Error parsing port to u16");
            match tcpconnection::connect_client(ip, port_u16) {
                Ok(stream) => {
                    println!("Connected to node: {:?}", stream);
                    streams.push(stream);
                },
                Err(e) => {
                    println!("Error connecting to client: {}", e);
                }
            }
        }
        else if server_input == "conns:" {
            /* 
             * Remove hanging connections from list (if any) after attempting to print connections:
             * - looping indexes in reverse clears all indexes from streams since
             *   remove() works in a way that removes idx and shifts to left
             *   making next index in sequence potentially non-existing 
             */
            let indexes_to_remove: Vec<usize> = tcpconnection::print_connected(&streams);
            for i in indexes_to_remove.into_iter().rev() {
                println!("> Garbage-collector: removing index: {}", i);
                streams.remove(i);
            }
        }
        else if server_input.starts_with("sendf:") {
            /*
             * format: sendf:/home/cheki/workspace/rust-tcp-http/README.md:127.0.0.1:47074
             */
            let path_ip_port: (&str, &str) = server_input[6..].split_once(":").unwrap();
            let file_path: &str = path_ip_port.0;
            let ip_port: Vec<&str> = path_ip_port.1.split(":").collect::<Vec<_>>();
            tcpconnection::send_file(&streams, file_path, ip_port).unwrap();
        }
        else {
            let mut i: usize = 0;
            while i < streams.iter().enumerate().len() {
                if tcpconnection::send_message(&server_input,  &streams[i], &fpath) != Ok(0) {
                    streams.remove(i);
                }
                i = i+1;
            }
        }
    }).unwrap();
    Ok(())
}
