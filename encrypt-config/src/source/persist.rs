use serde::{de::DeserializeOwned, Serialize};

/// A trait for persisted but not encrypted config source.
/// # Example
/// ```no_run
/// use encrypt_config::{Config, PersistSource};
/// use serde::{Deserialize, Serialize};
///
/// # #[cfg(feature = "secret")]
/// let mut config = Config::new("test");
/// # #[cfg(not(feature = "secret"))]
/// # let mut config = Config::new();
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Foo(String);
///
/// struct PersistSourceImpl;
/// impl PersistSource for PersistSourceImpl {
///     type Value = Foo;
///     type Map = Vec<(String, Self::Value)>;
///
/// #   #[cfg(not(feature = "default_config_dir"))]
///     fn path(&self) -> std::path::PathBuf {
///         std::path::PathBuf::from("tests").join("persist.conf")
///     }
/// #
/// #    #[cfg(feature = "default_config_dir")]
/// #    fn source_name(&self) -> String {
/// #        "persist.conf".to_owned()
/// #    }
///
///     fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
///         Ok(vec![("persist".to_owned(), Foo("persist".to_owned()))])
///     }
/// }
///
/// config.add_persist_source(PersistSourceImpl).unwrap();
/// let new_value = Foo("new persist".to_owned());
/// config.upgrade("persist", &new_value).unwrap();
/// assert_eq!(config.get::<_, Foo>("persist").unwrap(), new_value);
///
/// # #[cfg(feature = "secret")]
/// let mut config_new = Config::new("test");
/// # #[cfg(not(feature = "secret"))]
/// # let mut config_new = Config::new();
/// config_new.add_persist_source(PersistSourceImpl).unwrap(); // Read config from disk
/// assert_eq!(config_new.get::<_, Foo>("persist").unwrap(), new_value);
/// ```
#[cfg(feature = "persist")]
pub trait PersistSource {
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
