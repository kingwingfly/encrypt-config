use encrypt_config::{Config, NormalSource};

#[derive(Default, NormalSource)]
struct Normal {
    value: i32,
}

#[test]
fn test_concurrent_logic() {}
