use encrypt_config::{Config, PersistSource, SecretSource, Source};

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

fn main() {
    config_tests();
}
