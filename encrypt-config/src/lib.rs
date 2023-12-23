#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#[cfg(all(not(feature = "persist"), feature = "default_config_dir"))]
compile_error!("Feature `default_config_dir` only works with feature `persist` on.");
#[cfg(all(not(feature = "secret"), feature = "mock"))]
compile_error!("Feature `mock` is designed only for feature `secret` on.");

mod config;
#[cfg(feature = "secret")]
mod encrypt_utils;
mod error;
mod source;

pub use config::Config;
pub use error::*;
pub use source::*;

#[cfg(feature = "derive")]
pub use encrypt_config_derive::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[cfg_attr(test, derive(PartialEq, Debug))]
    struct Foo(String);

    #[test]
    fn normal_test() {
        struct NormalSource;
        impl Source for NormalSource {
            type Value = String;
            type Map = Vec<(String, Self::Value)>;

            fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                Ok(vec![("normal".to_owned(), "value".to_owned())])
            }
        }

        let mut config = Config::new(
            #[cfg(feature = "secret")]
            "test",
        ); // Now it's empty

        config.add_source(NormalSource).unwrap();
        assert_eq!(config.get::<_, String>("normal").unwrap(), "value");

        let new_expect = "new value".to_owned();
        config.upgrade("normal", &new_expect).unwrap();
        assert_eq!(config.get::<_, String>("normal").unwrap(), new_expect);
    }

    #[cfg(feature = "persist")]
    #[test]
    fn persist_test() {
        struct PersistSourceImpl;
        impl PersistSource for PersistSourceImpl {
            type Value = Foo;
            type Map = Vec<(String, Self::Value)>;

            fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                Ok(vec![("persist".to_owned(), Foo("persist".to_owned()))])
            }

            #[cfg(not(feature = "default_config_dir"))]
            fn path(&self) -> std::path::PathBuf {
                std::path::PathBuf::from("../tests").join("persist.conf")
            }

            #[cfg(feature = "default_config_dir")]
            fn source_name(&self) -> String {
                "persist.conf".to_owned()
            }
        }

        let mut config = Config::new(
            #[cfg(feature = "secret")]
            "test",
        ); // Now it's empty
        let expect = Foo("persist".to_owned());

        config.add_persist_source(PersistSourceImpl).unwrap(); // This will persist the config if feature `save_on_change` on
        assert_eq!(config.get::<_, Foo>("persist").unwrap(), expect);

        // upgrade tests
        let new_expect1 = Foo("new persist".to_owned());
        config.upgrade_all([("persist", &new_expect1)]).unwrap();
        assert_eq!(config.get::<_, Foo>("persist").unwrap(), new_expect1);
        let mut config_new = Config::new(
            #[cfg(feature = "secret")]
            "test",
        ); // Now it's empty
        config_new.add_persist_source(PersistSourceImpl).unwrap(); // Read persist config from disk
        assert_eq!(config_new.get::<_, Foo>("persist").unwrap(), new_expect1);

        std::fs::remove_file(PersistSourceImpl.path()).unwrap();
    }

    #[cfg(feature = "secret")]
    #[test]
    fn secret_test() {
        struct SecretSourceImpl;
        impl SecretSource for SecretSourceImpl {
            type Value = Foo;
            type Map = Vec<(String, Self::Value)>;

            #[cfg(not(feature = "default_config_dir"))]
            fn path(&self) -> std::path::PathBuf {
                std::path::PathBuf::from("../tests").join("secret.conf")
            }

            #[cfg(feature = "default_config_dir")]
            fn source_name(&self) -> String {
                "secret.conf".to_owned()
            }

            fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                Ok(vec![("secret".to_owned(), Foo("secret".to_owned()))])
            }
        }

        let mut config = Config::new("test"); // Now it's empty
        let expect = Foo("secret".to_owned());
        config.add_secret_source(SecretSourceImpl).unwrap();
        assert_eq!(config.get::<_, Foo>("secret").unwrap(), expect);

        let new_expect2 = Foo("new secret".to_owned());
        config.upgrade("secret", &new_expect2).unwrap();
        assert_eq!(config.get::<_, Foo>("secret").unwrap(), new_expect2);
        let mut config_new = Config::new("test");
        config_new.add_secret_source(SecretSourceImpl).unwrap(); // Read secret config from disk
        assert_eq!(config_new.get::<_, Foo>("secret").unwrap(), new_expect2); // The persist source is brought back

        std::fs::remove_file(SecretSourceImpl.path()).unwrap();
    }
}
