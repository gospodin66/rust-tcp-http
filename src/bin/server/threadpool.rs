use std::net::{TcpStream, TcpListener, Shutdown, IpAddr};
use std::sync::{Mutex, Arc};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use crate::server::{httpconnection, tcpconnection};
use crate::server::{thrstdin, validator};
use crate::server::thrchannel::{self, ThrChannel};

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
                    let job = match rx.lock() {
                        Ok(mutex_guard) => match mutex_guard.recv() {
                            Ok(job) => {
                                println!("-----------------------------------------");
                                println!("{}: Worker {} got a new job!", IDENTIFICATOR, id);
                                job
                            },
                            Err(e) => {
                                println!("Error: Failed receiving init data from client: {}", e);
                                return ()
                            }
                        },
                        Err(e) => {
                            println!("Error: Failed receiving init data from client: {}", e);
                            return ()
                        }
                    };
                    job();
                }).unwrap();
        Worker { id, thread: thr}
    }
}

pub fn handle_in_threadpool(listener: &TcpListener) -> Result<(), String> {
    let pool: ThreadPool = ThreadPool::new(THREAD_LIMIT);
    println!("{}: Initializing thread channel.", IDENTIFICATOR);
    let thrstdin_thrmain_channel: ThrChannel = thrchannel::ThrChannel::new_channel();
    println!("{}: Initializing input thread.", IDENTIFICATOR);
    match thrstdin::init_thread(thrstdin_thrmain_channel.rx) {
        Ok(()) => {},
        Err(e) => return Err(format!("Error: failed to initialize thread: {}", e))
    }
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
            Err(e) => { return Err(format!("{}: Error on creating stream: {}", IDENTIFICATOR, e)); }
        }
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, thrstdin_thrmain_channel_tx: Arc<Mutex<mpsc::Sender<TcpStream>>>) -> Result<(), String> {
    let (ip, port): (IpAddr, u16) = match stream.peer_addr() {
        Ok(saddr) => (saddr.ip(), saddr.port()),
        Err(e) => return Err(format!("{}: Error fetching ip:port for client: {}", IDENTIFICATOR, e))
    };
    let stream_clone: TcpStream = match stream.try_clone() {
        Ok(tcp_stream_clone) => tcp_stream_clone,
        Err(e) => return Err(format!("{}: Error clonning TcpStream: {}", IDENTIFICATOR, e))
    };
    let mut buffer: [u8; 4096] = [0; 4096];
    let mut _data: String = String::new();
    
    println!("\nReceived connection from {}:{}", ip, port);
    /* peek() - wait until client sends first packet */
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
            match httpconnection::write_http_response(&stream, &_data) {
                Ok(()) => {}, 
                Err(e) => println!("{}: Error sending html response to {}:{}: {}", IDENTIFICATOR, ip, port, e)
            }
            println!("\n<<<");
            /* disconnect HTTP connection after serving content */
            match stream.shutdown(Shutdown::Both) {
                Ok(()) => println!("<<< HTTP connection [{}:{}] closed.", ip, port),
                Err(e) => println!("shutdown() call failed on HTTP connection: {}", e)
            }
        },
        _ => {
            /* default TCP request - send new connection to thread-stdin */
            println!(">>> {}: Handling TCP connection {}:{}>>>\n", IDENTIFICATOR, &ip, &port);
            match thrstdin_thrmain_channel_tx.lock().unwrap().send(stream) {
                Ok(()) => println!(">>> threadpool-threadchannel_tx: Transmitter sent new stream to thrstdin"), 
                Err(e) => println!("threadpool-threadchannel_tx: Error sending new stream to thrstdin on listener: {}", e)
            }
            match tcpconnection::loop_connection(&stream_clone) {
                Ok(()) => {},
                Err(e) => println!("{}: Error handling tcp connection from {}:{}: {}", IDENTIFICATOR, ip, port, e)
            }
            /* attempt to disconnect TCP connection after loop exits */
            match stream_clone.shutdown(Shutdown::Both) {
                Ok(()) => println!("<<< TCP connection [{}:{}] closed.", ip, port),
                Err(e) => println!("shutdown() call failed on TCP connection: {}", e)
            }
        }
    }

    Ok(())
}
