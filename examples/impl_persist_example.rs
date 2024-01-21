use encrypt_config::{Config, PersistSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Foo(String);

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
            std::path::PathBuf::from("tests").join("persist.conf")
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
}

fn main() {
    persist_test();
}
