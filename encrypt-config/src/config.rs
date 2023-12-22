//! # Config
//! This module provides a `Config` struct that can be used to store configuration values.

use crate::{
    encrypt_utils::Encrypter, ConfigNotFound, ConfigResult, PersistSource, SecretSource, Source,
};
use std::collections::HashMap;
#[cfg(feature = "persist")]
use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::RwLock,
};

type CacheKey = String;
type CacheValue = Vec<u8>;
// type ConfigKV = (CacheKey, CacheValue);

#[cfg(feature = "persist")]
type PersistFile = RwLock<File>;

#[cfg(feature = "persist")]
enum Kind {
    Normal,
    Persist(PersistFile),
    #[cfg(feature = "encrypt")]
    Secret(PersistFile),
}

#[cfg(feature = "persist")]
type Cache = Vec<(Kind, HashMap<CacheKey, CacheValue>)>;
#[cfg(not(feature = "persist"))]
type Cache = HashMap<CacheKey, CacheValue>;

/// A struct that can be used to store configuration values.
/// # Example
/// See [`Source`], [`PersistSource`], [`SecretSource`]
pub struct Config {
    cache: Cache,
    #[cfg(feature = "encrypt")]
    encrypter: Encrypter,
}

impl Config {
    /// Create a new `Config` struct.
    /// # Arguments
    /// * `config_name` - The name of the rsa private key stored by `keyring`.
    #[cfg(feature = "encrypt")]
    pub fn new(secret_name: impl AsRef<str>) -> Self {
        Self {
            cache: Cache::new(),
            encrypter: Encrypter::new(secret_name).unwrap(),
        }
    }

    /// Create a new `Config` struct.
    #[cfg(not(feature = "encrypt"))]
    pub fn new() -> Self {
        Self {
            cache: Cache::new(),
        }
    }

    /// Get a value from the config.
    /// # Arguments
    /// * `key` - The key of the value to get.
    ///
    /// `R` must implement `serde::de::DeserializeOwned`, because this crate stores seriliazed data.
    #[cfg(feature = "persist")]
    pub fn get<K, R>(&self, key: K) -> ConfigResult<R>
    where
        K: AsRef<str>,
        R: serde::de::DeserializeOwned,
    {
        match self.cache.iter().find_map(|(_, map)| map.get(key.as_ref())) {
            Some(serded) => Ok(serde_json::from_slice(serded).unwrap()),
            None => Err(ConfigNotFound {
                key: key.as_ref().to_owned(),
            }
            .build()),
        }
    }

    /// Get a value from the config.
    /// # Arguments
    /// * `key` - The key of the value to get.
    ///
    /// `R` must implement `serde::de::DeserializeOwned`, because this crate stores seriliazed data.
    #[cfg(not(feature = "persist"))]
    pub fn get<K, R>(&self, key: K) -> ConfigResult<R>
    where
        K: AsRef<str>,
        R: serde::de::DeserializeOwned,
    {
        let serded = self.cache.get(key.as_ref()).context(ConfigNotFound {
            key: key.as_ref().to_owned(),
        })?;
        Ok(serde_json::from_slice(serded).unwrap())
    }

    /// Add a source to the config.
    /// The source must implement [`Source`] trait, which is for normal config that does not need to be encrypted or persisted.
    pub fn add_source(&mut self, source: impl Source) -> ConfigResult<()> {
        let patch = source.collect();
        #[cfg(feature = "persist")]
        self.cache.push((Kind::Normal, patch));
        #[cfg(not(feature = "persist"))]
        self.cache.extend(patch);
        Ok(())
    }

    /// Add a persist source to the config.
    /// The source must implement [`PersistSource`] trait, which is for config that needs to be persisted.
    #[cfg(feature = "persist")]
    pub fn add_persist_source(&mut self, source: impl PersistSource) -> ConfigResult<()> {
        let patch = source.collect();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(source.path())?;
        #[cfg(feature = "save_on_change")]
        file.write(&serde_json::to_vec(&patch)?)?;
        self.cache.push((Kind::Persist(RwLock::new(file)), patch));
        Ok(())
    }

    /// Add a secret source to the config.
    /// The source must implement [`SecretSource`] trait, which is for config that needs to be encrypted and persisted.
    #[cfg(feature = "encrypt")]
    pub fn add_secret_source(&mut self, source: impl SecretSource) -> ConfigResult<()> {
        let patch = source.collect(&self.encrypter);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(source.path())?;
        #[cfg(feature = "save_on_change")]
        file.write(&self.encrypter.encrypt(&patch)?)?;
        self.cache.push((Kind::Secret(RwLock::new(file)), patch));
        Ok(())
    }

    pub fn upgrade_all<'v, K, V, KV>(&mut self, kv: KV) -> ConfigResult<()>
    where
        K: AsRef<str>,
        V: serde::Serialize + 'v,
        KV: IntoIterator<Item = (K, &'v V)>,
    {
        for (key, value) in kv {
            self.upgrade(key, value)?;
        }
        Ok(())
    }

    #[cfg(feature = "persist")]
    pub fn upgrade<K, V>(&mut self, key: K, value: &V) -> ConfigResult<()>
    where
        K: AsRef<str>,
        V: serde::Serialize,
    {
        match self
            .cache
            .iter_mut()
            .find(|(_, map)| map.contains_key(key.as_ref()))
        {
            Some((kind, map)) => {
                let v = map.get_mut(key.as_ref()).unwrap();
                match kind {
                    Kind::Normal => {
                        *v = serde_json::to_vec(value)?;
                    }
                    Kind::Persist(file) => {
                        let mut file = file.write().unwrap();
                        *v = serde_json::to_vec(value)?;
                        #[cfg(feature = "save_on_change")]
                        file.write(&serde_json::to_vec(&map)?)?;
                    }
                    #[cfg(feature = "encrypt")]
                    Kind::Secret(file) => {
                        let mut file = file.write().unwrap();
                        *v = serde_json::to_vec(value)?;
                        #[cfg(feature = "save_on_change")]
                        file.write(&self.encrypter.encrypt(&map)?)?;
                    }
                }
                Ok(())
            }
            None => Err(ConfigNotFound {
                key: key.as_ref().to_owned(),
            }
            .build()),
        }
    }

    #[cfg(not(feature = "persist"))]
    pub fn upgrade<K, V>(&mut self, k: K, v: V) -> ConfigResult<()>
    where
        K: AsRef<str>,
        V: serde::Serialize,
    {
        let v = self.cache.get_mut(k.as_ref())?;
        *v = serde_json::to_vec(&v)?;
        Ok(())
    }

    #[cfg(not(feature = "save_on_change"))]
    fn save(&self) -> ConfigResult<()> {
        unimplemented!();
    }
}
