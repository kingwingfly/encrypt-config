use encrypt_config::{Config, NormalSource, PersistSource, SecretSource};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

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

fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(Config::default)
}

fn main() {
    // clean before test
    let files = vec![PersistConfig::path(), SecretConfig::path()];
    for file in &files {
        std::fs::remove_file(file).ok();
    }

    let cfg = config();
    {
        let normal_config = cfg.get::<NormalConfig>();
        assert_eq!(normal_config.count, 0);
    }
    {
        let mut normal_config = cfg.get_mut::<NormalConfig>();
        normal_config.count = 42;
        assert_eq!(normal_config.count, 42);
    }
    let jh = std::thread::spawn(move || {
        // work in another thread
        let mut persist_config = cfg.get_mut::<PersistConfig>();
        persist_config.name = "Louis".to_string();
        persist_config.age = 22;
        // change saved automatically after dropped
    });
    {
        let cfg = config();
        let mut secret_config = cfg.get_mut::<SecretConfig>();
        secret_config.password = "123456".to_string();
    }
    jh.join().unwrap();

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

    // You can also save manually
    let persist_config = cfg.get::<PersistConfig>();
    persist_config.save().unwrap();
    // You can also save in this way
    cfg.save(SecretConfig {
        password: "123".to_owned(),
    })
    .unwrap();

    // Restart again
    let cfg = Config::default();
    assert_eq!(cfg.get::<SecretConfig>().password, "123");
    // You can also get multiple configs at once
    let (_normal, _persist, _secret) =
        cfg.get_many::<(NormalConfig, PersistConfig, SecretConfig)>();

    // clean after test
    for file in files {
        std::fs::remove_file(file).ok();
    }
}
