use encrypt_config::{Config, SecretSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Foo(String);

fn secret_test() {
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

    let mut config = Config::new(
        #[cfg(feature = "secret")]
        "test",
    ); // Now it's empty
    let expect = Foo("secret".to_owned());
    config.add_secret_source(SecretSourceImpl).unwrap();
    assert_eq!(config.get::<_, Foo>("secret").unwrap(), expect);

    let new_expect2 = Foo("new secret".to_owned());
    config.upgrade("secret", &new_expect2).unwrap();
    assert_eq!(config.get::<_, Foo>("secret").unwrap(), new_expect2);
    let mut config_new = Config::new(
        #[cfg(feature = "secret")]
        "test",
    );
    config_new.add_secret_source(SecretSourceImpl).unwrap(); // Read secret config from disk
    assert_eq!(config_new.get::<_, Foo>("secret").unwrap(), new_expect2); // The persist source is brought back

    std::fs::remove_file(SecretSourceImpl.path()).unwrap();
}

fn main() {
    secret_test();
}
