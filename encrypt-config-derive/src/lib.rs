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
/// # Example
/// ```
/// # use encrypt_config_derive::NormalSource;
/// #[derive(Default, NormalSource)]
/// struct NormalConfig {
///     count: i32,
/// }
/// ```
#[proc_macro_derive(NormalSource)]
pub fn derive_normal_source(input: TokenStream) -> TokenStream {
    normal::derive_normal_source(input)
}

/// Derive macro for `PersistSource`.
/// # Example
/// ```
/// # use encrypt_config_derive::PersistSource;
/// # use serde::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize, Default, PersistSource)]
#[cfg_attr(
    feature = "default_config_dir",
    doc = "#[source(name = \"persist_config.json\")]"
)]
#[cfg_attr(
    not(feature = "default_config_dir"),
    doc = "#[source(path = \"/path/to/persist_config.json\")]"
)]
/// struct PersistConfig {
///    name: String,
///    age: i32,
/// }
/// ```
#[cfg(feature = "persist")]
#[proc_macro_derive(PersistSource, attributes(source))]
pub fn derive_persist_source(input: TokenStream) -> TokenStream {
    persist::derive_persist_source(input)
}

/// Derive macro for `SecretSource`.
/// # Example
/// ```
/// # use encrypt_config_derive::SecretSource;
/// # use serde::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize, Default, SecretSource)]
#[cfg_attr(
    feature = "default_config_dir",
    doc = "#[source(name = \"secret_config.json\", keyring_entry = \"secret\")]"
)]
#[cfg_attr(
    not(feature = "default_config_dir"),
    doc = "#[source(path = \"/path/to/secret_config\", keyring_entry = \"secret\")]"
)]
/// struct SecretConfig {
///    password: String,
/// }
/// ```
#[cfg(feature = "secret")]
#[proc_macro_derive(SecretSource, attributes(source))]
pub fn derive_secret_source(input: TokenStream) -> TokenStream {
    secret::derive_secret_source(input)
}
