use encrypt_config::{Config, NormalSource, PersistSource, SecretSource, TEST_OUT_DIR};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Default)]
struct NormalConfig {
    count: usize,
}

#[derive(Default, Serialize, Deserialize)]
struct PersistConfig {
    name: String,
    age: usize,
}

#[derive(Default, Serialize, Deserialize)]
struct SecretConfig {
    password: String,
}

fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut config = Config::default();
        // config.add_normal_source::<NormalConfig>();
        // config.add_persist_source::<PersistConfig>();
        // config.add_secret_source::<SecretConfig>();
        config
    })
}

fn main() {}
