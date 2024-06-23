//! # Config
//! This module provides a [`Config`] struct that can be used to store configuration values.

use crate::{
    error::{ConfigNotFound, ConfigResult},
    Source,
};
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

/// This holds a `RwLockReadGuard` of the config value until the end of the scope.
/// It is used to get an immutable reference to the config value.
/// One should drop it as soon as possible to avoid deadlocks.
/// # Deadlocks
/// - If you already held a [`ConfigMut`], [`Config::get()`] will block until you drop it.
pub struct ConfigRef<'a, T: 'static> {
    guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<&'a T>,
}

/// This holds a `RwLockWriteGuard` of the config value until the end of the scope.
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

    /// Load sources to the `Config`.
    /// `T: Source` or tuple of `T: Source` like `(T1, T2, T3)`.
    /// Please make sure each kind of source is loaded only once,
    /// or the later one will overwrite the previous one.
    /// If load fails (not existing or failed to decrypt),
    /// it will use the default value.
    #[allow(private_bounds)]
    pub fn load_source<T>(&mut self)
    where
        T: SaveLoad,
    {
        T::load_to(&mut self.cache);
    }

    /// Save sources in the `Config`.
    /// `T: Source` or tuple of `T: Source` like `(T1, T2, T3)`.
    /// # Deadlocks
    /// This method will try to get [`ConfigRef`] from the cache, so if [`ConfigMut`]
    /// exists, it will block until it is dropped.
    /// # Errors
    /// - If one of the sources is not found in the cache, it will return an error,
    /// and none of the sources will be saved.
    /// - If one of the sources fails to save (io error), it will return an error,
    /// sources that have been saved will not be rolled back, the rest will not be saved,
    /// and the file which stores the failed source may lose its content (just making sure
    /// you always have the right permission to operate the target file is okay).
    #[allow(private_bounds)]
    pub fn save<T>(&self) -> ConfigResult<()>
    where
        T: SaveLoad,
    {
        T::save(&self.cache)
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

    /// Take the ownership of the config value.
    /// This will remove the value from the config.
    pub fn take<T>(&mut self) -> ConfigResult<T>
    where
        T: Any + 'static,
    {
        let value = self
            .cache
            .remove(&TypeId::of::<T>())
            .context(ConfigNotFound {
                r#type: type_name::<T>(),
            })?;
        let value = value
            .inner
            .into_inner()
            .expect("EncryptConfig: Rwlock is poisoned");
        Ok(*value.downcast().unwrap())
    }
}

/// This trait is used to save and load the configuration values.
trait SaveLoad {
    fn save(cache: &Cache) -> ConfigResult<()>;
    fn load_to(cache: &mut Cache);
}

macro_rules! impl_savaload {
    ($($t: ident),+) => {
        impl<$($t, )+> SaveLoad for ($($t, )+)
        where
            $($t: Any + Source + Send + Sync + 'static,)+
        {
            #[allow(non_snake_case)]
            fn save(cache: &Cache) -> ConfigResult<()> {
                $(let $t = ConfigRef::<$t>::retrieve(cache)?;)+
                $($t.save()?;)+
                Ok(())
            }

            #[allow(non_snake_case)]
            fn load_to(cache: &mut Cache) {
                $(let $t = $t::load_or_default();)+
                $(cache.insert(
                    TypeId::of::<$t>(),
                    CacheValue {
                        inner: RwLock::new(Box::new($t)),
                    },
                );)+
            }
        }
    };
}

impl<T> SaveLoad for T
where
    T: Any + Source + Send + Sync + 'static,
{
    fn save(cache: &Cache) -> ConfigResult<()> {
        let t = ConfigRef::<T>::retrieve(cache)?;
        t.save()
    }

    fn load_to(cache: &mut Cache) {
        let t = T::load_or_default();
        cache.insert(
            TypeId::of::<T>(),
            CacheValue {
                inner: RwLock::new(Box::new(t)),
            },
        );
    }
}

impl_savaload!(T1);
impl_savaload!(T1, T2);
impl_savaload!(T1, T2, T3);
impl_savaload!(T1, T2, T3, T4);
impl_savaload!(T1, T2, T3, T4, T5);
impl_savaload!(T1, T2, T3, T4, T5, T6);
impl_savaload!(T1, T2, T3, T4, T5, T6, T7);
impl_savaload!(T1, T2, T3, T4, T5, T6, T7, T8);
