use encrypt_config::{Config, PersistSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct PersistConfig {
    value: i32,
}

impl PersistSource for PersistConfig {
    #[cfg(not(feature = "default_config_dir"))]
    const PATH: &'static str =
        const_str::concat!(encrypt_config::TEST_OUT_DIR, "/persist_config.json");
    #[cfg(feature = "default_config_dir")]
    const NAME: &'static str = "persist_config.json";
}

#[test]
fn persist_test() {
    std::fs::remove_file(PersistConfig::path()).ok();
    let mut config = Config::default();
    config.add_persist_source::<PersistConfig>().unwrap();
    let persist_config = config.get::<PersistConfig>().unwrap();
    assert_eq!(persist_config.value, 0);
    config.release::<PersistConfig>().unwrap();
    let mut persist_config = config.get_mut::<PersistConfig>().unwrap();
    persist_config.value = 42;
    assert_eq!(persist_config.value, 42);
    persist_config.save().unwrap();

    let mut config = Config::default();
    config.add_persist_source::<PersistConfig>().unwrap();
    let persist_config = config.get::<PersistConfig>().unwrap();
    assert_eq!(persist_config.value, 42);
    std::fs::remove_file(PersistConfig::path()).ok();
}
