use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use chrono::Local;
use crate::server::{cstmfiles, cstmconfig};
use std::sync::{Mutex, Arc, mpsc};
use std::sync::mpsc::TryRecvError;
use std::net::{Shutdown};

use super::cstmconfig::AssetsConfig;

pub fn init_thread(
    thrstdin_thrmain_channel_rx: Arc<Mutex<mpsc::Receiver<TcpStream>>>
) -> Result<(), String> {
    match loop_user_stdin(thrstdin_thrmain_channel_rx) {
        Ok(()) => { Ok(()) }
        Err(e) => {
            let err: String = format!("thrstdin: Write thread error: {}", e);
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

                /*
                

                    TODO: send stream to threadpool list (does not exist)
                
                
                */


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
            break;
        }
        else if response == "dc:<ip>:<port>" {
            // fetch stream by ip:port from streams
            //dc_node(&stream);
            break;
        }
        else if response == "conn:" {
            print_connected(&streams);
            continue;
        }
        else if response == "sendf:" {
            send_file(&streams);
            continue;
        }

        for (i, mut s) in streams.iter().enumerate() {
            match s.write(response.as_bytes()) {
                Ok(bytes) => {
                    let msg = format!(
                        "[{}]: Sent to {:?} [{} bytes] -- {}",
                        Local::now().to_rfc3339(),
                        s,
                        bytes,
                        response
                    );
                    cstmfiles::f_write(&fpath, msg).unwrap();
                    println!("thrstdin: Sent to {:?} [{} bytes] -- {}", s, bytes, response);
                }, 
                Err(e) => { 
                    println!("thrstdin: Error writing to stream: {:?} -- {} -- removing from streams", s, e);
                    streams.remove(i);
                    break;
                }
            }
            s.flush().unwrap();
        }    
    }).unwrap();
    Ok(())
}


fn dc_all_nodes(streams: &Vec<TcpStream>) {
    for stream in streams.iter() {
        stream.shutdown(Shutdown::Both).unwrap();
    }
}

fn send_file(streams: &Vec<TcpStream>) {
    /*
    - get file
    - convert to bytes
    - send bytes
    -------------------
    client assembles bytes
     */ 
    for _node in streams.iter() {

    }
}




fn process_stdin() -> String {
    let mut contents: String = String::new();
    io::stdin()
        .read_line(&mut contents)
        .expect("thrstdin: Error reading stdin");
    let response: String = format!("{}", contents.trim());
    if response.is_empty() {
        let err: String = format!("thrstdin: empty response");
        println!("{}", err);
        return String::new()
    }
    return response
}


fn print_connected(streams: &Vec<TcpStream>) {
    println!("\nConnected streams:");
    for (i, s) in streams.iter().enumerate() {
        println!("{} -- {:?}", i, s);
    }
    println!("");
}