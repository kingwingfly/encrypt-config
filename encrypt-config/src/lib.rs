#![doc = include_str!("../README.md")]
#![deny(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    elided_lifetimes_in_paths
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(not(feature = "persist"), feature = "default_config_dir"))]
compile_error!("Feature `default_config_dir` only works with feature `persist` on.");
#[cfg(all(not(feature = "secret"), feature = "mock"))]
compile_error!("Feature `mock` is designed only for feature `secret` on.");

/// The output directory for the generated files when testing.
pub const TEST_OUT_DIR: &str = concat!(env!("OUT_DIR"), "/encrypt_config_cache");

pub mod config;
#[cfg(feature = "secret")]
pub mod encrypt_utils;
pub mod error;
pub mod source;

pub use config::Config;
#[cfg(feature = "derive")]
pub use encrypt_config_derive::*;
pub use source::*;
