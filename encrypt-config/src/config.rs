//! # Config
//! This module provides a [`Config`] struct that can be used to store configuration values.

use crate::Source;
use std::{
    any::{Any, TypeId},
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
    inner: HashMap<TypeId, CacheValue>,
}

impl Cache {
    fn get_or_default<T: Source + Any + Send + Sync>(&self) -> &mut CacheValue {
        self.inner.entry(TypeId::of::<T>()).or_insert(CacheValue {
            inner: RwLock::new(Box::new(T::load_or_default())),
        })
    }

    fn take_or_default<T: Source + Any + Send + Sync>(&self) -> T {
        match self.inner.remove(&TypeId::of::<T>()) {
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
/// The value will be saved as dropping if changed, if saving fails, it will print the error.
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
            }
            self.dirty = false;
        }
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
        Self::default()
    }

    /// Get an immutable ref from the config.
    /// See [`ConfigRef`] for more details.
    pub fn get<T>(&self) -> ConfigRef<T>
    where
        T: Source + Any + Send + Sync,
    {
        T::retrieve(&self.cache)
    }

    /// Get a mutable ref from the config.
    /// See [`ConfigMut`] for more details.
    pub fn get_mut<T>(&self) -> ConfigMut<T>
    where
        T: Source + Any + Send + Sync,
    {
        T::retrieve_mut(&self.cache)
    }

    /// Take the ownership of the config value.
    /// This will remove the value from the config.
    pub fn take<T>(&self) -> T
    where
        T: Source + Any + Send + Sync,
    {
        T::take(&self.cache)
    }

    /// Save the config value manually.
    /// Note that the changes you made through [`ConfigMut`]
    /// will be saved as it leaves the scope.
    pub fn save<T>(&self) {
        todo!()
    }
}

macro_rules! impl_cacheable {
    ($($t: ident),+) => {
        impl<$($t, )+> Cacheable for ($($t,)+)
        where
            $($t: Source + Any + Send + Sync,)+
        {
            #[allow(non_snake_case)]
            fn retrieve(cache: &Cache) -> ($(ConfigRef<'_, $t>,)+) {
                $(let $t = $t::retrieve(cache);)+
                ($($t,)+)
            }

            #[allow(non_snake_case)]
            fn retrieve_mut(cache: &Cache) -> ($(ConfigMut<'_, $t>,)+) {
                $(let $t = $t::retrieve_mut(cache);)+
                ($($t,)+)
            }

            #[allow(non_snake_case)]
            fn take(cache: &Cache) -> ($($t,)+) {
                $(let $t = $t::take(cache);)+
                ($($t,)+)
            }
        }
    };
}

// impl_cacheable!(T1);
impl_cacheable!(T1, T2);
// impl_cacheable!(T1, T2, T3);
// impl_cacheable!(T1, T2, T3, T4);
// impl_cacheable!(T1, T2, T3, T4, T5);
// impl_cacheable!(T1, T2, T3, T4, T5, T6);
// impl_cacheable!(T1, T2, T3, T4, T5, T6, T7);
// impl_cacheable!(T1, T2, T3, T4, T5, T6, T7, T8);
