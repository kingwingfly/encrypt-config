#![doc = include_str!("../README.md")]

mod config;
mod encrypt_utils;
mod error;

pub use config::{Config, ConfigPatch, SecretConfigPatch};
pub use error::*;

use encrypt_utils::Encrypter;

/// A trait for normal config source that is neither encrypted or persisted.
/// # Example
/// See [`config::Config`]
pub trait Source {
    type Value: serde::Serialize;
    type Map: IntoIterator<Item = (String, Self::Value)>;

    fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>>;

    fn upgrade(&self, key: impl AsRef<str>, new_value: &Self::Value) -> ConfigPatch {
        let serded = serde_json::to_vec(&new_value).unwrap();
        let func = Box::new(move || Ok(serded));
        ConfigPatch::new(key.as_ref().to_owned(), func)
    }
}

/// A trait for persisted but not encrypted config source.
/// # Example
/// See [`config::ConfigPatch`]
pub trait PersistSource {
    type Value: serde::Serialize + serde::de::DeserializeOwned;

    fn source_name(&self) -> String;

    /// This will be used to initialize the source if not existing.
    fn default(&self) -> Self::Value;

    #[cfg(feature = "default_config_dir")]
    fn path(&self) -> std::path::PathBuf {
        dirs_next::config_dir()
            .expect("Default config dir unknown, turn off feature `default_config_dir`")
            .join(self.source_name())
    }

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf;

    fn collect(&self) -> ConfigPatch {
        match std::fs::read(self.path()) {
            Ok(serded) => {
                let func = Box::new(move || Ok(serded));
                ConfigPatch::new(self.source_name(), func)
            }
            Err(_) => self.upgrade(&self.default()),
        }
    }

    fn upgrade(&self, new_value: &Self::Value) -> ConfigPatch {
        let path = self.path();
        let serded = serde_json::to_vec(new_value).unwrap();
        let func = Box::new(move || {
            std::fs::write(path, &serded)?;
            Ok(serded)
        });
        ConfigPatch::new(self.source_name(), func)
    }
}

/// A trait for persisted and encrypted config source.
/// # Example
/// See [`config::SecretConfigPatch`]
pub trait SecretSource {
    type Value: serde::Serialize + serde::de::DeserializeOwned;

    fn source_name(&self) -> String;

    /// This will be used to initialize the source if not existing.
    fn default(&self) -> Self::Value;

    #[cfg(feature = "default_config_dir")]
    fn path(&self) -> std::path::PathBuf {
        dirs_next::config_dir()
            .expect("Default config dir unknown, turn off feature `default_config_dir`")
            .join(self.source_name())
    }

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf;

    fn collect(&self) -> SecretConfigPatch {
        match std::fs::read(self.path()) {
            Ok(encrypted) => {
                let func = Box::new(move |encrypter: &Encrypter| {
                    let serded = encrypter.decrypt(&encrypted).unwrap();
                    Ok(serded)
                });
                SecretConfigPatch::new(self.source_name(), func)
            }
            Err(_) => self.upgrade(&self.default()),
        }
    }

    fn upgrade(&self, new_value: &Self::Value) -> SecretConfigPatch {
        let path = self.path();
        let new_value = serde_json::to_vec(new_value).unwrap();
        let func = Box::new(move |encrypter: &Encrypter| {
            let encrypted = encrypter.encrypt_serded(&new_value)?;
            std::fs::write(path, encrypted)?;
            Ok(new_value)
        });
        SecretConfigPatch::new(self.source_name(), func)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NormalSource;
    impl Source for NormalSource {
        type Value = String;
        type Map = Vec<(String, Self::Value)>;

        fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
            Ok(vec![("key".to_owned(), "value".to_owned())])
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Foo(String);

    struct PersistSourceImpl;
    impl PersistSource for PersistSourceImpl {
        type Value = Foo;

        fn source_name(&self) -> String {
            "test".to_owned()
        }

        fn default(&self) -> Self::Value {
            Foo("hello".to_owned())
        }

        fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::from("tests").join(self.source_name())
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Bar(String);

    struct SecretSourceImpl;
    impl SecretSource for SecretSourceImpl {
        type Value = Bar;

        fn source_name(&self) -> String {
            "secret_test".to_owned()
        }

        fn default(&self) -> Self::Value {
            Bar("world".to_owned())
        }

        fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::from("tests").join(self.source_name())
        }
    }

    #[test]
    fn config_tests() {
        let mut config = Config::new("test");
        config.add_source(NormalSource).unwrap();
        config.add_persist_source(PersistSourceImpl).unwrap();
        config.add_secret_source(SecretSourceImpl).unwrap();
        let v: String = config.get("key").unwrap();
        assert_eq!(v, "value");
        let v: Foo = config.get("test").unwrap();
        assert_eq!(v, Foo("hello".to_owned()));
        let v: Bar = config.get("secret_test").unwrap();
        assert_eq!(v, Bar("world".to_owned()));
        let patch = NormalSource.upgrade("key", &"new_value".to_owned());
        patch.apply(&mut config).unwrap();
        let v: String = config.get("key").unwrap();
        assert_eq!(v, "new_value");
        let patch = PersistSourceImpl.upgrade(&Foo("hi".to_owned()));
        patch.apply(&mut config).unwrap();
        let v: Foo = config.get("test").unwrap();
        assert_eq!(v, Foo("hi".to_owned()));
        let patch = SecretSourceImpl.upgrade(&Bar("Louis".to_owned()));
        patch.apply(&mut config).unwrap();
        let v: Bar = config.get("secret_test").unwrap();
        assert_eq!(v, Bar("Louis".to_owned()));
        std::fs::remove_file("tests/secret_test").unwrap();
        std::fs::remove_file("tests/test").unwrap();
    }
}
