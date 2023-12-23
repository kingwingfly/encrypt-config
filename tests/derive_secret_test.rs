use encrypt_config::SecretSource;
use std::collections::HashMap;

#[cfg(not(feature = "default_config_dir"))]
#[test]
fn derive_secret_test() {
    use serde::{Deserialize, Serialize};

    #[derive(SecretSource)]
    #[source(path("tests/secret.conf"), default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.default().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo(String);

    #[derive(SecretSource)]
    #[source(value(Foo), path("tests/secret.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
    struct SourceFoo;
    assert_eq!(
        SourceFoo.default().unwrap(),
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
        SourceArray.default().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );
}
