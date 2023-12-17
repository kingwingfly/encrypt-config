use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

use crate::ConfigResult;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Encrypter {
    priv_key: RsaPrivateKey,
}

pub type Encrypted = Vec<u8>;
pub type Decrypted = Vec<u8>;

impl Encrypter {
    pub fn new(config_name: impl AsRef<str>) -> ConfigResult<Self> {
        let entry = keyring_entry(config_name);
        match entry.get_password() {
            Ok(serded_enc) => Ok(serde_json::from_str(&serded_enc).unwrap()),
            Err(_) => {
                let new_enc = Encrypter::build();
                entry
                    .set_password(&serde_json::to_string(&new_enc).unwrap())
                    .unwrap();
                Ok(new_enc)
            }
        }
    }

    fn build() -> Self {
        let mut rng = rand::thread_rng();
        let bits = if cfg!(not(target_os = "windows")) {
            2048
        } else {
            1024 // too long isn't accepted by Win
        };
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        Self { priv_key }
    }

    // pub fn encrypt<S: serde::Serialize>(&self, to_encrypt: &S) -> ConfigResult<Encrypted> {
    //     let origin = serde_json::to_vec(to_encrypt).unwrap();
    //     self.encrypt_serded(&origin)
    // }

    pub fn encrypt_serded(&self, origin: &[u8]) -> ConfigResult<Encrypted> {
        let mut rng = rand::thread_rng();
        let chunk_size = if cfg!(not(target_os = "windows")) {
            245 // (2048 >> 3) - 11
        } else {
            117 // (1024 >> 3) - 11
        };
        let pub_key = RsaPublicKey::from(&self.priv_key);
        let mut encrypted = vec![];
        for c in origin.chunks(chunk_size) {
            encrypted.extend(pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, c).unwrap());
        }
        Ok(encrypted)
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> ConfigResult<Decrypted> {
        let mut decrypted = vec![];
        let chunk_size = if cfg!(not(target_os = "windows")) {
            256
        } else {
            128
        };
        for c in encrypted.chunks(chunk_size) {
            decrypted.extend(self.priv_key.decrypt(Pkcs1v15Encrypt, c).unwrap());
        }
        Ok(decrypted)
    }
}

fn keyring_entry(config_name: impl AsRef<str>) -> keyring::Entry {
    let user = std::env::var("USER").unwrap_or("unknown".to_string());
    keyring::Entry::new_with_target("user", config_name.as_ref(), &user).unwrap()
}
