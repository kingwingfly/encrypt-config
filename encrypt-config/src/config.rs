//! # Config
//! This module provides a `Config` struct that can be used to store configuration values.

use snafu::OptionExt;
use std::collections::HashMap;

use crate::{
    encrypt_utils::Encrypter, CollectFailed, ConfigNotFound, ConfigResult, PersistSource,
    SecretSource, Source,
};

type ConfigKey = String;
type ConfigValue = Vec<u8>;
type ConfigKV = (ConfigKey, ConfigValue);

/// A struct that can be used to store configuration values.
/// # Example
/// See [`Source`], [`PersistSource`], [`SecretSource`]
#[derive(Debug)]
pub struct Config {
    inner: HashMap<ConfigKey, ConfigValue>,
    encrypter: Encrypter,
}

impl Config {
    /// Create a new `Config` struct.
    /// # Arguments
    /// * `config_name` - The name of the rsa private key stored by `keyring`.
    pub fn new(secret_name: impl AsRef<str>) -> Self {
        Self {
            inner: HashMap::new(),
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
        let serded = self.inner.get(key.as_ref()).context(ConfigNotFound {
            key: key.as_ref().to_owned(),
        })?;
        Ok(serde_json::from_slice(serded).unwrap())
    }

    /// Add a source to the config.
    /// The source must implement [`Source`] trait, which is for normal config that does not need to be encrypted or persisted.
    pub fn add_source(&mut self, source: impl Source) -> ConfigResult<()> {
        let patch = source
            .collect()
            .map_err(|_| CollectFailed.build())?
            .into_iter()
            .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()));
        self.inner.extend(patch);
        Ok(())
    }

    /// Add a persist source to the config.
    /// The source must implement [`PersistSource`] trait, which is for config that needs to be persisted.
    pub fn add_persist_source(&mut self, source: impl PersistSource) -> ConfigResult<()> {
        let patch = source.collect();
        self.inner.extend(patch);
        Ok(())
    }

    /// Add a secret source to the config.
    /// The source must implement [`SecretSource`] trait, which is for config that needs to be encrypted and persisted.
    pub fn add_secret_source(&mut self, source: impl SecretSource) -> ConfigResult<()> {
        let patch = source.collect(&self.encrypter);
        self.inner.extend(patch);
        Ok(())
    }
}

type PatchFunc = Box<dyn FnOnce() -> ConfigResult<ConfigKV>>;

/// A patch that can be used to modify the config.
/// You can get a [`ConfigPatch`] by calling [`PersistSource::upgrade`], and apply it by calling [`ConfigPatch::apply`] to a config.
/// No change will happen until you call [`ConfigPatch::apply`].
/// # Example
/// See [`PersistSource`]
pub struct ConfigPatch {
    func: PatchFunc,
}

impl ConfigPatch {
    pub(crate) fn new(func: PatchFunc) -> Self {
        Self { func }
    }

    pub fn apply(self, config: &mut Config) -> ConfigResult<()> {
        let func = self.func;
        let (k, v) = func()?;
        config.inner.insert(k, v);
        Ok(())
    }
}

type Func = Box<dyn FnOnce(&Encrypter) -> ConfigResult<ConfigKV>>;

/// A patch that can be used to modify the config.
/// You can get a [`SecretConfigPatch`] by calling [`SecretSource::upgrade`], and apply it by calling [`SecretConfigPatch::apply`] to a config.
/// No change will happen until you call [`SecretConfigPatch::apply`].
/// # Example
/// See [`SecretSource`]
pub struct SecretConfigPatch {
    func: Func,
}

impl SecretConfigPatch {
    pub(crate) fn new(func: Func) -> Self {
        Self { func }
    }

    pub fn apply(self, config: &mut Config) -> ConfigResult<()> {
        let func = self.func;
        let (k, v) = func(&config.encrypter)?;
        config.inner.insert(k, v);
        Ok(())
    }
}
