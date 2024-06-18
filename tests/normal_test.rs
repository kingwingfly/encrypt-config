use encrypt_config::{Config, NormalSource};

#[derive(Default)]
struct NormalConfig {
    value: i32,
}

impl NormalSource for NormalConfig {}

#[test]
fn normal_test() {
    let mut config = Config::default();
    config.add_normal_source::<NormalConfig>().unwrap();
    {
        let normal_config = config.get::<NormalConfig>().unwrap();
        assert_eq!(normal_config.value, 0);
    }
    {
        let mut normal_config = config.get_mut::<NormalConfig>().unwrap();
        normal_config.value = 42;
        assert_eq!(normal_config.value, 42);
    }
    {
        let normal_config = config.take::<NormalConfig>().unwrap();
        assert_eq!(normal_config.value, 42);
    }
}
