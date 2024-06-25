#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(not(feature = "persist"), feature = "default_config_dir"))]
compile_error!("Feature `default_config_dir` only works with feature `persist` on.");
#[cfg(all(not(feature = "secret"), feature = "mock"))]
compile_error!("Feature `mock` is designed only for feature `secret` on.");
#[cfg(all(
    target_os = "linux",
    feature = "secret",
    not(any(
        feature = "linux-secret-service",
        feature = "linux-keyutils",
        feature = "mock"
    ))
))]
compile_error!("On Linux, there are two supported platform credential stores: the secret-service and the keyutils. You must enable one of the following features: `linux-secret-service` `linux-keyutils` `mock`.");
#[cfg(all(
    target_os = "linux",
    feature = "secret",
    any(
        all(feature = "linux-secret-service", feature = "linux-keyutils"),
        all(feature = "linux-secret-service", feature = "mock"),
        all(feature = "linux-keyutils", feature = "mock")
    )
))]
compile_error!("On Linux, Only one of the following features can be enabled: `linux-secret-service` `linux-keyutils` `mock`.");
#[cfg(all(
    target_os = "linux",
    not(feature = "secret"),
    any(feature = "linux-secret-service", feature = "linux-keyutils")
))]
compile_error!("On Linux, `linux-secret-service` `linux-keyutils` are only for `secret` on");
#[cfg(all(
    not(target_os = "linux")
    any(feature = "linux-secret-service", feature = "linux-keyutils"),
))]
compile_error!("`linux-secret-service` `linux-keyutils` are only for Linux.");

/// The output directory for the generated files when testing.
pub const TEST_OUT_DIR: &str = concat!(env!("OUT_DIR"), "/encrypt_config_cache");

pub mod config;
#[cfg(feature = "secret")]
pub mod encrypt_utils;
pub mod error;
pub mod source;

pub use config::Config;
pub use source::*;
