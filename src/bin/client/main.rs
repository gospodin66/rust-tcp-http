mod thrstdin;
mod helpers;
mod cstmfiles;
mod core;
mod coreerr;
extern crate base64;

fn main() {
    match core::client() {
       Ok(()) => println!("core: Main has finsihed the job successfuly."),
       Err(e) => println!("worker-thread: Error on main: Errno {}, {}", e.errno, e.errmsg)
    }
}