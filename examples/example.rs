use encrypt_config::{Config, NormalSource, PersistSource, SecretSource};
use serde::{Deserialize, Serialize};

#[derive(Default, NormalSource)]
struct NormalConfig {
    count: usize,
}

#[derive(Default, Serialize, Deserialize, PersistSource)]
#[cfg_attr(feature = "default_config_dir", source(name = "persist_config.json"))]
#[cfg_attr(
    not(feature = "default_config_dir"),
    source(path = const_str::concat!(encrypt_config::TEST_OUT_DIR, "/persist_config.json"))
)]
struct PersistConfig {
    name: String,
    age: usize,
}

#[derive(Default, Serialize, Deserialize, SecretSource)]
#[cfg_attr(
    feature = "default_config_dir",
    source(name = "secret_config", keyring_entry = "secret")
)]
#[cfg_attr(
    not(feature = "default_config_dir"),
    source(path = const_str::concat!(encrypt_config::TEST_OUT_DIR, "/secret_config"), keyring_entry = "secret")
)]
struct SecretConfig {
    password: String,
}

fn main() {
    // clean before test
    let files = vec![PersistConfig::path(), SecretConfig::path()];
    for file in &files {
        std::fs::remove_file(file).ok();
    }
    {
        let cfg = Config::default();
        let normal = cfg.get::<NormalConfig>();
        // default value
        assert_eq!(normal.count, 0);
        let mut normal = cfg.get_mut::<NormalConfig>();
        normal.count = 42;
        assert_eq!(normal.count, 42);
    }
    {
        let cfg = Config::new();
        let mut persist = cfg.get_mut::<PersistConfig>();
        persist.name = "Louis".to_string();
        persist.age = 22;
        let mut secret = cfg.get_mut::<SecretConfig>();
        secret.password = "123456".to_string();
        // Changes will be saved automatically as Config is dropped
    }
    {
        // Assume this is a new config in the next start
        let cfg = Config::default();
        // normal config will not be saved
        assert_eq!(cfg.get::<NormalConfig>().count, 0);
        // persist config will be saved
        assert_eq!(cfg.get::<PersistConfig>().name, "Louis");
        // secret config will be encrypted
        assert_eq!(cfg.get::<SecretConfig>().password, "123456");

        // The secret config file should not be able to load directly
        let encrypted_file = std::fs::File::open(SecretConfig::path()).unwrap();
        assert!(serde_json::from_reader::<_, SecretConfig>(encrypted_file).is_err());

        // You can also save manually, but this will not refresh the config cache
        let persist = cfg.get::<PersistConfig>();
        persist.save().unwrap();
        // Instead, You can save in this way, this will refresh the cache
        cfg.save(SecretConfig {
            password: "123".to_owned(),
        })
        .unwrap();
    }
    {
        // Restart again
        let config = Config::default();
        assert_eq!(config.get::<SecretConfig>().password, "123");
    }
    // clean after test
    for file in files {
        std::fs::remove_file(file).ok();
    }
}
