use encrypt_config::{Config, PersistSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PersistSource)]
#[cfg_attr(feature = "default_config_dir", source(name = "persist_config.json"))]
#[cfg_attr(
    not(feature = "default_config_dir"),
    source(path = const_str::concat!(encrypt_config::TEST_OUT_DIR, "/persist_config.json"))
)]
struct PersistConfig {
    value: i32,
}

#[test]
fn persist_test() {
    std::fs::remove_file(PersistConfig::path()).ok();
    let mut config = Config::default();
    config.load_source::<PersistConfig>();
    {
        let persist_config = config.get::<PersistConfig>().unwrap();
        assert_eq!(persist_config.value, 0);
    }
    let mut persist_config = config.get_mut::<PersistConfig>().unwrap();
    persist_config.value = 42;
    assert_eq!(persist_config.value, 42);
    persist_config.save().unwrap();

    let mut config = Config::default();
    config.load_source::<PersistConfig>();
    let persist_config = config.get::<PersistConfig>().unwrap();
    assert_eq!(persist_config.value, 42);
    std::fs::remove_file(PersistConfig::path()).ok();
}
