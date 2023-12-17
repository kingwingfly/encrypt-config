mod config;
mod encrypt_utils;
mod error;

pub use config::Config;
pub use error::*;

use config::{ConfigKey, ConfigPatch, SecretConfigPatch};
use encrypt_utils::Encrypter;

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

#[cfg(test)]
mod tests {
    use super::*;

    struct NormalSource;
    impl Source for NormalSource {
        type Value = String;
        type Map = Vec<(String, Self::Value)>;

        fn collect(&self) -> ConfigResult<Self::Map> {
            Ok(vec![("key".to_owned(), "value".to_owned())])
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Foo(String);

    struct PersistSourceImpl;
    impl PersistSource for PersistSourceImpl {
        type Value = Foo;

        fn source_name(&self) -> ConfigKey {
            "test".to_owned()
        }

        fn default(&self) -> Self::Value {
            Foo("hello".to_owned())
        }

        fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::from("test").join(self.source_name())
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Bar(String);

    struct SecretSourceImpl;
    impl SecretSource for SecretSourceImpl {
        type Value = Bar;

        fn source_name(&self) -> ConfigKey {
            "secret_test".to_owned()
        }

        fn default(&self) -> Self::Value {
            Bar("world".to_owned())
        }

        fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::from("test").join(self.source_name())
        }
    }

    #[test]
    fn source_test() {
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
        let patch = NormalSource
            .upgrade("key", &"new_value".to_owned())
            .unwrap();
        patch.apply(&mut config).unwrap();
        let v: String = config.get("key").unwrap();
        assert_eq!(v, "new_value");
        let patch = PersistSourceImpl.upgrade(&Foo("hi".to_owned())).unwrap();
        patch.apply(&mut config).unwrap();
        let v: Foo = config.get("test").unwrap();
        assert_eq!(v, Foo("hi".to_owned()));
        let patch = SecretSourceImpl.upgrade(&Bar("Louis".to_owned())).unwrap();
        patch.apply(&mut config).unwrap();
        let v: Bar = config.get("secret_test").unwrap();
        assert_eq!(v, Bar("Louis".to_owned()));
        std::fs::remove_file("test/secret_test").unwrap();
        std::fs::remove_file("test/test").unwrap();
    }
}
