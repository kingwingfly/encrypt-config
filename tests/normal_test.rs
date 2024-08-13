use encrypt_config::{Config, NormalSource};

#[derive(Default, NormalSource)]
struct NormalConfig {
    value: i32,
}

#[test]
fn normal_test() {
    let cfg: Config<1> = Config::default();
    {
        let normal = cfg.get::<NormalConfig>();
        assert_eq!(normal.value, 0);
        let (normal,) = cfg.get_many::<(NormalConfig,)>();
        assert_eq!(normal.value, 0);
    }
    {
        let mut normal = cfg.get_mut::<NormalConfig>();
        normal.value = 42;
        assert_eq!(normal.value, 42);
    }
}

#[test]
#[should_panic]
fn write_while_reading() {
    let cfg: Config<1> = Config::default();
    let _normal_ref = cfg.get::<NormalConfig>();
    let _normal_mut = cfg.get_mut::<NormalConfig>();
}
