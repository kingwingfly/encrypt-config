use encrypt_config::{Config, Source};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Foo(String);

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

fn main() {
    normal_test();
}
