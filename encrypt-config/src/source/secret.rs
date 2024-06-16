use serde::{de::DeserializeOwned, Serialize};

/// A trait for persisted and encrypted config source.
#[cfg(feature = "secret")]
pub trait SecretSource {
    /// The type of the config value
    type Value: Serialize + DeserializeOwned;
    /// The type of the config map. It must be iterable, the first item of the tuple is the key, which should be `String` only.
    type Map: IntoIterator<Item = (String, Self::Value)>;

    /// The default config values from this source.
    /// This is the only way to add new config key-value pairs,
    /// because we cannot infer the source type(`normal`, `persist` and `secret`) of a new key after source merged into config if not so.
    fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>>;

    /// The name of the config file. Its parent directory is the OS' default config directory.
    /// It will be used as the file name if feature `default_config_dir` is on.
    #[cfg(feature = "default_config_dir")]
    fn source_name(&self) -> String;

    /// The path to persist the config file. Using the OS' default config directory.
    /// It will be used as the file path if feature `default_config_dir` is on.
    #[cfg(feature = "default_config_dir")]
    fn path(&self) -> std::path::PathBuf {
        dirs_next::config_dir()
            .expect("Default config dir unknown, please turn off feature `default_config_dir`")
            .join(self.source_name())
    }

    /// The path to persist the config file.
    #[cfg(not(feature = "default_config_dir"))]
    fn path(&self) -> std::path::PathBuf;
}
