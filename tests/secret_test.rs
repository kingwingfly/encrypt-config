use encrypt_config::{Config, SecretSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct SecretConfig {
    value: i32,
}

impl SecretSource for SecretConfig {
    #[cfg(not(feature = "default_config_dir"))]
    const PATH: &'static str =
        const_str::concat!(encrypt_config::TEST_OUT_DIR, "/secret_config.json");
    #[cfg(feature = "default_config_dir")]
    const NAME: &'static str = "secret_config.json";

    const KEY_ENTRY: &'static str = "secret";
}

#[test]
fn secret_test() {
    std::fs::remove_file(SecretConfig::path()).ok();
    let mut config = Config::default();
    config.add_secret_source::<SecretConfig>().unwrap();
    {
        let secret_config = config.get::<SecretConfig>().unwrap();
        assert_eq!(secret_config.value, 0);
    }
    let mut secret_config = config.get_mut::<SecretConfig>().unwrap();
    secret_config.value = 42;
    assert_eq!(secret_config.value, 42);
    secret_config.save().unwrap();

    let mut config = Config::default();
    config.add_secret_source::<SecretConfig>().unwrap();
    let secret_config = config.get::<SecretConfig>().unwrap();
    assert_eq!(secret_config.value, 42);
    std::fs::remove_file(SecretConfig::path()).ok();
}
