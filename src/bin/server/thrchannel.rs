use std::sync::{Arc,Mutex,mpsc};
use std::net::TcpStream;

pub struct ThrChannel {
    pub tx: Arc<Mutex<mpsc::Sender<TcpStream>>>,
    pub rx: Arc<Mutex<mpsc::Receiver<TcpStream>>>
}

impl ThrChannel {
    pub fn new_channel() -> ThrChannel {
        let (tx, rx) : (mpsc::Sender<TcpStream>, mpsc::Receiver<TcpStream>) = mpsc::channel();
        return ThrChannel {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        };
    }
}

