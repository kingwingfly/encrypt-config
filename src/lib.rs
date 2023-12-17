//! # encrypt-config
//! A crate helping managing, persisting, encrypting configs.

mod config;
mod encrypt_utils;
mod error;

pub use config::{Config, ConfigKey, ConfigPatch, SecretConfigPatch};
pub use error::*;

use encrypt_utils::Encrypter;

/// A trait for normal config source that is neither encrypted or persisted.
/// # Example
/// See [`config::Config`]
pub trait Source {
    type Value: serde::Serialize;
    type Map: IntoIterator<Item = (String, Self::Value)>;

    fn collect(&self) -> ConfigResult<Self::Map>;

    fn upgrade(&self, key: impl AsRef<str>, new_value: &Self::Value) -> ConfigResult<ConfigPatch> {
        let serded = serde_json::to_vec(&new_value).unwrap();
        let func = Box::new(move || Ok(serded));
        let patch = ConfigPatch::new(key.as_ref().to_owned(), func);
        Ok(patch)
    }
}

/// A trait for persisted but not encrypted config source.
/// # Example
/// See [`config::ConfigPatch`]
pub trait PersistSource {
    type Value: serde::Serialize + serde::de::DeserializeOwned;

    fn source_name(&self) -> ConfigKey;

    /// This will be used to initialize the source if not existing.
    fn default(&self) -> Self::Value;

    #[cfg(feature = "default_config_dir")]
    fn path(&self) -> std::path::PathBuf {
        dirs_next::config_dir().unwrap().join(self.source_name())
    }

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf;

    fn collect(&self) -> ConfigResult<ConfigPatch> {
        match std::fs::read(self.path()) {
            Ok(serded) => {
                let func = Box::new(move || Ok(serded));
                let patch = ConfigPatch::new(self.source_name(), func);
                Ok(patch)
            }
            Err(_) => Ok(self.upgrade(&self.default()).unwrap()),
        }
    }

    fn upgrade(&self, new_value: &Self::Value) -> ConfigResult<ConfigPatch> {
        let path = self.path();
        let serded = serde_json::to_vec(new_value).unwrap();
        let func = Box::new(move || {
            std::fs::write(path, &serded).unwrap();
            Ok(serded)
        });
        let patch = ConfigPatch::new(self.source_name(), func);
        Ok(patch)
    }
}

/// A trait for persisted and encrypted config source.
/// # Example
/// See [`config::SecretConfigPatch`]
pub trait SecretSource {
    type Value: serde::Serialize + serde::de::DeserializeOwned;

    fn source_name(&self) -> ConfigKey;

    /// This will be used to initialize the source if not existing.
    fn default(&self) -> Self::Value;

    #[cfg(feature = "default_config_dir")]
    fn path(&self) -> std::path::PathBuf {
        dirs_next::config_dir().unwrap().join(self.source_name())
    }

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf;

    fn collect(&self) -> ConfigResult<SecretConfigPatch> {
        match std::fs::read(self.path()) {
            Ok(encrypted) => {
                let func = Box::new(move |encrypter: &Encrypter| {
                    let serded = encrypter.decrypt(&encrypted).unwrap();
                    Ok(serded)
                });
                let patch = SecretConfigPatch::new(self.source_name(), func);
                Ok(patch)
            }
            Err(_) => Ok(self.upgrade(&self.default()).unwrap()),
        }
    }

    fn upgrade(&self, new_value: &Self::Value) -> ConfigResult<SecretConfigPatch> {
        let path = self.path();
        let new_value = serde_json::to_vec(new_value).unwrap();
        let func = Box::new(move |encrypter: &Encrypter| {
            let encrypted = encrypter.encrypt_serded(&new_value).unwrap();
            std::fs::write(path, &encrypted).unwrap();
            Ok(new_value)
        });
        let patch = SecretConfigPatch::new(self.source_name(), func);
        Ok(patch)
    }
}
