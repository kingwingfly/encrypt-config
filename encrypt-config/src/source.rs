use crate::encrypt_utils::Encrypter;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// A trait for normal config source that is neither encrypted or persisted.
/// # Example
/// ```no_run
/// use encrypt_config::{Config, Source};
///
/// let mut config = Config::new("test");
///
/// struct NormalSource;
/// impl Source for NormalSource {
///     type Value = String;
///     type Map = Vec<(String, Self::Value)>;
///
///     fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
///         Ok(vec![("key".to_owned(), "value".to_owned())])
///     }
/// }
///
/// config.add_source(NormalSource).unwrap();
/// let v: String = config.get("key").unwrap();
/// assert_eq!(v, "value");
/// ```
pub trait Source {
    type Value: Serialize + DeserializeOwned;
    type Map: IntoIterator<Item = (String, Self::Value)>;

    fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>>;
}

/// A trait for persisted but not encrypted config source.
/// # Example
/// ```no_run
/// use encrypt_config::{Config, PersistSource};
/// use serde::{Deserialize, Serialize};
///
/// let mut config = Config::new("test");
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Foo(String);
///
/// struct PersistSourceImpl;
/// impl PersistSource for PersistSourceImpl {
///     type Value = Foo;
///
///     #[cfg(not(feature = "default_config_dir"))]
///     fn path(&self) -> std::path::PathBuf {
///         std::path::PathBuf::from("tests").join("persist.conf")
///     }
///
///     #[cfg(feature = "default_config_dir")]
///     fn source_name(&self) -> String {
///         "persist.conf".to_owned()
///     }
/// }
///
/// config.add_persist_source(PersistSourceImpl).unwrap();
/// let new_value = Foo("hello".to_owned());
/// let patch = PersistSourceImpl.upgrade("persist", &new_value);
/// patch.apply(&mut config).unwrap();
/// assert_eq!(config.get::<_, Foo>("persist").unwrap(), new_value);
///
/// let mut config_new = Config::new("test");
/// config_new.add_persist_source(PersistSourceImpl).unwrap(); // Read config from disk
/// assert_eq!(config_new.get::<_, Foo>("persist").unwrap(), new_value);
/// ```
#[cfg(feature = "persist")]
pub trait PersistSource {
    type Value: Serialize + DeserializeOwned;

    #[cfg(feature = "default_config_dir")]
    fn source_name(&self) -> String;

    #[cfg(feature = "default_config_dir")]
    fn path(&self) -> std::path::PathBuf {
        dirs_next::config_dir()
            .expect("Default config dir unknown, please turn off feature `default_config_dir`")
            .join(self.source_name())
    }

    /// Take effect only when the persisted config doesn't exists
    fn default(&self) -> HashMap<String, Self::Value> {
        HashMap::new()
    }

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf;

    fn collect(&self) -> HashMap<String, Vec<u8>> {
        match std::fs::read(self.path()) {
            Ok(serded) => serde_json::from_slice(&serded).unwrap(),
            Err(_) => self
                .default()
                .into_iter()
                .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()))
                .collect(),
        }
    }
}

/// A trait for persisted and encrypted config source.
/// # Example
/// ```no_run
/// use encrypt_config::{Config, SecretSource};
/// use serde::{Deserialize, Serialize};
///
/// let mut config = Config::new("test");
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Foo(String);
///
/// struct SecretSourceImpl;
/// impl SecretSource for SecretSourceImpl {
///     type Value = Foo;
///
///     #[cfg(not(feature = "default_config_dir"))]
///     fn path(&self) -> std::path::PathBuf {
///         std::path::PathBuf::from("tests").join("secret.conf")
///     }
///
///     #[cfg(feature = "default_config_dir")]
///     fn source_name(&self) -> String {
///         "secret.conf".to_owned()
///     }
/// }
///
/// config.add_secret_source(SecretSourceImpl).unwrap();
/// let new_value = Foo("world".to_owned());
/// let patch = SecretSourceImpl.upgrade("secret", &new_value);
/// patch.apply(&mut config).unwrap();
/// assert_eq!(config.get::<_, Foo>("secret").unwrap(), new_value);
/// ```
#[cfg(feature = "encrypt")]
pub trait SecretSource {
    type Value: Serialize + DeserializeOwned;

    #[cfg(feature = "default_config_dir")]
    fn source_name(&self) -> String;

    #[cfg(feature = "default_config_dir")]
    fn path(&self) -> std::path::PathBuf {
        dirs_next::config_dir()
            .expect("Default config dir unknown, please turn off feature `default_config_dir`")
            .join(self.source_name())
    }

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf;

    /// Take effect only when the persisted config doesn't exists or cannnot be decrypted
    fn default(&self) -> HashMap<String, Self::Value> {
        HashMap::new()
    }

    fn collect(&self, encrypter: &Encrypter) -> HashMap<String, Vec<u8>> {
        match std::fs::read(self.path()) {
            Ok(encrypted) => {
                serde_json::from_slice(&encrypter.decrypt(&encrypted).unwrap()).unwrap()
            }
            Err(_) => self
                .default()
                .into_iter()
                .map(|(k, v)| (k, serde_json::to_vec(&v).unwrap()))
                .collect(),
        }
    }
}
