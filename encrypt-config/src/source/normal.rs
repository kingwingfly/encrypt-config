use serde::{de::DeserializeOwned, Serialize};

/// A trait for normal config source that is neither encrypted or persisted.
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
