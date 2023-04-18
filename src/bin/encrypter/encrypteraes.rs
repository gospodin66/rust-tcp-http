pub mod encrypteraes {
    use aes_gcm::{
        aead::{
            Aead,
            KeyInit,
            OsRng,
            consts::{
                B1,
                B0
            },
            generic_array::GenericArray
        },
        Aes256Gcm,
        AesGcm,
        aes::{
            cipher::typenum::{
                UTerm,
                UInt
            },
            Aes256
        }
    };

    #[derive(Clone)]
    pub struct AesEncrypter {
        _key: GenericArray<u8, UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>>,
        cipher: AesGcm<Aes256, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>
    }
    
    impl AesEncrypter {
        pub fn construct() -> AesEncrypter {
            let key = Aes256Gcm::generate_key(&mut OsRng);
            AesEncrypter {
                _key: key,
                cipher: Aes256Gcm::new(&key),
            }
        }
    
        pub fn encrypt(self, nonce: &GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>, message: &[u8]) -> Vec<u8> {
            return self.cipher.encrypt(nonce, message).unwrap()
        }
        
        pub fn decrypt(self, nonce: &GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>, ciphertext: &Vec<u8>) -> Vec<u8> {
            return self.cipher.decrypt(nonce, ciphertext.as_ref()).unwrap();
        }
    }
}
