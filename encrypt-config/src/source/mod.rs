//! Source module for the encrypt-config crate.

mod normal;
#[cfg(feature = "persist")]
mod persist;
#[cfg(feature = "secret")]
mod secret;

pub use normal::*;
#[cfg(feature = "persist")]
pub use persist::*;
#[cfg(feature = "secret")]
pub use secret::*;

use crate::error::ConfigResult;

/// Source trait for the encrypt-config crate. You can impl your logic for loading and saving the configuration here.
/// Moreover, you can use derive macros to implement [`NormalSource`], [`PersistSource`], and [`SecretSource`] in this crate.
/// In provided ways, `Source` will be implemented when deriving, so that derived structs can be accepted by the [`Config`](crate::Config) struct.
pub trait Source: Default {
    /// Load logic for the source, return default value is recommended.
    fn load() -> Self
    where
        Self: Sized;
    /// Save logic for the source.
    fn save(&self) -> ConfigResult<()>;
}
