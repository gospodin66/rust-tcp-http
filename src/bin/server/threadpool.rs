use std::borrow::Cow;
use std::io::prelude::*;
#[allow(unused_imports)]
use std::net::{TcpStream, TcpListener, Shutdown, IpAddr};
use std::sync::{Mutex, Arc};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use chrono::Local;
use crate::server::thrchannel::{self, ThrChannel};
use crate::server::{response, cstmfiles, cstmconfig::AssetsConfig};
use crate::server::{thrstdin, validator};

/*
 * 1. The ThreadPool will create a channel and hold on to the sending side of the channel.
 * 2. Each Worker will hold on to the receiving side of the channel.
 * 3. We’ll create a new Job struct that will hold the closures we want to send down the channel.
 * 4. The execute method will send the job it wants to execute down the sending side of the channel.
 * 5. In its thread, the Worker will loop over its receiving side of the channel and execute the closures of any jobs it receives.
 */
static THREAD_LIMIT : usize = 10;
const IDENTIFICATOR: &str = "threadpool";
type Job = Box<dyn FnOnce() + Send + 'static>;

#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    tx: mpsc::Sender<Job>
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (tx,rx) = mpsc::channel();
        let rx: Arc<Mutex<mpsc::Receiver<Box<dyn FnOnce() + Send>>>> = Arc::new(Mutex::new(rx));
        let mut workers: Vec<Worker> = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }
        ThreadPool { workers, tx }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job: Box<F> = Box::new(f);
        self.tx.send(job).unwrap();
    }

}

/*
 * Worker is responsible for taking jobs and exec them 
 */
#[allow(dead_code)]
pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thr: JoinHandle<()> = 
            thread::Builder
                ::new()
                .name(String::from("thr-worker"))
                .spawn(move || loop {
                    // retrieve job from channel
                    let job: Box<dyn FnOnce() + Send> = rx.lock().unwrap().recv().unwrap();
                    println!("-----------------------------------------");
                    println!("{}: Worker {} got a new job!", IDENTIFICATOR, id);
                    job();
                }).unwrap();
        Worker {
            id,
            thread: thr,
        }
    }
}

pub fn handle_in_threadpool(
    listener: &TcpListener,
) -> Result<(), String> {
    let pool: ThreadPool = ThreadPool::new(THREAD_LIMIT);
    println!("{}: Initializing thread channel.", IDENTIFICATOR);
    let thrstdin_thrmain_channel: ThrChannel = thrchannel::ThrChannel::new_channel();
    println!("{}: Initializing input thread.", IDENTIFICATOR);
    thrstdin::init_thread(thrstdin_thrmain_channel.rx).unwrap();
    for s in listener.incoming() {
        match s {
            Ok(stream) => {
                let thrstdin_thrmain_channel_tx_clone: Arc<Mutex<mpsc::Sender<TcpStream>>> = thrstdin_thrmain_channel.tx.clone();
                pool.execute(move || {
                    match handle_connection(stream, thrstdin_thrmain_channel_tx_clone) {
                        Ok(()) => {},
                        Err(e) => println!("{}: Error on connection handler: {}", IDENTIFICATOR, e)
                    }
                });
            },
            Err(e) => {
                return Err(format!("{}: Error on creating stream: {}", IDENTIFICATOR, e));
            }
        }
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, thrstdin_thrmain_channel_tx: Arc<Mutex<mpsc::Sender<TcpStream>>>) -> Result<(), String> {
    let ip: IpAddr = stream.peer_addr().expect(format!("{}: failed to parse IP address from stream", IDENTIFICATOR).as_str()).ip();
    let port: u16 = stream.peer_addr().expect(format!("{}: failed to parse port from stream", IDENTIFICATOR).as_str()).port();
    let stream_clone: TcpStream = stream.try_clone().expect(format!("{}: clone-stream failed...", IDENTIFICATOR).as_str());
    let mut buffer: [u8; 4096] = [0; 4096];
    let mut _data: String = String::new();
    /*
     * peek() - wait until client sends first packet
     */
    println!("\nReceived connection from {}:{}", ip, port);
    match stream.peek(&mut buffer) {
        Ok(bytes) => {
            _data = String::from_utf8_lossy(&buffer[..]).trim_matches(char::from(0)).to_string();
            println!("Connection [{}:{}]:\n>>> bytes: {}[b]\n>>> data: {:?}", ip, port, bytes, _data);
        }
        Err(e) => return Err(format!("{}: Error recieving data: {}", IDENTIFICATOR, e))
    }
    match validator::validate_http_request(&_data) {
        Ok(_http_request) => {
            /* HTTP request - do not send new connection to thread-stdin */
            println!(">>> {}: Handling HTTP response {}:{}\n>>>\n", IDENTIFICATOR, &ip, &port);
            match response::write_http_response(&stream, &_data) {
                Ok(()) => {}, 
                Err(e) => println!("{}: Error sending html response to {}:{}: {}", IDENTIFICATOR, ip, port, e)
            }
            println!("\n<<<");
            /* disconnect HTTP connection after serving content */
            match stream.shutdown(Shutdown::Both) {
                Ok(()) => println!("<<< HTTP connection closed."),
                Err(e) => println!("shutdown() call failed: {}", e)
            }
        },
        _ => {
            /* default TCP request - send new connection to thread-stdin */
            match thrstdin_thrmain_channel_tx.lock().unwrap().send(stream) {
                Ok(()) => println!(">>> threadpool-threadchannel_tx: Transmitter sent new stream to thrstdin"), 
                Err(e) => println!("threadpool-threadchannel_tx: Error sending new stream to thrstdin on listener: {}", e)
            }
            println!(">>> {}: Handling TCP connection {}:{}", IDENTIFICATOR, &ip, &port);
            match loop_connection(&stream_clone) {
                Ok(()) => {},
                Err(e) => println!("{}: Error handling tcp connection to {}:{}: {}", IDENTIFICATOR, ip, port, e)
            }
        }
    }

    Ok(())
}

fn loop_connection(mut stream: &TcpStream) -> Result<(), String> {
    let ip: IpAddr = stream.peer_addr().unwrap().ip();
    let port: u16 = stream.peer_addr().unwrap().port();
    let assets_cfg: AssetsConfig = AssetsConfig::new_cfg();
    let fpath: String = String::from(assets_cfg.log_dir+"/"+&assets_cfg.log_path);
    loop {
        let mut buffer: [u8; 4096] = [0; 4096];
        match stream.read(&mut buffer) {
            Ok(bytes) => {
                if bytes == 0 {
                    println!("tcp-handler: Empty line");
                    break;
                }
                let recv: Cow<'_, str> = String::from_utf8_lossy(&buffer[..]);
                let data: &str = recv.trim_matches(char::from(0));
                let msg: String = format!(
                    "[{}] -- [{}:{}] -- [{} bytes]: {}\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    ip,
                    port,
                    bytes,
                    &data
                );
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

