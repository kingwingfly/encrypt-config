//! Source module for the encrypt-config crate.

#[cfg(feature = "secret")]
use crate::encrypt_utils::Encrypter;
#[cfg(feature = "persist")]
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "persist")]
use std::path::PathBuf;

pub use rom_cache::Cacheable;

/// Normal source trait.
pub trait NormalSource: rom_cache::Cacheable {}

/// Persist source trait.
#[cfg(feature = "persist")]
pub trait PersistSource: rom_cache::Cacheable + Serialize + DeserializeOwned {
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
    fn load() -> std::io::Result<Self> {
        let path = Self::path();
        let file = std::fs::File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
    /// Save the persist source.
    fn store(&self) -> std::io::Result<()> {
        let path = Self::path();
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        let file = std::fs::File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}

/// Secret source trait.
#[cfg(feature = "secret")]
pub trait SecretSource: rom_cache::Cacheable + Serialize + DeserializeOwned {
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
    fn load() -> ::std::io::Result<Self> {
        let path = Self::path();
        let encrypter =
            Encrypter::new(Self::KEYRING_ENTRY).map_err(|_| std::io::ErrorKind::InvalidData)?;
        let file = std::fs::File::open(path)?;
        let encrypted: Vec<u8> = std::io::Read::bytes(file).collect::<Result<_, _>>()?;
        encrypter
            .decrypt(&encrypted)
            .map_err(|_| std::io::ErrorKind::InvalidData.into())
    }
    /// Save the secret source.
    fn store(&self) -> ::std::io::Result<()> {
        use std::io::Write as _;

        let path = Self::path();
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        let encrypter =
            Encrypter::new(Self::KEYRING_ENTRY).map_err(|_| std::io::ErrorKind::InvalidData)?;
        let encrypted = encrypter
            .encrypt(self)
            .map_err(|_| std::io::ErrorKind::InvalidData)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(&encrypted)?;
        file.flush()?;
        Ok(())
    }
}
