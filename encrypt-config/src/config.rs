//! # Config
//! This module provides a [`Config`] struct that can be used to store configuration values.

#[cfg(feature = "secret")]
use crate::encrypt_utils::Encrypter;
#[cfg(feature = "persist")]
use crate::PersistSource;
#[cfg(feature = "secret")]
use crate::SecretSource;
use crate::{CollectFailed, ConfigNotFound, ConfigResult, Source};
use snafu::OptionExt;
use std::collections::HashMap;
#[cfg(feature = "persist")]
use std::{path::PathBuf, sync::RwLock};

type CacheKey = String;
type CacheValue = Vec<u8>;
// type ConfigKV = (CacheKey, CacheValue);

#[cfg(feature = "persist")]
type PersistPath = RwLock<PathBuf>;

#[cfg(feature = "persist")]
enum Kind {
    Normal,
    Persist(PersistPath),
    #[cfg(feature = "secret")]
    Secret(PersistPath),
}

#[cfg(feature = "persist")]
type Cache = Vec<(Kind, HashMap<CacheKey, CacheValue>)>;
#[cfg(not(feature = "persist"))]
type Cache = HashMap<CacheKey, CacheValue>;

/// A struct that can be used to store configuration values.
/// # Example
/// See [`Source`], [`PersistSource`], [`SecretSource`]
#[derive(Default)]
pub struct Config {
    cache: Cache,
    #[cfg(feature = "secret")]
    encrypter: Encrypter,
}

impl Config {
    /// Create a new [`Config`] struct.
    /// # Arguments
    /// * `config_name` - The name of the rsa private key stored by `keyring`. Only needed when feature `secret` is on.
    ///
    #[cfg_attr(
        feature = "secret",
        doc = "To avoid entering the password during testing, you can enable `mock` feature. This can always return `Config`s with the **same** Encrypter during **each** test."
    )]
    pub fn new(#[cfg(feature = "secret")] secret_name: impl AsRef<str>) -> Self {
        Self {
            cache: Cache::new(),
            #[cfg(feature = "secret")]
            encrypter: Encrypter::new(secret_name).unwrap(),
        }
    }

    /// Get a value from the config.
    /// # Arguments
    /// * `key` - The key of the value to get.
    ///
    /// `R` must implement `serde::de::DeserializeOwned`, because this crate stores seriliazed data.
    pub fn get<K, R>(&self, key: K) -> ConfigResult<R>
    where
        K: AsRef<str>,
        R: serde::de::DeserializeOwned,
    {
        #[cfg(not(feature = "persist"))]
        let found = self.cache.get(key.as_ref());
        #[cfg(feature = "persist")]
        let found = self.cache.iter().find_map(|(_, map)| map.get(key.as_ref()));
        let serded = found.context(ConfigNotFound {
            key: key.as_ref().to_owned(),
        })?;
        Ok(serde_json::from_slice(serded)?)
    }

    /// Add a source to the config.
    /// The source must implement [`Source`] trait, which is for normal config that does not need to be encrypted or persisted.
    pub fn add_source(&mut self, source: impl Source) -> ConfigResult<()> {
        let map = source
            .default()
            .map_err(|_| CollectFailed.build())?
            .into_iter()
            .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()));
        #[cfg(feature = "persist")]
        self.cache.push((Kind::Normal, map.collect()));
        #[cfg(not(feature = "persist"))]
        self.cache.extend(map);
        Ok(())
    }

    /// Add a persist source to the config.
    /// The source must implement [`PersistSource`] trait, which is for config that needs to be persisted.
    #[cfg(feature = "persist")]
    pub fn add_persist_source(&mut self, source: impl PersistSource) -> ConfigResult<()> {
        let map = match std::fs::read(source.path()) {
            Ok(serded) => serde_json::from_slice(&serded)?,
            Err(_) => source
                .default()
                .map_err(|_| CollectFailed.build())?
                .into_iter()
                .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()))
                .collect(),
        };
        #[cfg(feature = "save_on_change")]
        std::fs::write(source.path(), serde_json::to_vec(&map)?)?;
        self.cache
            .push((Kind::Persist(RwLock::new(source.path())), map));
        Ok(())
    }

    /// Add a secret source to the config.
    /// The source must implement [`SecretSource`] trait, which is for config that needs to be encrypted and persisted.
    #[cfg(feature = "secret")]
    pub fn add_secret_source(&mut self, source: impl SecretSource) -> ConfigResult<()> {
        let map = match std::fs::read(source.path()) {
            Ok(encrypted) => serde_json::from_slice(&self.encrypter.decrypt(&encrypted)?)?,
            Err(_) => source
                .default()
                .map_err(|_| CollectFailed.build())?
                .into_iter()
                .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()))
                .collect(),
        };
        #[cfg(feature = "save_on_change")]
        std::fs::write(source.path(), self.encrypter.encrypt(&map)?)?;

        self.cache
            .push((Kind::Secret(RwLock::new(source.path())), map));
        Ok(())
    }

    /// Upgrade all values in the config. The keys must already exist.
    /// If not, those key-values before the first not existing one will be upgraded, and those after will be omitted.
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

    /// Upgrade a value in the config. The key must already exist.
    pub fn upgrade<K, V>(&mut self, key: K, value: &V) -> ConfigResult<()>
    where
        K: AsRef<str>,
        V: serde::Serialize,
    {
        #[cfg(feature = "persist")]
        {
            let (_kind, map) = self
                .cache
                .iter_mut()
                .find(|(_, map)| map.contains_key(key.as_ref()))
                .context(ConfigNotFound {
                    key: key.as_ref().to_owned(),
                })?;

            let v = map.get_mut(key.as_ref()).unwrap();
            *v = serde_json::to_vec(value)?;
            #[cfg(feature = "save_on_change")]
            match _kind {
                Kind::Normal => {}
                Kind::Persist(path) => {
                    let path = path.write().unwrap();
                    std::fs::write(&*path, serde_json::to_vec(map)?)?;
                }
                #[cfg(feature = "secret")]
                Kind::Secret(path) => {
                    let path = path.write().unwrap();
                    std::fs::write(&*path, self.encrypter.encrypt(map)?)?;
                }
            }
        }
        #[cfg(not(feature = "persist"))]
        {
            let v = self.cache.get_mut(key.as_ref()).context(ConfigNotFound {
                key: key.as_ref().to_owned(),
            })?;
            *v = serde_json::to_vec(value)?;
        }
        Ok(())
    }

    #[allow(unused)]
    #[cfg(not(feature = "save_on_change"))]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "save_on_change")))]
    fn save(&self) -> ConfigResult<()> {
        unimplemented!();
    }
}
