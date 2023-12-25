use serde::{de::DeserializeOwned, Serialize};

/// A trait for normal config source that is neither encrypted or persisted.
/// # Example
/// ```no_run
/// use encrypt_config::{Config, Source};
///
/// #[cfg(feature = "secret")]
/// let mut config = Config::new("test");
/// #[cfg(not(feature = "secret"))]
/// let mut config = Config::new();
///
/// struct NormalSource;
/// impl Source for NormalSource {
///     type Value = String;
///     type Map = Vec<(String, Self::Value)>;
///
///     fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
///         Ok(vec![("key".to_owned(), "value".to_owned())])
///     }
/// }
///
/// config.add_source(NormalSource).unwrap();
/// let v: String = config.get("key").unwrap();
/// assert_eq!(v, "value");
/// ```
pub trait Source {
    /// The type of the config value
    type Value: Serialize + DeserializeOwned;
    /// The type of the config map. It must be iterable, the first item of the tuple is the key, which should be `String` only.
    type Map: IntoIterator<Item = (String, Self::Value)>;

    /// The default config values from this source.
    /// This is the only way to add new config key-value pairs,
    /// because we cannot infer the source type(`normal`, `persist` and `secret`) of a new key after source merged into config if not so.
    fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>>;
}
