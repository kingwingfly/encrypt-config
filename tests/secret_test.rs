use encrypt_config::{Config, SecretSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, SecretSource)]
#[cfg_attr(
    feature = "default_config_dir",
    source(name = "secret_config", keyring_entry = "secret")
)]
#[cfg_attr(
    not(feature = "default_config_dir"),
    source(path = const_str::concat!(encrypt_config::TEST_OUT_DIR, "/secret_config"), keyring_entry = "secret")
)]
struct SecretConfig {
    value: i32,
}

#[test]
fn secret_test() {
    std::fs::remove_file(SecretConfig::path()).ok();
    let mut config = Config::default();
    {
        let secret_config = config.get::<SecretConfig>().unwrap();
        assert_eq!(secret_config.value, 0);
    }
    {
        let mut secret_config = config.get_mut::<SecretConfig>().unwrap();
        secret_config.value = 42;
        assert_eq!(secret_config.value, 42);
    }

    let mut config = Config::default();
    let secret_config = config.get::<SecretConfig>().unwrap();
    assert_eq!(secret_config.value, 42);
    std::fs::remove_file(SecretConfig::path()).ok();
}
