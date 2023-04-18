pub mod encrypterrsa {
    use openssl::pkey::{Private, Public};
    use openssl::symm::Cipher;
    use openssl::rsa::{Rsa, Padding};

    use crate::cstmfiles::cstmfiles;


    pub fn create_keypair(passphrase: &String, public_path: &String, private_path: &String) -> Result<(), String> {

        let bits = 1024;
        let rsa = Rsa::generate(bits).unwrap();
        let private_key: Vec<u8> = rsa.private_key_to_pem_passphrase(
            Cipher::aes_256_cbc(), 
            passphrase.as_bytes()
        ).unwrap();
        let public_key: Vec<u8> = rsa.public_key_to_pem().unwrap();
        let private_key_str = String::from_utf8(private_key).unwrap();
        let public_key_str = String::from_utf8(public_key).unwrap();

        match cstmfiles::create(&private_path) {
            Ok(()) => println!("File created successfuly."),
            Err(_e) => {}
        }
        match cstmfiles::create(&public_path) {
            Ok(()) => println!("File created successfuly."),
            Err(_e) => {}
        }

        match cstmfiles::write(&public_path, public_key_str) {
            Ok(()) => println!("Successfuly created public key."),
            Err(_err) => {}
        }
        match cstmfiles::write(&private_path, private_key_str) {
            Ok(()) => println!("Successfuly created private key."),
            Err(_err) => {}
        }

        Ok(())

    }


    pub fn encrypt_public(
        data: String,
        rsa_pub: Rsa<Public>,
    ) -> Result<Vec<u8>, String>
    {
        let mut buf: Vec<u8> = vec![0; rsa_pub.size() as usize];
        let _ = rsa_pub.public_encrypt(data.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
        //let encrypted_raw_b64 = base64::encode(String::from_utf8_lossy(&buf).as_bytes());

        Ok(buf)
    }


    pub fn decrypt_public(
        data: Vec<u8>,
        rsa_priv: Rsa<Private>,
    ) -> Result<Vec<u8>, String>
    {
        let mut buf: Vec<u8> = vec![0; rsa_priv.size() as usize];
        let _ = rsa_priv.private_decrypt(&data, &mut buf, Padding::PKCS1).unwrap();

        Ok(buf)
    }



    pub fn encrypt_private(
        data: String,
        rsa_priv: Rsa<Private>,
    ) -> Result<Vec<u8>, String>
    {
        let mut buf: Vec<u8> = vec![0; rsa_priv.size() as usize];
        let _ = rsa_priv.private_encrypt(data.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
        //let encrypted_raw_b64 = base64::encode(String::from_utf8_lossy(&buf).as_bytes());

        Ok(buf)
    }


    pub fn decrypt_private(
        data: Vec<u8>,
        rsa_pub: Rsa<Public>,
    ) -> Result<Vec<u8>, String>
    {
        let mut buf: Vec<u8> = vec![0; rsa_pub.size() as usize];
        let _ = rsa_pub.public_decrypt(&data, &mut buf, Padding::PKCS1).unwrap();

        Ok(buf)
    }
}