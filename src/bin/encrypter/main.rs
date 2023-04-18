use std::env;
mod encrypter;
mod encrypterrsa;
mod encrypteraes;
mod cstmfiles;

/*
 * cargo run --bin encrypter "RSA" "This is secret data." "public" "pass123" "2395208055"
 */

fn main() {
    let args: Vec<String> = env::args().collect();
    let method: &str = args[1].as_str();
    let data: &[u8] = args[2].as_bytes();
    let mode: String = String::from(&args[3]);
    let passphrase: String = String::from(&args[4]);
    let keypair_id: u32 = args[5].parse::<u32>().unwrap();
    let crypt = encrypter::Encrypter {};
    let res: u8 = crypt.encrypter(method, data, mode, passphrase, keypair_id);
    if res == 0 {
        println!("core: Main has finsihed the job successfuly -- res: {}", res);
    } else {
        println!("core: Main has finsihed with error -- res: {}", res);
    }
}