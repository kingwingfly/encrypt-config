use encrypt_config::{Config, NormalSource};

struct NormalConfig {
    value: i32,
}

impl NormalSource for NormalConfig {}

#[test]
fn normal_test() {
    let mut config = Config::default();
    config
        .add_normal_source(NormalConfig { value: 42 })
        .unwrap();
    {
        let normal_config = config.get::<NormalConfig>().unwrap();
        assert_eq!(normal_config.value, 42);
    }
    let mut normal_config = config.get_mut::<NormalConfig>().unwrap();
    normal_config.value = 0;
    assert_eq!(normal_config.value, 0);
}
