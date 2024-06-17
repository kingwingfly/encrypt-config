#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(not(feature = "persist"), feature = "default_config_dir"))]
compile_error!("Feature `default_config_dir` only works with feature `persist` on.");
#[cfg(all(not(feature = "persist"), feature = "save_on_change"))]
compile_error!("Feature `save_on_change` is designed only for feature `persist` on.");
#[cfg(all(not(feature = "persist"), feature = "mock"))]
compile_error!("Feature `mock` is designed only for feature `persist` on.");

/// The output directory for the generated files when testing.
pub const TEST_OUT_DIR: &str = concat!(env!("OUT_DIR"), "/encrypt_config_cache");

mod config;
#[cfg(feature = "secret")]
mod encrypt_utils;
mod error;
mod source;

pub use config::Config;
pub use source::*;

#[cfg(feature = "derive")]
pub use encrypt_config_derive::*;
