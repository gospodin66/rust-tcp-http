use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::net::Shutdown;


pub fn init_thread(stream: TcpStream) -> Result<(), String> {
    match loop_user_stdin(stream) {
        Ok(()) => { Ok(()) }
        Err(e) => {
            let err = format!("thrstdin: Write thread error: {}", e);
            println!("{}", err);
            return Err(err);
        }
    }
}

pub fn loop_user_stdin(mut stream: TcpStream) -> Result<(), String> {
    thread::Builder::new().name("thr-stdin".to_string()).spawn(move || {
        loop {

            let response: String = process_stdin();

            if response == "exit:" {
                stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
            
            match stream.write(response.as_bytes()) {
                Ok(bytes) => {
                    println!("> bytes sent: {}", bytes);
                }, 
                Err(e) => { 
                    println!("thrstdin: Error writing to stream: {:?} -- {}", stream, e);
                    break;
                }
            }
            stream.flush().unwrap();
        }
    }).unwrap();
    Ok(())
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


