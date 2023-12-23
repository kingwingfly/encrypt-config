use encrypt_config::Source;
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
        SourceVec.default().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Source)]
    #[source(default(HashMap::from([("key".to_owned(), "value".to_owned())])))]
    struct SourceHashMap;
    assert_eq!(
        SourceHashMap.default().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Source)]
    #[source(default([("key".to_owned(), "value".to_owned())]))]
    struct SourceArray;
    assert_eq!(
        SourceArray.default().unwrap(),
        HashMap::from([("key".to_owned(), "value".to_owned())])
    );

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo(String);
    #[derive(Source)]
    #[source(value(Foo), default([("key".to_owned(), Foo("value".to_owned()))]))]
    struct SourceFoo;
    assert_eq!(
        SourceFoo.default().unwrap(),
        HashMap::from([("key".to_owned(), Foo("value".to_owned()))])
    );
}
