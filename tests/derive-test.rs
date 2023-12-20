use encrypt_config::{PersistSource, SecretSource, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[test]
fn derive_normal_test() {
    #[derive(Source)]
    struct SourceNoDefault;

    #[derive(Source)]
    #[source(default(vec![("key".to_owned(), "value".to_owned())]))]
    struct SourceVec;
    assert_eq!(
        SourceVec.collect().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Source)]
    #[source(default(HashMap::from([("key".to_owned(), "value".to_owned())])))]
    struct SourceHashMap;
    assert_eq!(
        SourceHashMap.collect().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Source)]
    #[source(default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.collect().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo(String);
    #[derive(Source)]
    #[source(value(Foo), default([("key".to_owned(), Foo("value".to_owned()))]))]
    struct SourceFoo;
    assert_eq!(
        SourceFoo.collect().unwrap(),
        HashMap::from([("key".to_owned(), Foo("value".to_owned()))])
    );
}

#[cfg(not(feature = "default_config_dir"))]
#[test]
fn derive_persist_test() {
    #[derive(PersistSource)]
    #[source(path("tests/persist.conf"), default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.default(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo(String);

    #[derive(PersistSource)]
    #[source(value(Foo), path("tests/persist.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
    struct SourceFoo;
    assert_eq!(
        SourceFoo.default(),
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
        SourceArray.default(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );
}

#[cfg(not(feature = "default_config_dir"))]
#[test]
fn derive_secret_test() {
    #[derive(SecretSource)]
    #[source(path("tests/secret.conf"), default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.default(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo(String);

    #[derive(SecretSource)]
    #[source(value(Foo), path("tests/secret.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
    struct SourceFoo;
    assert_eq!(
        SourceFoo.default(),
        HashMap::from([("key".to_owned(), Foo("value".to_owned()))])
    );
}

#[cfg(feature = "default_config_dir")]
#[test]
fn derive_secret_test_default_coonfig_dir() {
    #[derive(SecretSource)]
    #[source(source_name("secret.conf"), default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.default(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );
}
