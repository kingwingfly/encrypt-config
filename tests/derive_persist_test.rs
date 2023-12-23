use encrypt_config::PersistSource;
use std::collections::HashMap;

#[cfg(not(feature = "default_config_dir"))]
#[test]
fn derive_persist_test() {
    use serde::{Deserialize, Serialize};

    #[derive(PersistSource)]
    #[source(path("tests/persist.conf"), default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.default().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo(String);

    #[derive(PersistSource)]
    #[source(value(Foo), path("tests/persist.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
    struct SourceFoo;
    assert_eq!(
        SourceFoo.default().unwrap(),
        HashMap::from([("key".to_owned(), Foo("value".to_owned()))])
    );
}

#[cfg(feature = "default_config_dir")]
#[test]
fn derive_persist_test_default_coonfig_dir() {
    #[derive(PersistSource)]
    #[source(source_name("persist.conf"), default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.default().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );
}
