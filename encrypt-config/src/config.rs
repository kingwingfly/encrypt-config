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
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

enum CacheValue {
    Normal {
        inner: RefCell<Box<dyn Any>>,
    },
    #[cfg(feature = "persist")]
    Persist {
        inner: RefCell<Box<dyn Any>>,
    },
    #[cfg(feature = "secret")]
    Secret {
        inner: RefCell<Box<dyn Any>>,
    },
}

type Cache = HashMap<TypeId, CacheValue>;

/// A struct that can be used to store configuration values.
/// # Example
/// See [`NormalSource`], [`PersistSource`], [`SecretSource`]
pub struct Config {
    cache: Cache,
}

impl Default for Config {
    /// Create an empty [`Config`] struct.
    #[cfg_attr(
        feature = "secret",
        doc = "The default keyring entry is `encrypt_config`"
    )]
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConfigRef<'a, T: 'static> {
    inner: Ref<'a, Box<dyn Any>>,
    _marker: PhantomData<&'a T>,
}

pub struct ConfigMut<'a, T: 'static> {
    inner: RefMut<'a, Box<dyn Any>>,
    _marker: PhantomData<&'a mut T>,
}

impl<T: 'static> Deref for ConfigRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.downcast_ref().unwrap()
    }
}

impl<T: 'static> Deref for ConfigMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.downcast_ref().unwrap()
    }
}

impl<T: 'static> DerefMut for ConfigMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.downcast_mut().unwrap()
    }
}

trait FromCache {
    type Item<'new>;

    fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>>;
}

impl<T: 'static> FromCache for ConfigRef<'_, T> {
    type Item<'new> = ConfigRef<'new, T>;

    fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>> {
        let inner = match cache.get(&TypeId::of::<T>()).context(ConfigNotFound {
            r#type: type_name::<T>(),
        })? {
            CacheValue::Normal { inner } => inner.borrow(),
            #[cfg(feature = "persist")]
            CacheValue::Persist { inner, .. } => inner.borrow(),
            #[cfg(feature = "secret")]
            CacheValue::Secret { inner, .. } => inner.borrow(),
        };
        Ok(ConfigRef {
            inner,
            _marker: PhantomData,
        })
    }
}

impl<T: 'static> FromCache for ConfigMut<'_, T> {
    type Item<'new> = ConfigMut<'new, T>;

    fn retrieve(cache: &Cache) -> ConfigResult<Self::Item<'_>> {
        let inner = match cache.get(&TypeId::of::<T>()).context(ConfigNotFound {
            r#type: type_name::<T>(),
        })? {
            CacheValue::Normal { inner } => inner.borrow_mut(),
            #[cfg(feature = "persist")]
            CacheValue::Persist { inner, .. } => inner.borrow_mut(),
            #[cfg(feature = "secret")]
            CacheValue::Secret { inner, .. } => inner.borrow_mut(),
        };
        Ok(ConfigMut {
            inner,
            _marker: PhantomData,
        })
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
            cache: Cache::new(),
        }
    }

    /// Get an immutable ref from the config.
    pub fn get<T>(&self) -> ConfigResult<ConfigRef<T>>
    where
        T: Any + 'static,
    {
        ConfigRef::retrieve(&self.cache)
    }

    /// Get a mutable ref from the config.
    pub fn get_mut<T>(&self) -> ConfigResult<ConfigMut<T>>
    where
        T: Any + 'static,
    {
        ConfigMut::retrieve(&self.cache)
    }

    /// Add a normal source to the config.
    /// The source must implement [`NormalSource`] trait, which is for normal config that does not need to be encrypted or persisted.
    pub fn add_normal_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + NormalSource + 'static,
    {
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue::Normal {
                inner: RefCell::new(Box::new(T::default()) as Box<dyn Any>),
            },
        );
        Ok(())
    }

    /// Add a persist source to the config.
    /// The source must implement [`PersistSource`] trait, which is for config that needs to be persisted.
    #[cfg(feature = "persist")]
    pub fn add_persist_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + PersistSource + 'static,
        for<'de> T: Deserialize<'de>,
    {
        let value: T = T::load().unwrap_or_default();
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue::Persist {
                inner: RefCell::new(Box::new(value)),
            },
        );
        Ok(())
    }

    /// Add a secret source to the config.
    /// The source must implement [`SecretSource`] trait, which is for config that needs to be encrypted and persisted.
    #[cfg(feature = "secret")]
    pub fn add_secret_source<T>(&mut self) -> ConfigResult<()>
    where
        T: Any + SecretSource + 'static,
        for<'de> T: Deserialize<'de>,
    {
        let value: T = T::load().unwrap_or_default();
        self.cache.insert(
            TypeId::of::<T>(),
            CacheValue::Secret {
                inner: RefCell::new(Box::new(value)),
            },
        );
        Ok(())
    }
}
