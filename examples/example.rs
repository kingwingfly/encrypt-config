use encrypt_config::{Config, PersistSource, SecretSource, Source};
use serde::{Deserialize, Serialize};

struct NormalSource;
impl Source for NormalSource {
    type Value = String;
    type Map = Vec<(String, Self::Value)>;

    fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
        Ok(vec![("key".to_owned(), "value".to_owned())])
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Foo(String);

struct PersistSourceImpl;
impl PersistSource for PersistSourceImpl {
    type Value = Foo;

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("tests").join("persist.conf")
    }

    #[cfg(feature = "default_config_dir")]
    fn source_name(&self) -> String {
        "persist_test".to_owned()
    }
}

struct SecretSourceImpl;
impl SecretSource for SecretSourceImpl {
    type Value = Foo;

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("tests").join("secret.conf")
    }

    #[cfg(feature = "default_config_dir")]
    fn source_name(&self) -> String {
        "secret_test".to_owned()
    }
}

fn config_tests() {
    let mut config = Config::new("test"); // Now it's empty
    config.add_source(NormalSource).unwrap();
    assert_eq!(config.get::<_, String>("key").unwrap(), "value");

    assert_eq!(config.get::<_, String>("key").unwrap(), "new value");

    config.add_persist_source(PersistSourceImpl).unwrap();
    let new_value = Foo("hello".to_owned());

    assert_eq!(config.get::<_, Foo>("persist").unwrap(), new_value);

    let mut config_new = Config::new("test");
    config_new.add_persist_source(PersistSourceImpl).unwrap(); // Read config from disk
    assert_eq!(config_new.get::<_, Foo>("persist").unwrap(), new_value);

    config.add_secret_source(SecretSourceImpl).unwrap();
    let new_value = Foo("world".to_owned());
    assert_eq!(config.get::<_, Foo>("secret").unwrap(), new_value);

    std::fs::remove_file("tests/persist.conf").unwrap();
    std::fs::remove_file("tests/secret.conf").unwrap();
}

fn main() {
    config_tests();
}
