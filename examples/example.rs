use encrypt_config::{Config, PersistSource, SecretSource, Source};
use serde::{Deserialize, Serialize};

struct NormalSource;
impl Source for NormalSource {
    type Value = String;
    type Map = Vec<(String, Self::Value)>;

    fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
        Ok(vec![("key".to_owned(), "value".to_owned())])
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Foo(String);

struct PersistSourceImpl;
impl PersistSource for PersistSourceImpl {
    type Value = Foo;
    type Map = Vec<(String, Self::Value)>;

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("tests").join("persist.conf")
    }

    #[cfg(feature = "default_config_dir")]
    fn source_name(&self) -> String {
        "persist.conf".to_owned()
    }

    fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
        Ok(vec![("persist".to_owned(), Foo("persist".to_owned()))])
    }
}

struct SecretSourceImpl;
impl SecretSource for SecretSourceImpl {
    type Value = Foo;
    type Map = Vec<(String, Self::Value)>;

    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("tests").join("secret.conf")
    }

    #[cfg(feature = "default_config_dir")]
    fn source_name(&self) -> String {
        "secret.conf".to_owned()
    }

    fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
        Ok(vec![("secret".to_owned(), Foo("secret".to_owned()))])
    }
}

fn config_tests() {
    let mut config = Config::new("test"); // Now it's empty
    config.add_source(NormalSource).unwrap();
    assert_eq!(config.get::<_, String>("key").unwrap(), "value");

    let expect = Foo("persist".to_owned());
    config.add_persist_source(PersistSourceImpl).unwrap(); // This will persist the config if feature `save_on_change` on
    assert_eq!(config.get::<_, Foo>("persist").unwrap(), expect);

    let mut config_new = Config::new("test");
    config_new.add_persist_source(PersistSourceImpl).unwrap(); // Read config from disk
    assert_eq!(config_new.get::<_, Foo>("persist").unwrap(), expect); // The persist source is brought back
    assert!(config_new.get::<_, String>("key").is_err()); // The normal source is forgotten

    let expect = Foo("secret".to_owned());
    config.add_secret_source(SecretSourceImpl).unwrap();
    assert_eq!(config.get::<_, Foo>("secret").unwrap(), expect);

    // upgrade tests
    let new_expect = "new value".to_owned();
    config.upgrade("key", &new_expect).unwrap();
    assert_eq!(config.get::<_, String>("persist").unwrap(), new_expect);

    let new_expect1 = Foo("new persist".to_owned());
    let new_expect2 = Foo("new secret".to_owned());
    config
        .upgrade_all([("persist", &new_expect1), ("secret", &new_expect2)])
        .unwrap();
    assert_eq!(config.get::<_, Foo>("persist").unwrap(), new_expect1);
    assert_eq!(config.get::<_, Foo>("secret").unwrap(), new_expect2);

    let mut config_new = Config::new("test");
    config_new.add_persist_source(PersistSourceImpl).unwrap(); // Read persist config from disk
    config_new.add_secret_source(SecretSourceImpl).unwrap(); // Read secret config from disk
    assert_eq!(config_new.get::<_, Foo>("persist").unwrap(), new_expect1);
    assert_eq!(config_new.get::<_, Foo>("secret").unwrap(), new_expect2); // The persist source is brought back

    std::fs::remove_file(PersistSourceImpl.path()).unwrap();
    std::fs::remove_file(SecretSourceImpl.path()).unwrap();
}

fn main() {
    config_tests();
}
