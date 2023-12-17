use std::collections::HashMap;

use crate::{encrypt_utils::Encrypter, ConfigResult, PersistSource, SecretSource, Source};

pub type ConfigKey = String;
pub type ConfigValue = Vec<u8>;

#[derive(Debug)]
pub struct Config {
    inner: HashMap<ConfigKey, ConfigValue>,
    encrypter: Encrypter,
}

impl Config {
    pub fn new(config_name: impl AsRef<str>) -> Self {
        Self {
            inner: HashMap::new(),
            encrypter: Encrypter::new(config_name).unwrap(),
        }
    }

    pub fn get<K, R>(&self, key: K) -> ConfigResult<R>
    where
        K: AsRef<str>,
        R: serde::de::DeserializeOwned,
    {
        let serded = self.inner.get(key.as_ref()).unwrap();
        Ok(serde_json::from_slice(serded).unwrap())
    }

    pub fn add_source(&mut self, source: impl Source) -> ConfigResult<()> {
        let map = source
            .collect()?
            .into_iter()
            .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()));
        self.inner.extend(map);
        Ok(())
    }

    pub fn add_persist_source(&mut self, source: impl PersistSource) -> ConfigResult<()> {
        let patch = source.collect()?;
        patch.apply(self)?;
        Ok(())
    }

    pub fn add_secret_source(&mut self, source: impl SecretSource) -> ConfigResult<()> {
        let patch = source.collect()?;
        patch.apply(self)?;
        Ok(())
    }
}

pub struct ConfigPatch {
    key: ConfigKey,
    func: Box<dyn FnOnce() -> ConfigResult<ConfigValue>>,
}

impl ConfigPatch {
    pub(crate) fn new(key: ConfigKey, func: Box<dyn FnOnce() -> ConfigResult<Vec<u8>>>) -> Self {
        Self { key, func }
    }

    pub fn apply(self, config: &mut Config) -> ConfigResult<()> {
        let func = self.func;
        let new_value = func()?;
        config.inner.insert(self.key, new_value);
        Ok(())
    }
}

pub struct SecretConfigPatch {
    key: ConfigKey,
    func: Box<dyn FnOnce(&Encrypter) -> ConfigResult<ConfigValue>>,
}

impl SecretConfigPatch {
    pub(crate) fn new(
        key: ConfigKey,
        func: Box<dyn FnOnce(&Encrypter) -> ConfigResult<Vec<u8>>>,
    ) -> Self {
        Self { key, func }
    }

    pub fn apply(self, config: &mut Config) -> ConfigResult<()> {
        let func = self.func;
        let new_value = func(&config.encrypter)?;
        config.inner.insert(self.key, new_value);
        Ok(())
    }
}
