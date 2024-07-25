use encrypt_config::{Config, NormalSource};
use loom::sync::Arc;
use loom::thread::{self, JoinHandle};

#[derive(Default, NormalSource)]
struct Normal {
    value: i32,
}

#[test]
#[should_panic]
fn write_while_reading() {
    loom::model(|| {
        let cfg = Config::default();
        let _normal_ref = cfg.get::<Normal>();
        let _normal_mut = cfg.get_mut::<Normal>();
    })
}

#[test]
#[should_panic]
fn write_while_writing() {
    loom::model(|| {
        let cfg = Config::default();
        let _normal_ref = cfg.get_mut::<Normal>();
        let _normal_mut = cfg.get_mut::<Normal>();
    })
}

#[test]
fn read_while_reading() {
    loom::model(|| {
        let cfg = Config::default();
        let _normal_ref1 = cfg.get::<Normal>();
        let _normal_ref2 = cfg.get::<Normal>();
    })
}

#[test]
fn multi_thread() {
    loom::model(|| {
        let cfg = Arc::new(Config::default());
        let jhs = (0..2)
            .map(|_| {
                let cfg = Arc::clone(&cfg);
                thread::spawn(move || {
                    let normal_ref = cfg.get::<Normal>();
                    assert_eq!(normal_ref.value, 0);
                })
            })
            .collect::<Vec<JoinHandle<_>>>();
        for jh in jhs {
            jh.join().unwrap();
        }
    })
}
