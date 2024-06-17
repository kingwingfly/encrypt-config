//! # Config
//! This module provides a [`Config`] struct that can be used to store configuration values.

#[cfg(feature = "persist")]
use crate::PersistSource;
#[cfg(feature = "secret")]
use crate::SecretSource;
use crate::{
    error::{ConfigNotFound, ConfigResult},
    NormalSource,
};
#[cfg(feature = "persist")]
use serde::Deserialize;
use snafu::OptionExt as _;
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

struct CacheValue {
    inner: RwLock<Box<dyn Any + Send + Sync>>,
}

type Cache = HashMap<TypeId, CacheValue>;

/// A struct that can be used to store configuration values.
pub struct Config {
    cache: Cache,
}

impl Default for Config {
    /// Create an empty [`Config`] struct.
    fn default() -> Self {
        Self::new()
    }
}

/// This holds a `RwLockReadGuard` of the config value util the end of the scope.
/// It is used to get an immutable reference to the config value.
/// One should drop it as soon as possible to avoid deadlocks.
/// # Deadlocks
/// - If you already held a [`ConfigMut`], [`Config::get()`] will block until you drop it.
pub struct ConfigRef<'a, T: 'static> {
    guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<&'a T>,
}

/// This holds a `RwLockWriteGuard` of the config value util the end of the scope.
/// It is used to get a mutable reference to the config value.
/// One should drop it as soon as possible to avoid deadlocks.
/// # Deadlocks
/// - If you already held a [`ConfigRef`] or [`ConfigMut`], [`Config::get_mut()`] will block until you drop it.
pub struct ConfigMut<'a, T: 'static> {
    guard: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<&'a mut T>,
}

impl<T: 'static> Deref for ConfigRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref().unwrap()
    }
}

impl<T: 'static> Deref for ConfigMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref().unwrap()
    }
}

impl<T: 'static> DerefMut for ConfigMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.downcast_mut().unwrap()
    }
}

trait FromCache {
    type Item<'new>;

    fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>>;
}

impl<T: 'static> FromCache for ConfigRef<'_, T> {
    type Item<'new> = ConfigRef<'new, T>;

    fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>> {
        let guard = cache
            .get(&TypeId::of::<T>())
            .context(ConfigNotFound {
                r#type: type_name::<T>(),
            })?
            .inner
            .read()
            .unwrap();
        Ok(ConfigRef {
            guard,
            _marker: PhantomData,
        })
    }
}

impl<T: 'static> FromCache for ConfigMut<'_, T> {
    type Item<'new> = ConfigMut<'new, T>;

    fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>> {
        let guard = cache
            .get(&TypeId::of::<T>())
            .context(ConfigNotFound {
                r#type: type_name::<T>(),
            })?
            .inner
            .write()
            .unwrap();
        Ok(ConfigMut {
            guard,
            _marker: PhantomData,
        })
    }
}

impl Config {
    /// Create a new [`Config`] struct.
    ///
    #[cfg_attr(
        feature = "secret",
        doc = "To avoid entering the password during testing, you can enable `mock` feature. This can always return the **same** Encrypter during **each** test."
    )]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Get an immutable ref from the config.
    /// See [`ConfigRef`] for more details.
    pub fn get<T>(&self) -> ConfigResult<ConfigRef<T>>
    where
        T: Any + 'static,
    {
        ConfigRef::retrieve(&self.cache)
    }

    /// Get a mutable ref from the config.
    /// See [`ConfigMut`] for more details.
    pub fn get_mut<T>(&self) -> ConfigResult<ConfigMut<T>>
    where
        T: Any + 'static,
    {
        ConfigMut::retrieve(&self.cache)
    }

    /// Add a normal source to the config.
    /// The source must implement [`NormalSource`] trait, which is for normal config that does not need to be encrypted or persisted.
    ///
    /// The `Default` trait is required as the default value of the config.
    pub fn add_normal_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + NormalSource + Default + Send + Sync + 'static,
    {
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue {
                inner: RwLock::new(Box::new(T::default())),
            },
        );
        Ok(())
    }

    /// Add a persist source to the config.
    /// The source must implement [`PersistSource`] trait, which is for config that needs to be persisted.
    ///
    /// The `Default` trait is required as the default value of the config.
    #[cfg(feature = "persist")]
    pub fn add_persist_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + PersistSource + Default + Send + Sync + 'static,
        for<'de> T: Deserialize<'de>,
    {
        let value: T = T::load().unwrap_or_default();
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue {
                inner: RwLock::new(Box::new(value)),
            },
        );
        Ok(())
    }

    /// Add a secret source to the config.
    /// The source must implement [`SecretSource`] trait, which is for config that needs to be encrypted and persisted.
    ///
    /// The `Default` trait is required as the default value of the config.
    #[cfg(feature = "secret")]
    pub fn add_secret_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + SecretSource + Default + Send + Sync + 'static,
        for<'de> T: Deserialize<'de>,
    {
        let value: T = T::load().unwrap_or_default();
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue {
                inner: RwLock::new(Box::new(value)),
            },
        );
        Ok(())
    }
}
