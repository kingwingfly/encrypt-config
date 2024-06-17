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
    cell::UnsafeCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

struct CacheValue {
    inner: UnsafeCell<Box<dyn Any + Send + Sync>>,
}

unsafe impl Send for CacheValue {}
unsafe impl Sync for CacheValue {}

type Cache = HashMap<TypeId, CacheValue>;
type AccessMap = HashMap<TypeId, Access>;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Access {
    Read,
    Write,
}

/// A struct that can be used to store configuration values.
/// # Example
/// See [`NormalSource`], [`PersistSource`], [`SecretSource`]
pub struct Config {
    cache: Cache,
    access: AccessMap,
}

impl Default for Config {
    /// Create an empty [`Config`] struct.
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConfigRef<'a, T: 'static> {
    inner: &'a T,
}

pub struct ConfigMut<'a, T: 'static> {
    inner: &'a mut T,
}

impl<T: 'static> Deref for ConfigRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner
    }
}

impl<T: 'static> Deref for ConfigMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner
    }
}

impl<T: 'static> DerefMut for ConfigMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner
    }
}

trait FromCache {
    type Item<'new>;

    fn accesses(access: &mut AccessMap);

    unsafe fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>>;
}

impl<T: 'static> FromCache for ConfigRef<'_, T> {
    type Item<'new> = ConfigRef<'new, T>;

    fn accesses(access: &mut AccessMap) {
        assert_eq!(
            *access.entry(TypeId::of::<T>()).or_insert(Access::Read),
            Access::Read,
            "Attempting to access {} mutably and immutably at the same time",
            type_name::<T>(),
        );
    }

    unsafe fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>> {
        let inner = (&*cache
            .get(&TypeId::of::<T>())
            .context(ConfigNotFound {
                r#type: type_name::<T>(),
            })?
            .inner
            .get())
            .downcast_ref::<T>()
            .unwrap();
        Ok(ConfigRef { inner })
    }
}

impl<T: 'static> FromCache for ConfigMut<'_, T> {
    type Item<'new> = ConfigMut<'new, T>;

    fn accesses(access: &mut AccessMap) {
        match access.insert(TypeId::of::<T>(), Access::Write) {
            Some(Access::Read) => panic!(
                "Attempting to access {} mutably and immutably at the same time",
                std::any::type_name::<T>()
            ),
            Some(Access::Write) => panic!(
                "Attempting to access {} mutably twice",
                std::any::type_name::<T>()
            ),
            None => (),
        }
    }

    unsafe fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>> {
        let inner = (&mut *cache
            .get(&TypeId::of::<T>())
            .context(ConfigNotFound {
                r#type: type_name::<T>(),
            })?
            .inner
            .get())
            .downcast_mut::<T>()
            .unwrap();
        Ok(ConfigMut { inner })
    }
}

impl Config {
    /// Create a new [`Config`] struct.
    #[cfg_attr(
        feature = "persist",
        doc = "To avoid manually delete the config file persisted during testing, you can enable `mock` feature. This will impl `Drop` for `Config` which automatically delete the config file persisted."
    )]
    ///
    #[cfg_attr(
        feature = "secret",
        doc = "To avoid entering the password during testing, you can enable `mock` feature. This can always return `Config`s with the **same** Encrypter during **each** test."
    )]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            access: HashMap::new(),
        }
    }

    /// Get an immutable ref from the config.
    pub fn get<T>(&mut self) -> ConfigResult<ConfigRef<T>>
    where
        T: Any + 'static,
    {
        ConfigRef::<'_, T>::accesses(&mut self.access);
        unsafe { ConfigRef::retrieve(&self.cache) }
    }

    /// Get a mutable ref from the config.
    pub fn get_mut<T>(&mut self) -> ConfigResult<ConfigMut<T>>
    where
        T: Any + 'static,
    {
        ConfigMut::<'_, T>::accesses(&mut self.access);
        unsafe { ConfigMut::retrieve(&self.cache) }
    }

    /// Release a ref in config
    pub fn release<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + 'static,
    {
        self.access.remove(&TypeId::of::<T>());
        Ok(())
    }

    /// Add a normal source to the config.
    /// The source must implement [`NormalSource`] trait, which is for normal config that does not need to be encrypted or persisted.
    pub fn add_normal_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + NormalSource + Send + Sync + 'static,
    {
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue {
                inner: UnsafeCell::new(Box::new(T::default())),
            },
        );
        Ok(())
    }

    /// Add a persist source to the config.
    /// The source must implement [`PersistSource`] trait, which is for config that needs to be persisted.
    #[cfg(feature = "persist")]
    pub fn add_persist_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + PersistSource + Send + Sync + 'static,
        for<'de> T: Deserialize<'de>,
    {
        let value: T = T::load().unwrap_or_default();
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue {
                inner: UnsafeCell::new(Box::new(value)),
            },
        );
        Ok(())
    }

    /// Add a secret source to the config.
    /// The source must implement [`SecretSource`] trait, which is for config that needs to be encrypted and persisted.
    #[cfg(feature = "secret")]
    pub fn add_secret_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + SecretSource + Send + Sync + 'static,
        for<'de> T: Deserialize<'de>,
    {
        let value: T = T::load().unwrap_or_default();
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue {
                inner: UnsafeCell::new(Box::new(value)),
            },
        );
        Ok(())
    }
}
