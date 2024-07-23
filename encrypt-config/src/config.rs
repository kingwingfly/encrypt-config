//! # Config
//! This module provides a [`Config`] struct that can be used to store configuration values.

use crate::{error::ConfigResult, Source};
use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

struct CacheValue {
    inner: RwLock<Box<dyn Any + Send + Sync>>,
}

#[derive(Default)]
struct Cache {
    inner: UnsafeCell<HashMap<TypeId, CacheValue>>,
}
unsafe impl Sync for Cache {}

impl Cache {
    fn get_or_default<T: Source + Any + Send + Sync>(&self) -> &CacheValue {
        // SAFETY: The inner value is behind a RwLock.
        let map = unsafe { &mut *self.inner.get() };
        map.entry(TypeId::of::<T>()).or_insert(CacheValue {
            inner: RwLock::new(Box::new(T::load_or_default())),
        })
    }

    fn take_or_default<T: Source + Any + Send + Sync>(&self) -> T {
        // SAFETY: UnsafeCell is used to allow interior mutability.
        let map = unsafe { &mut *self.inner.get() };

        match map.remove(&TypeId::of::<T>()) {
            Some(value) => *value
                .inner
                .into_inner()
                .expect("EncryptConfig: Rwlock is poisoned")
                .downcast()
                .unwrap(),
            None => T::load_or_default(),
        }
    }
}

/// A struct that can be used to **cache** configuration values.
pub struct Config {
    cache: Cache,
}

impl Default for Config {
    /// Create an empty [`Config`] struct.
    fn default() -> Self {
        Self {
            cache: Cache::default(),
        }
    }
}

/// This holds a `RwLockReadGuard` of the config value until the end of the scope.
/// It is used to get an immutable reference to the config value.
/// One should drop it as soon as possible to avoid deadlocks.
/// # Deadlocks
/// - If you already held a [`ConfigMut`], [`Config::get()`] will block until you drop it.
pub struct ConfigRef<'a, T>
where
    T: Source + Any,
{
    guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<&'a T>,
}

/// This holds a `RwLockWriteGuard` of the config value until the end of the scope.
/// It is used to get a mutable reference to the config value.
/// One should drop it as soon as possible to avoid deadlocks.
///
/// The value will be written back as dropping if changed. If saving fails, it will print an error.
/// # Deadlocks
/// - If you already held a [`ConfigRef`] or [`ConfigMut`], [`Config::get_mut()`] will block until you drop it.
pub struct ConfigMut<'a, T>
where
    T: Source + Any,
{
    guard: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    dirty: bool,
    _marker: PhantomData<&'a mut T>,
}

impl<T> Deref for ConfigRef<'_, T>
where
    T: Source + Any,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref().unwrap()
    }
}

impl<T> Deref for ConfigMut<'_, T>
where
    T: Source + Any,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref().unwrap()
    }
}

impl<T> DerefMut for ConfigMut<'_, T>
where
    T: Source + Any,
{
    fn deref_mut(&mut self) -> &mut T {
        self.dirty = true;
        self.guard.downcast_mut().unwrap()
    }
}

impl<T> Drop for ConfigMut<'_, T>
where
    T: Source + Any,
{
    fn drop(&mut self) {
        if self.dirty {
            if let Err(e) = self.save() {
                println!(
                    "Error from encrypt_config: {e}\nThis msg is printed while saving as dropping."
                );
                return;
            }
            self.dirty = false;
        }
    }
}

trait Cacheable
where
    Self: Source + Any + Sized,
{
    fn retrieve(cache: &Cache) -> ConfigRef<'_, Self>;
    fn retrieve_mut(cache: &Cache) -> ConfigMut<'_, Self>;
    fn take(cache: &Cache) -> Self;
}

impl<T> Cacheable for T
where
    T: Source + Any + Send + Sync,
{
    fn retrieve(cache: &Cache) -> ConfigRef<'_, T> {
        ConfigRef {
            guard: cache.get_or_default::<T>().inner.read().unwrap(),
            _marker: PhantomData,
        }
    }

    fn retrieve_mut(cache: &Cache) -> ConfigMut<'_, T> {
        ConfigMut {
            guard: cache.get_or_default::<T>().inner.write().unwrap(),
            dirty: false,
            _marker: PhantomData,
        }
    }

    fn take(cache: &Cache) -> T {
        cache.take_or_default::<T>()
    }
}

impl Config {
    /// Create a new [`Config`] cache.
    /// This behaves like a native cache in CPU:
    /// 1. If cache hit, return the cached value when reading, while update the cached value and then write back when writing.
    /// 2. If cache miss, load the cached value from the source to cache when reading, while write and then load when writing.
    ///
    #[cfg_attr(
        feature = "secret",
        doc = "To avoid entering the password during testing, you can enable `mock` feature. This can always return the **same** Encrypter during **each** test."
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an immutable ref from the config.
    /// If the value is not found, it will be created with the default value.
    /// See [`ConfigRef`] for more details.
    pub fn get<T>(&self) -> ConfigRef<T>
    where
        T: Source + Any + Send + Sync,
    {
        T::retrieve(&self.cache)
    }

    /// Get a mutable ref from the config.
    /// If the value is not found, it will be created with the default value.
    /// See [`ConfigMut`] for more details.
    pub fn get_mut<T>(&self) -> ConfigMut<T>
    where
        T: Source + Any + Send + Sync,
    {
        T::retrieve_mut(&self.cache)
    }

    /// Take the ownership of the config value.
    /// If the value is not found, it will be created with the default value.
    /// This will remove the value from the config.
    pub fn take<T>(&self) -> T
    where
        T: Source + Any + Send + Sync,
    {
        T::take(&self.cache)
    }

    /// Save the config value manually.
    /// Note that the changes you made through [`ConfigMut`]
    /// will be saved as leaving the scope automatically.
    ///
    /// Ideally, it's better to change cache first and then set dirty flag when writing,
    /// and save the value when the cache drops. However, this is hard to implement
    /// for manual saving by now. So, one is supposed to use `get` and `get_mut` to change the value.
    pub fn save<T>(&self, value: T) -> ConfigResult<()>
    where
        T: Source + Any + Send + Sync,
    {
        // TODO:
        // If cache hit, write cache and set dirty, write back when dropping Cache
        // instead of writing back here.
        value.save()?;
        // SAFETY:
        match unsafe { (*self.cache.inner.get()).get_mut(&TypeId::of::<T>()) } {
            Some(cache) => {
                *cache.inner.write().unwrap() = Box::new(value);
            }
            None => {
                T::retrieve(&self.cache);
            }
        }
        Ok(())
    }
}
