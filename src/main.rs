/*
cargo run --bin multithreaded_tcp_http 
cargo run --bin client 192.168.1.61 9998
*/

#[path = "bin/server/server.rs"] mod server;
//#[path = "bin/encrypter/encrypter.rs"] mod encrypter;

fn main() {
    let server = server::Server{};
    match server.server() {
        Ok(()) => println!("main: Server closed normally."),
        Err(err) => println!("main: Fatal Error! closing server due to: {}", err)
    }
}