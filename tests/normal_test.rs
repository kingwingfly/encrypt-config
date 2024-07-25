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
        let (normal,) = cfg.get_many::<(NormalConfig,)>();
        assert_eq!(normal.value, 0);
    }
    {
        let mut normal = cfg.get_mut::<NormalConfig>();
        normal.value = 42;
        assert_eq!(normal.value, 42);
    }
    {
        let normal = cfg.take::<NormalConfig>();
        assert_eq!(normal.value, 42);
    }
}

#[test]
#[should_panic]
fn too_many_readings() {
    let cfg = Config::default();
    let normals = (0..=32)
        .map(|_| cfg.get::<NormalConfig>())
        .collect::<Vec<_>>();
    assert_eq!(normals.len(), 33);
}

#[test]
#[should_panic]
fn write_while_reading() {
    let cfg = Config::default();
    let _normal_ref = cfg.get::<NormalConfig>();
    let _normal_mut = cfg.get_mut::<NormalConfig>();
}

macro_rules! many_normal {
    ($($t: ident),+) => {
        $(
        #[derive(Default, NormalSource)]
        struct $t {
            value: i32,
        }
        )+
    };
}

many_normal!(N1, N2, N3, N4, N5, N6, N7, N8);

#[test]
fn many_normal_test() {
    let cfg = Config::default();
    {
        let (n1, n2, n3, n4, n5, n6, n7, n8) = cfg.get_many::<(N1, N2, N3, N4, N5, N6, N7, N8)>();
        assert_eq!(n1.value, 0);
        assert_eq!(n2.value, 0);
        assert_eq!(n3.value, 0);
        assert_eq!(n4.value, 0);
        assert_eq!(n5.value, 0);
        assert_eq!(n6.value, 0);
        assert_eq!(n7.value, 0);
        assert_eq!(n8.value, 0);
    }
    {
        let (_n1, _n2, _n3, _n4, _n5, _n6, _n7, _n8) =
            cfg.get_mut_many::<(N1, N2, N3, N4, N5, N6, N7, N8)>();
    }
}
