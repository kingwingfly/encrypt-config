use encrypt_config::{Config, NormalSource};

#[derive(Default, NormalSource)]
struct NormalConfig {
    value: i32,
}

#[test]
fn normal_test() {
    let config = Config::default();
    {
        let normal_config = config.get::<NormalConfig>();
        assert_eq!(normal_config.value, 0);
    }
    {
        let mut normal_config = config.get_mut::<NormalConfig>();
        normal_config.value = 42;
        assert_eq!(normal_config.value, 42);
    }
    {
        let normal_config = config.take::<NormalConfig>();
        assert_eq!(normal_config.value, 42);
    }
}
