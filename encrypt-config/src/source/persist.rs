use crate::error::ConfigResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A trait for persisted but not encrypted config source.
#[cfg(feature = "persist")]
pub trait PersistSource: Serialize + for<'de> Deserialize<'de> + Default {
    /// The path to persist the config file.
    #[cfg(not(feature = "default_config_dir"))]
    const PATH: &'static str;
    /// The name of the config file. Its parent directory is the OS' default config directory.
    #[cfg(feature = "default_config_dir")]
    const NAME: &'static str;

    /// Return the path to the config file.
    fn path() -> PathBuf {
        #[cfg(not(feature = "default_config_dir"))]
        let path = PathBuf::from(Self::PATH);
        #[cfg(feature = "default_config_dir")]
        let path = dirs::config_dir()
            .expect("Default config dir unknown in your OS.")
            .join(Self::NAME);
        path
    }

    /// Load the config from the file.
    fn load() -> ConfigResult<Self> {
        let path = Self::path();
        let file = std::fs::File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }

    /// Save the config to the file.
    fn save(&self) -> ConfigResult<()> {
        let path = Self::path();
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        let file = std::fs::File::create(path).unwrap();
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}
