//! # Encrypt-utils
//! Encryption helper.

use crate::ConfigResult;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Encrypter {
    priv_key: RsaPrivateKey,
}

pub(crate) type Encrypted = Vec<u8>;
pub(crate) type Decrypted = Vec<u8>;

impl Encrypter {
    pub(crate) fn new(secret_name: impl AsRef<str>) -> ConfigResult<Self> {
        let entry = keyring_entry(secret_name);
        match entry.get_password() {
            Ok(serded_enc) => Ok(serde_json::from_str(&serded_enc)?),
            Err(keyring::Error::NoEntry) => {
                let new_enc = Encrypter::build();
                entry.set_password(&serde_json::to_string(&new_enc).unwrap())?;
                Ok(new_enc)
            }
            Err(e) => Err(e)?,
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

    pub(crate) fn encrypt<S: serde::Serialize>(&self, to_encrypt: &S) -> ConfigResult<Encrypted> {
        let origin = serde_json::to_vec(to_encrypt).unwrap();
        self.encrypt_serded(&origin)
    }

    /// This is used to encrypt seriliazed Value.
    /// # Arguments
    /// * origin - The returning of `serde_json::to_vec`
    /// # Example
    /// ```ignore
    /// let foo = Foo::new();
    /// let serded = serde_json::to_vec(&foo)?;
    /// let encrypter = Encrypter::new("test")?;
    /// let encrypted = encrypter.encrypt_serded(serded)?;
    /// ```
    /// # Question
    /// Q: Why not use `Foo` as origin more conviniently?
    ///
    /// A: The user passes `&Foo` to [`SecretSource::upgrade`] to upgrade the config, which returns a [`SecretConfigPatch`],
    /// containing a [`Func`] as its field. `Func`, which is a boxed closure, should take the ownership of `Foo` if directly use
    /// it. To avoid this, and due to we need seriliaze it anyway, we just move its serded `Vec<u8>` into the closure.
    pub(crate) fn encrypt_serded(&self, origin: &[u8]) -> ConfigResult<Encrypted> {
        let mut rng = rand::thread_rng();
        let chunk_size = if cfg!(not(target_os = "windows")) {
            245 // (2048 >> 3) - 11
        } else {
            117 // (1024 >> 3) - 11
        };
        let pub_key = RsaPublicKey::from(&self.priv_key);
        let mut encrypted = vec![];
        for c in origin.chunks(chunk_size) {
            encrypted.extend(pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, c)?);
        }
        Ok(encrypted)
    }

    pub(crate) fn decrypt(&self, encrypted: &[u8]) -> ConfigResult<Decrypted> {
        let mut decrypted = vec![];
        let chunk_size = if cfg!(not(target_os = "windows")) {
            256
        } else {
            128
        };
        for c in encrypted.chunks(chunk_size) {
            decrypted.extend(self.priv_key.decrypt(Pkcs1v15Encrypt, c)?);
        }
        Ok(decrypted)
    }
}

fn keyring_entry(secret_name: impl AsRef<str>) -> keyring::Entry {
    let user = std::env::var("USER").unwrap_or("unknown".to_string());
    #[cfg(test)]
    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    keyring::Entry::new_with_target("user", secret_name.as_ref(), &user).unwrap()
}
