//! # Config
//! This module provides a `Config` struct that can be used to store configuration values.

use std::collections::HashMap;

use crate::{encrypt_utils::Encrypter, ConfigResult, PersistSource, SecretSource, Source};

pub type ConfigKey = String;
pub(crate) type ConfigValue = Vec<u8>;

/// A struct that can be used to store configuration values.
/// # Example
/// ```
/// use encrypt_config::{Config, Source, ConfigResult};
///
/// let mut config = Config::new("test");
///
/// struct NormalSource;
/// impl Source for NormalSource {
///     type Value = String;
///     type Map = Vec<(String, Self::Value)>;
///
///     fn collect(&self) -> ConfigResult<Self::Map> {
///         Ok(vec![("key".to_owned(), "value".to_owned())])
///     }
/// }
///
/// config.add_source(NormalSource).unwrap();
/// let v: String = config.get("key").unwrap();
/// assert_eq!(v, "value");
/// ```
#[derive(Debug)]
pub struct Config {
    inner: HashMap<ConfigKey, ConfigValue>,
    encrypter: Encrypter,
}

impl Config {
    /// Create a new `Config` struct.
    /// # Arguments
    /// * `config_name` - The name of the rsa private key stored by `keyring`.
    pub fn new(config_name: impl AsRef<str>) -> Self {
        Self {
            inner: HashMap::new(),
            encrypter: Encrypter::new(config_name).unwrap(),
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
        let serded = self.inner.get(key.as_ref()).unwrap();
        Ok(serde_json::from_slice(serded).unwrap())
    }

    /// Add a source to the config.
    /// The source must implement [`Source`] trait, which is for normal config that does not need to be encrypted or persisted.
    pub fn add_source(&mut self, source: impl Source) -> ConfigResult<()> {
        let map = source
            .collect()?
            .into_iter()
            .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()));
        self.inner.extend(map);
        Ok(())
    }

    /// Add a persist source to the config.
    /// The source must implement [`PersistSource`] trait, which is for config that needs to be persisted.
    pub fn add_persist_source(&mut self, source: impl PersistSource) -> ConfigResult<()> {
        let patch = source.collect()?;
        patch.apply(self)?;
        Ok(())
    }

    /// Add a secret source to the config.
    /// The source must implement [`SecretSource`] trait, which is for config that needs to be encrypted and persisted.
    pub fn add_secret_source(&mut self, source: impl SecretSource) -> ConfigResult<()> {
        let patch = source.collect()?;
        patch.apply(self)?;
        Ok(())
    }
}

/// A patch that can be used to modify the config.
/// You can get a `ConfigPatch` by calling [`PersistSource::upgrade`], and apply it by calling [`ConfigPatch::apply`] to a config.
/// No change will happen until you call [`ConfigPatch::apply`].
/// # Example
/// ```rust
/// use encrypt_config::{Config, ConfigKey, PersistSource, ConfigResult};
///
/// let mut config = Config::new("test");
///
/// #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
/// struct Foo(String);
///
/// struct PersistSourceImpl;
/// impl PersistSource for PersistSourceImpl {
///     type Value = Foo;
///
///     fn source_name(&self) -> ConfigKey {
///         "test".to_owned()
///     }
///
///     fn default(&self) -> Self::Value {
///         Foo("hello".to_owned())
///     }
///
///     /// You can omit this if you turn on the `default_config_dir` feature. The feature will use the default config dir of your OS.
///     fn path(&self) -> std::path::PathBuf {
///         std::path::PathBuf::from("tests").join(self.source_name())
///     }
/// }
/// config.add_persist_source(PersistSourceImpl).unwrap();
/// let v: Foo = config.get("test").unwrap();
/// assert_eq!(v, Foo("hello".to_owned()));
/// let patch = PersistSourceImpl.upgrade(&Foo("hi".to_owned())).unwrap();
/// patch.apply(&mut config).unwrap();
/// let v: Foo = config.get("test").unwrap();
/// assert_eq!(v, Foo("hi".to_owned()));
/// # std::fs::remove_file("tests/test").unwrap();
/// ```
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

/// A patch that can be used to modify the config.
/// You can get a `SecretConfigPatch` by calling [`SecretSource::upgrade`], and apply it by calling [`SecretConfigPatch::apply`] to a config.
/// No change will happen until you call [`SecretConfigPatch::apply`].
/// # Example
/// ```rust
/// use encrypt_config::{Config, ConfigKey, SecretSource, ConfigResult};
///
/// let mut config = Config::new("test");
///
/// #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
/// struct Foo(String);
///
/// struct SecretSourceImpl;
/// impl SecretSource for SecretSourceImpl {
///     type Value = Foo;
///
///     fn source_name(&self) -> ConfigKey {
///         "secret_test".to_owned()
///     }
///
///     fn default(&self) -> Self::Value {
///         Foo("hello".to_owned())
///     }
///
///     /// You can omit this if you turn on the `default_config_dir` feature. The feature will use the default config dir of your OS.
///     fn path(&self) -> std::path::PathBuf {
///         std::path::PathBuf::from("tests").join(self.source_name())
///     }
/// }
/// config.add_secret_source(SecretSourceImpl).unwrap();
/// let v: Foo = config.get("secret_test").unwrap();
/// assert_eq!(v, Foo("hello".to_owned()));
/// let patch = SecretSourceImpl.upgrade(&Foo("hi".to_owned())).unwrap();
/// patch.apply(&mut config).unwrap();
/// let v: Foo = config.get("secret_test").unwrap();
/// assert_eq!(v, Foo("hi".to_owned()));
/// # std::fs::remove_file("tests/secret_test").unwrap();
/// ```
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
