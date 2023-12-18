#![doc = include_str!("../README.md")]

mod config;
mod encrypt_utils;
mod error;
mod source;

pub use config::{Config, ConfigPatch, SecretConfigPatch};
pub use error::*;
pub use source::{PersistSource, SecretSource, Source};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use super::*;

    struct NormalSource;
    impl Source for NormalSource {
        type Value = String;
        type Map = Vec<(String, Self::Value)>;

        fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
            Ok(vec![("key".to_owned(), "value".to_owned())])
        }
    }

    #[derive(Serialize, Deserialize)]
    #[cfg_attr(test, derive(PartialEq, Debug))]
    struct Foo(String);

    struct PersistSourceImpl;
    impl PersistSource for PersistSourceImpl {
        type Value = Foo;

        fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::from("tests").join("persist.conf")
        }
    }

    struct SecretSourceImpl;
    impl SecretSource for SecretSourceImpl {
        type Value = Foo;

        fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::from("tests").join("secret.conf")
        }
    }

    #[test]
    fn config_tests() {
        let mut config = Config::new("test"); // Now it's empty
        config.add_source(NormalSource).unwrap();
        assert_eq!(config.get::<_, String>("key").unwrap(), "value");
        let patch = NormalSource.upgrade("key", &"new value".to_owned());
        patch.apply(&mut config).unwrap();
        assert_eq!(config.get::<_, String>("key").unwrap(), "new value");

        config.add_persist_source(PersistSourceImpl).unwrap();
        let new_value = Foo("hello".to_owned());
        let patch = PersistSourceImpl.upgrade("persist", &new_value);
        patch.apply(&mut config).unwrap();
        assert_eq!(config.get::<_, Foo>("persist").unwrap(), new_value);

        let mut config_new = Config::new("test");
        config_new.add_persist_source(PersistSourceImpl).unwrap(); // Read config from disk
        assert_eq!(config_new.get::<_, Foo>("persist").unwrap(), new_value);

        config.add_secret_source(SecretSourceImpl).unwrap();
        let new_value = Foo("world".to_owned());
        let patch = SecretSourceImpl.upgrade("secret", &new_value);
        patch.apply(&mut config).unwrap();
        assert_eq!(config.get::<_, Foo>("secret").unwrap(), new_value);

        std::fs::remove_file("tests/persist.conf").unwrap();
        std::fs::remove_file("tests/secret.conf").unwrap();
    }

    #[test]
    fn default_test() {
        struct DefaultSource;
        impl PersistSource for DefaultSource {
            type Value = String;

            fn path(&self) -> std::path::PathBuf {
                std::path::PathBuf::from("tests").join("default.conf")
            }

            fn default(&self) -> HashMap<String, Self::Value> {
                HashMap::from([("key".to_owned(), "value".to_owned())])
            }
        }

        let mut config = Config::new("test");
        config.add_persist_source(DefaultSource).unwrap();
        assert_eq!(config.get::<_, String>("key").unwrap(), "value");
    }
}
