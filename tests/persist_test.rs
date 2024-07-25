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
    {
        let cfg = Config::default();
        {
            let persist = cfg.get::<PersistConfig>();
            assert_eq!(persist.value, 0);
        }
        {
            let mut persist = cfg.get_mut::<PersistConfig>();
            persist.value = 42;
            assert_eq!(persist.value, 42);
        }
    }
    {
        let cfg = Config::default();
        let persist = cfg.get::<PersistConfig>();
        assert_eq!(persist.value, 42);
    }
    std::fs::remove_file(PersistConfig::path()).ok();
}
