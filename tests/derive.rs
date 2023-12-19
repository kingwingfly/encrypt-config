use encrypt_config::Source;
use std::collections::HashMap;

#[test]
fn derive_source_test() {
    #[derive(Source)]
    #[source(normal, default(vec![("key".to_owned(), "value".to_owned())]))]
    struct SourceImpl;
    assert_eq!(
        SourceImpl.collect().unwrap(),
        vec![("key".to_owned(), "value".to_owned())]
    );
    #[derive(Source)]
    #[source(normal, default(HashMap::from([("key".to_owned(), "value".to_owned())])))]
    struct SourceImpl_;
    assert_eq!(
        SourceImpl_.collect().unwrap(),
        vec![("key".to_owned(), "value".to_owned())]
    );
}
