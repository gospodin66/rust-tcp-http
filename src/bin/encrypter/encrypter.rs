extern crate openssl;

use aes_gcm::Nonce;
use aes_gcm::aead::consts::{B0, B1};
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aes::cipher::typenum::UInt;
use aes_gcm::aes::cipher::typenum::UTerm;
use std::fs;
use std::process;
use openssl::rsa::Rsa;

use crate::cstmfiles;
use crate::encrypteraes;
use crate::encrypterrsa;

pub struct Encrypter {}

#[allow(dead_code)]
impl Encrypter {

    pub fn encrypter(&self, method: &str, data: &[u8], mode: String, passphrase: String, keypair_id: u32) -> u8 {
        if method == "AES" {
            /*
            * 
            * TODO:: IMPLEMENT AES KEY STORAGE ( directory "./AES-keys/" )
            *        IMPLEMENT AES256GCM ENCRYPTION OVER RSA KEYPAIRS
            *        - A sends public key to B
            *        - B generates session key & encrypts with public key => sends to A
            *        - A decrypts session key with private key
            *        - session key is used to encrypt communication with AES
            *        - messages are sent encrypted with aes key - infinite length as RSA has max msg len = key len
            * 
            * 
            */
            // 96-bits; unique per message
            let nonce: &GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>> = Nonce::from_slice(b"unique nonce"); 
            self.encrypter_aes(nonce, data);
            return 0;
        }
        else if method == "RSA" {
            self.encrypter_rsa(mode, &passphrase, keypair_id, data);
            return 0;
        }
        else {
            println!("Invalid action (should use AES|RSA)");
        }
        return 1;
    }


    fn encrypter_aes(&self, nonce: &GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>, data: &[u8]) {
        let aes = encrypteraes::encrypteraes::AesEncrypter::construct();
        let aes_clone = aes.clone();
        let ciphertext = aes.encrypt(nonce, data);
        println!("Encrypted: {:?}", ciphertext);

        let decrypted = aes_clone.decrypt(nonce, &ciphertext);
        println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
    }


    fn encrypter_rsa(&self, mode: String, passphrase: &String, keypair_id: u32, data: &[u8]) {
        let base_keys_dir = "src/bin/encrypter/RSA-keys";
        let mut keypair_path = format!("{}/{}", base_keys_dir, keypair_id);

        let mut private_path = format!("{}/private.pem", &keypair_path);
        let mut public_path = format!("{}/public.pem", &keypair_path);

        if fs::metadata(&keypair_path).is_ok() {
            println!("Keypair exists: {}", &keypair_path);
        } else {

            let new_keypair_id = rand::random::<u32>();
            println!("Keypair does not exist: {} -- creating new [{}]", &keypair_path, &new_keypair_id);

            keypair_path = format!("{}/{}", base_keys_dir, new_keypair_id);
            private_path = format!("{}/private.pem", &keypair_path);
            public_path = format!("{}/public.pem", &keypair_path);

            match cstmfiles::cstmfiles::create_dir(&keypair_path) {
                Ok(()) => {},
                Err(_err) => {
                    process::exit(0x0100);
                }
            }
            
            match encrypterrsa::encrypterrsa::create_keypair(&passphrase, &public_path, &private_path) {
                Ok(()) => println!("Keypair created successfuly"),
                Err(e) => println!("Error creating keypair: {}", e)
            }
        }

        let private_key_pem = match cstmfiles::cstmfiles::read(&private_path) {
            Ok(privkey) => privkey,
            Err(e) => e 
        };
        let public_key_pem = match cstmfiles::cstmfiles::read(&public_path) {
            Ok(pubkey) => pubkey,
            Err(e) => e 
        };

        let rsa_pub = match Rsa::public_key_from_pem(public_key_pem.as_bytes()) {
            Ok(pubkey) => pubkey,
            _ => panic!("Error: fetching public key from pem")
        };
        let rsa_priv = match Rsa::private_key_from_pem_passphrase(
            private_key_pem.as_bytes(), passphrase.as_bytes()
        ) {
            Ok(privkey) => privkey,
            _ => panic!("Error: fetching private key from passphrase.")
        };


        if mode == "public" {
            let encrypted_bytes = encrypterrsa::encrypterrsa::encrypt_public(
                format!("{}",std::str::from_utf8(&data).unwrap()), rsa_pub
            ).unwrap();
            println!("Encrypted bytes: {:?}", &encrypted_bytes);
            let decrypted_message = encrypterrsa::encrypterrsa::decrypt_public(encrypted_bytes, rsa_priv).unwrap();
            println!("Decrypted message: {:?}", String::from_utf8(decrypted_message).unwrap());
        }
        else if mode == "private" {
            let encrypted_bytes = encrypterrsa::encrypterrsa::encrypt_private(
                format!("{}",std::str::from_utf8(&data).unwrap()), rsa_priv
            ).unwrap();
            println!("Encrypted bytes: {:?}", &encrypted_bytes);
            let decrypted_message = encrypterrsa::encrypterrsa::decrypt_private(encrypted_bytes, rsa_pub).unwrap();
            println!("Decrypted message: {:?}", String::from_utf8(decrypted_message).unwrap());

        } else {
            println!("Invalid mode (try private|public)");
            process::exit(0x0100);
        }
        
    }
}
