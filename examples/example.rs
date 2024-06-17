use encrypt_config::{Config, NormalSource, PersistSource, SecretSource, TEST_OUT_DIR};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Default, NormalSource)]
struct NormalConfig {
    count: usize,
}

#[derive(Default, Serialize, Deserialize, PersistSource)]
#[source(path = const_str::concat!(TEST_OUT_DIR, "/persist_config.json"))]
struct PersistConfig {
    name: String,
    age: usize,
}

#[derive(Default, Serialize, Deserialize, SecretSource)]
#[source(path = const_str::concat!(TEST_OUT_DIR, "/secret_config.json"), keyring_entry = "secret")]
struct SecretConfig {
    password: String,
}

fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut config = Config::default();
        config.add_normal_source::<NormalConfig>().unwrap();
        config.add_persist_source::<PersistConfig>().unwrap();
        config.add_secret_source::<SecretConfig>().unwrap();
        config
    })
}

fn main() {
    // clean before test
    let files = vec![PersistConfig::path(), SecretConfig::path()];
    for file in &files {
        std::fs::remove_file(file).ok();
    }

    let cfg = config();
    {
        let normal_config = cfg.get::<NormalConfig>().unwrap();
        assert_eq!(normal_config.count, 0);
    }
    let mut normal_config = cfg.get_mut::<NormalConfig>().unwrap();
    normal_config.count = 42;
    assert_eq!(normal_config.count, 42);

    let jh = std::thread::spawn(|| {
        // work in another thread
        let cfg = config();
        let mut persist_config = cfg.get_mut::<PersistConfig>().unwrap();
        persist_config.name = "Louis".to_string();
        persist_config.age = 22;
        // save to file
        persist_config.save().unwrap();
    });
    let cfg = config();
    let mut secret_config = cfg.get_mut::<SecretConfig>().unwrap();
    secret_config.password = "123456".to_string();
    // encrypt and save to file
    secret_config.save().unwrap();
    jh.join().unwrap();

    // let's new a config in the next start
    let mut config = Config::default();
    config.add_normal_source::<NormalConfig>().unwrap();
    config.add_persist_source::<PersistConfig>().unwrap();
    config.add_secret_source::<SecretConfig>().unwrap();

    // normal config will not be saved
    assert_eq!(config.get::<NormalConfig>().unwrap().count, 0);
    // persist config will be saved
    assert_eq!(config.get::<PersistConfig>().unwrap().name, "Louis");
    // secret config will be encrypted
    assert_eq!(config.get::<SecretConfig>().unwrap().password, "123456");

    // The secret config file should not be able to load directly
    let encrypted_file = std::fs::File::open(SecretConfig::path()).unwrap();
    assert!(serde_json::from_reader::<_, SecretConfig>(encrypted_file).is_err());

    // clean after test
    for file in files {
        std::fs::remove_file(file).ok();
    }
}
