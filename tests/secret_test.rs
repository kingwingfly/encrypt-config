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
    let config = Config::default();
    {
        let secret_config = config.get::<SecretConfig>();
        assert_eq!(secret_config.value, 0);
    }
    {
        let mut secret_config = config.get_mut::<SecretConfig>();
        secret_config.value = 42;
        assert_eq!(secret_config.value, 42);
    }

    let config = Config::default();
    {
        let secret_config = config.get::<SecretConfig>();
        assert_eq!(secret_config.value, 42);
    }
    {
        let (mut secret, _) = config.get_mut_many::<(SecretConfig,)>();
        secret.value = 0;
    }
    {
        let secret_config = config.get::<SecretConfig>();
        assert_eq!(secret_config.value, 0);
    }

    let config = Config::default();
    {
        let secret_config = config.get::<SecretConfig>();
        assert_eq!(secret_config.value, 0);
    }

    std::fs::remove_file(SecretConfig::path()).ok();
}
