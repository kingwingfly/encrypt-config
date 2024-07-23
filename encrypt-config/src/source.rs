//! Source module for the encrypt-config crate.

#[cfg(feature = "secret")]
use crate::encrypt_utils::Encrypter;
use crate::error::ConfigResult;
#[cfg(feature = "derive")]
pub use encrypt_config_derive::*;
#[cfg(feature = "persist")]
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "persist")]
use std::path::PathBuf;

/// Source trait for the encrypt-config crate. You can impl your logic for loading and saving the configuration here.
/// Moreover, you can use derive macros to implement [`NormalSource`], [`PersistSource`], and [`SecretSource`] in this crate.
/// In provided ways, `Source` will be implemented when deriving, so that derived structs can be accepted by the [`Config`](crate::Config) struct.
pub trait Source: Default {
    /// Try to load the source, return error if failed.
    fn load() -> ConfigResult<Self>
    where
        Self: Sized;
    /// Save logic for the source.
    fn save(&self) -> ConfigResult<()>;

    /// Load logic for the source, return default value if failed.
    fn load_or_default() -> Self
    where
        Self: Sized + Default,
    {
        Self::load().unwrap_or_default()
    }
}

/// Normal source trait.
pub trait NormalSource: Source {}

/// Persist source trait.
#[cfg(feature = "persist")]
pub trait PersistSource: Source + Serialize + DeserializeOwned {
    /// Path for the persist source.
    #[cfg(not(feature = "default_config_dir"))]
    const PATH: &'static str;
    /// Name for the persist source.
    #[cfg(feature = "default_config_dir")]
    const NAME: &'static str;

    /// Path for the persist source.
    fn path() -> PathBuf {
        #[cfg(not(feature = "default_config_dir"))]
        {
            PathBuf::from(Self::PATH)
        }
        #[cfg(feature = "default_config_dir")]
        {
            dirs::config_dir()
                .expect("Default config dir unknown in your OS.")
                .join(Self::NAME)
        }
    }
    /// Load the persist source.
    fn load() -> ConfigResult<Self> {
        let path = Self::path();
        let file = std::fs::File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
    /// Save the persist source.
    fn save(&self) -> ConfigResult<()> {
        let path = Self::path();
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        let file = std::fs::File::create(path).unwrap();
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}

/// Secret source trait.
#[cfg(feature = "secret")]
pub trait SecretSource: Source + Serialize + DeserializeOwned {
    /// Path for the persist source.
    #[cfg(not(feature = "default_config_dir"))]
    const PATH: &'static str;
    /// Name for the persist source.
    #[cfg(feature = "default_config_dir")]
    const NAME: &'static str;
    /// Keyring entry for the secret source.
    const KEYRING_ENTRY: &'static str;

    /// Path for the persist source.
    fn path() -> PathBuf {
        #[cfg(not(feature = "default_config_dir"))]
        {
            PathBuf::from(Self::PATH)
        }
        #[cfg(feature = "default_config_dir")]
        {
            dirs::config_dir()
                .expect("Default config dir unknown in your OS.")
                .join(Self::NAME)
        }
    }
    /// Load the secret source.
    fn load() -> ConfigResult<Self> {
        let path = Self::path();
        let encrypter = Encrypter::new(Self::KEYRING_ENTRY)?;
        let file = std::fs::File::open(path)?;
        let encrypted: Vec<u8> = std::io::Read::bytes(file).collect::<Result<_, _>>()?;
        encrypter.decrypt(&encrypted)
    }
    /// Save the secret source.
    fn save(&self) -> ConfigResult<()> {
        use std::io::Write as _;

        let path = Self::path();
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        let encrypter = Encrypter::new(Self::KEYRING_ENTRY)?;
        let encrypted = encrypter.encrypt(self)?;
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(&encrypted)?;
        file.flush()?;
        Ok(())
    }
}
