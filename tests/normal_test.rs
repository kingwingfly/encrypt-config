use encrypt_config::{Config, NormalSource};

#[derive(Default, NormalSource)]
struct NormalConfig {
    value: i32,
}

#[test]
fn normal_test() {
    let cfg = Config::default();
    {
        let normal = cfg.get::<NormalConfig>();
        assert_eq!(normal.value, 0);
        let (normal, _) = cfg.get_many::<(NormalConfig,)>();
        assert_eq!(normal.value, 0);

        let mut normal = cfg.get_mut::<NormalConfig>();
        normal.value = 42;
        assert_eq!(normal.value, 42);
    }
    {
        let normal = cfg.take::<NormalConfig>();
        assert_eq!(normal.value, 42);
    }
}
