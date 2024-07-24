//! # Config
//! This module provides a [`Config`] struct that can be used to cache configuration values.

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
/// This behaves like a native cache in CPU:
/// 1. If cache hit, return the cached value when reading, while update the cached value and then write back when writing.
/// 2. If cache miss, load the cached value from the source to cache when reading, while write and then load when writing.
///
#[cfg_attr(
    feature = "secret",
    doc = "To avoid entering the password during testing, you can enable `mock` feature. This can always return the **same** Encrypter during **each** test."
)]
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

impl Config {
    /// Create a new [`Config`] cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an immutable ref from the config.
    /// If the value is not found, it will be created with the default value.
    /// See [`ConfigRef`] for more details.
    pub fn get<T>(&self) -> ConfigRef<'_, T>
    where
        T: Source + Any + Send + Sync,
    {
        T::retrieve(&self.cache)
    }

    /// Get a mutable ref from the config.
    /// If the value is not found, it will be created with the default value.
    /// See [`ConfigMut`] for more details.
    pub fn get_mut<T>(&self) -> ConfigMut<'_, T>
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

    /// Get many immutable refs from the config.
    ///
    /// T: (T1, T2, T3,)
    ///
    /// If the value is not found, it will be created with the default value.
    /// See [`ConfigRef`] for more details.
    pub fn get_many<T>(&self) -> <T as Cacheable<((),)>>::Ref<'_>
    where
        T: Cacheable<((),)> + Any + Send + Sync,
    {
        T::retrieve(&self.cache)
    }

    /// Get many mutable refs from the config.
    ///
    /// T: (T1, T2, T3,)
    ///
    /// If the value is not found, it will be created with the default value.
    /// See [`ConfigMut`] for more details.
    pub fn get_mut_many<T>(&self) -> <T as Cacheable<((),)>>::Mut<'_>
    where
        T: Cacheable<((),)> + Any + Send + Sync,
    {
        T::retrieve_mut(&self.cache)
    }

    /// Take the ownerships of the config value.
    ///
    /// T: (T1, T2, T3,)
    ///
    /// If the value is not found, it will be created with the default value.
    /// This will remove the value from the config.
    pub fn take_many<T>(&self) -> <T as Cacheable<((),)>>::Owned
    where
        T: Cacheable<((),)> + Any + Send + Sync,
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

#[allow(missing_docs, private_bounds, private_interfaces)]
pub trait Cacheable<T>
where
    Self: Any,
{
    type Ref<'a>;
    type Mut<'a>;
    type Owned;
    fn retrieve(cache: &Cache) -> Self::Ref<'_>;
    fn retrieve_mut(cache: &Cache) -> Self::Mut<'_>;
    fn take(cache: &Cache) -> Self::Owned;
}

#[allow(private_bounds, private_interfaces)]
impl<T> Cacheable<()> for T
where
    T: Source + Any + Send + Sync,
{
    type Ref<'a> = ConfigRef<'a, T>;
    type Mut<'a> = ConfigMut<'a, T>;
    type Owned = T;

    fn retrieve(cache: &Cache) -> Self::Ref<'_> {
        ConfigRef {
            guard: cache.get_or_default::<T>().inner.read().unwrap(),
            _marker: PhantomData,
        }
    }

    fn retrieve_mut(cache: &Cache) -> Self::Mut<'_> {
        ConfigMut {
            guard: cache.get_or_default::<T>().inner.write().unwrap(),
            dirty: false,
            _marker: PhantomData,
        }
    }

    fn take(cache: &Cache) -> Self::Owned {
        cache.take_or_default::<T>()
    }
}

macro_rules! impl_cacheable {
    ($($t: ident),+$(,)?) => {
        #[allow(private_bounds, private_interfaces)]
        impl<$($t,)+> Cacheable<((),)> for ($($t,)+)
        where
            $($t: Cacheable<()>,)+
        {
            type Ref<'a> = ($(<$t as Cacheable<()>>::Ref<'a>,)+);
            type Mut<'a> = ($(<$t as Cacheable<()>>::Mut<'a>,)+);
            type Owned = ($(<$t as Cacheable<()>>::Owned,)+);
            fn retrieve(cache: &Cache) -> Self::Ref<'_> {
                ($(<$t as Cacheable<()>>::retrieve(cache),)+)
            }

            fn retrieve_mut(cache: &Cache) -> Self::Mut<'_> {
                ($(<$t as Cacheable<()>>::retrieve_mut(cache),)+)
            }

            fn take(cache: &Cache) -> Self::Owned {
                ($(<$t as Cacheable<()>>::take(cache),)+)
            }
        }
    };
}

impl_cacheable!(T1);
impl_cacheable!(T1, T2);
impl_cacheable!(T1, T2, T3);
impl_cacheable!(T1, T2, T3, T4);
impl_cacheable!(T1, T2, T3, T4, T5);
impl_cacheable!(T1, T2, T3, T4, T5, T6);
impl_cacheable!(T1, T2, T3, T4, T5, T6, T7);
impl_cacheable!(T1, T2, T3, T4, T5, T6, T7, T8);
