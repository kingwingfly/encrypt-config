//! # Encrypt-utils
//! Encryption helper.

use crate::error::{ConfigError, ConfigResult};
use keyring::Entry;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Encrypter {
    priv_key: RsaPrivateKey,
}

impl Default for Encrypter {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        #[cfg(not(target_os = "windows"))]
        let bits = 2048;
        #[cfg(target_os = "windows")]
        let bits = 1024; // too long isn't accepted by Win
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        Self { priv_key }
    }
}

impl Encrypter {
    pub(crate) fn new(secret_name: impl AsRef<str>) -> ConfigResult<&'static Self> {
        static ENCRYPTERS: OnceLock<RwLock<HashMap<String, &'static Encrypter>>> = OnceLock::new();
        let encrypters = ENCRYPTERS.get_or_init(|| RwLock::new(HashMap::new()));
        let mut encrypters = encrypters.write().unwrap();
        // Why not `read` to examine, then `write` to insert if not exists?
        // Because many threads may try examining at the same time, and all of them may find
        // the entry not exists. Then all of them will try to insert, which is not what we want.
        if let Some(encrypter) = encrypters.get(secret_name.as_ref()) {
            return Ok(encrypter);
        }
        let new = Box::leak(Box::new(Self::init(secret_name.as_ref())?));
        encrypters.insert(secret_name.as_ref().to_owned(), new);
        Ok(new)
    }

    /// Init a encrypter. Load if exists, otherwise create and save a new one.
    fn init(secret_name: &str) -> ConfigResult<Self> {
        let entry = keyring_entry(secret_name)?;
        match entry.get_password() {
            Ok(serded_enc) => {
                serde_json::from_str(&serded_enc).map_err(|_| ConfigError::LoadEncrypterFailed)
            }
            Err(keyring::Error::NoEntry) => {
                let new_enc = Encrypter::default();
                entry
                    .set_password(&serde_json::to_string(&new_enc).unwrap())
                    .map_err(|_| ConfigError::KeyringError)?;
                Ok(new_enc)
            }
            Err(_) => Err(ConfigError::KeyringError),
        }
    }

    pub(crate) fn encrypt<T: serde::Serialize>(&self, to_encrypt: &T) -> ConfigResult<Vec<u8>> {
        let origin = serde_json::to_vec(to_encrypt)?;
        self.encrypt_serded(&origin)
    }

    fn encrypt_serded(&self, origin: &[u8]) -> ConfigResult<Vec<u8>> {
        let mut rng = rand::thread_rng();
        #[cfg(not(target_os = "windows"))]
        const CHUNK_SIZE: usize = 245; // (2048 >> 3) - 11
        #[cfg(target_os = "windows")]
        const CHUNK_SIZE: usize = 117; // (1024 >> 3) - 11
        let pub_key = RsaPublicKey::from(&self.priv_key);
        let mut encrypted = vec![];
        for c in origin.chunks(CHUNK_SIZE) {
            encrypted.extend(pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, c)?);
        }
        Ok(encrypted)
    }

    pub(crate) fn decrypt<T>(&self, encrypted: &[u8]) -> ConfigResult<T>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        #[cfg(not(target_os = "windows"))]
        const CHUNK_SIZE: usize = 256;
        #[cfg(target_os = "windows")]
        const CHUNK_SIZE: usize = 128;
        let mut decrypted = vec![];
        for c in encrypted.chunks(CHUNK_SIZE) {
            decrypted.extend(self.priv_key.decrypt(Pkcs1v15Encrypt, c)?);
        }
        Ok(serde_json::from_slice(&decrypted)?)
    }
}

fn keyring_entry(secret_name: impl AsRef<str>) -> ConfigResult<Entry> {
    #[cfg(feature = "mock")]
    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    let user = std::env::var("USER").unwrap_or("unknown".to_string());
    Entry::new(secret_name.as_ref(), &user).map_err(|_| ConfigError::KeyringError)
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use super::*;

    #[test]
    fn encrypter_test() {
        let encrypter1 = Encrypter::new("test").unwrap();
        let encrypter2 = Encrypter::new("test").unwrap();
        assert_eq!(encrypter1, encrypter2);
        let encrypter3 = Encrypter::new("another").unwrap();
        assert_ne!(encrypter1, encrypter3);
    }
}
