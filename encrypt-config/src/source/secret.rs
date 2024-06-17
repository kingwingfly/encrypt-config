use super::PersistSource;

/// A trait for persisted and encrypted config source.
#[cfg(feature = "secret")]
pub trait SecretSource: PersistSource {}
