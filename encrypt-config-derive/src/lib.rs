//! Derive macros for [encrypt-config](https://crates.io/crates/encrypt_config)

#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(not(feature = "persist"), feature = "default_config_dir"))]
compile_error!("Feature `default_config_dir` only works with feature `persist` on.");

mod normal;
#[cfg(feature = "persist")]
mod persist;
#[cfg(feature = "secret")]
mod secret;

use proc_macro::TokenStream;

/// Derive macro for `NormalSource`.
#[proc_macro_derive(NormalSource)]
pub fn derive_normal_source(input: TokenStream) -> TokenStream {
    normal::derive_normal_source(input)
}

#[cfg(feature = "persist")]
/// Derive macro for `PersistSource`.
#[proc_macro_derive(PersistSource, attributes(source))]
pub fn derive_persist_source(input: TokenStream) -> TokenStream {
    persist::derive_persist_source(input)
}

#[cfg(feature = "secret")]
/// Derive macro for `SecretSource`.
#[proc_macro_derive(SecretSource, attributes(source))]
pub fn derive_secret_source(input: TokenStream) -> TokenStream {
    secret::derive_secret_source(input)
}
