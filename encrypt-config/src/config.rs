//! # Config
//! This module provides a [`Config`] struct that can be used to cache configuration values.

use rom_cache::cache::{CacheMut, CacheRef};
use std::any::Any;

type Cache = rom_cache::Cache<1, 8>;

/// A struct that can be used to **cache** configuration values.
/// This behaves like a native cache in CPU:
/// 1. If cache hit, reading returns the cached value, while writing upgrades the cached value then set cache flag dirty.
/// 2. If cache miss, reading loads the value from the source to cache, while writing saves the value to source then loads it to cache.
/// 3. All caches values dirty will be written back when Config dropped.
///
/// You are safe to get **at most 8** different types' refs and mut_refs at the same time in all threads,
/// due to the default cache capacity.
/// Or panic occurs with errors like `Busy`.
#[cfg_attr(
    feature = "secret",
    doc = "To avoid entering the password during testing, you can enable `mock` feature. This can always return the **same** Encrypter during **each** test."
)]
pub struct Config {
    cache: Cache,
}

impl Default for Config {
    /// Create an empty [`Config`] cache.
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

    /// Get an immutable ref ([`CfgRef`]) from the config.
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
    ///
    /// Caution: You can only get up to 63 immutable refs ([`CfgRef`]) of each type at the same time.
    ///
    /// If the value was marked as writing, it would panic like `RefCell`.
    /// See [`CfgRef`] for more details.
    pub fn get<T>(&self) -> <T as Cacheable<()>>::Ref<'_>
    where
        T: Cacheable<()> + Any + Send + Sync,
    {
        T::retrieve(&self.cache)
    }

    /// Get a mutable ref ([`CfgMut`]) from the config.
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
    ///
    /// Caution: You can only get up to 1 mutable ref ([`CfgMut`]) of each type at the same time.
    ///
    /// If the value was marked as writing, it would panic like `RefCell`.
    /// See [`CfgMut`] for more details.
    pub fn get_mut<T>(&self) -> <T as Cacheable<()>>::Mut<'_>
    where
        T: Cacheable<()> + Any + Send + Sync,
    {
        T::retrieve_mut(&self.cache)
    }

    /// Get many immutable refs from the config.
    ///
    /// T: (T1, T2, T3,)
    ///
    /// Caution: You can only get up to 32 (1 << 5) immutable refs ([`CfgRef`]) at the same time.
    ///
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
    /// See [`CfgRef`] for more details.
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
    /// Caution: You can only get up to 1 mutable ref ([`CfgMut`]) at the same time.
    ///
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
    /// See [`CfgMut`] for more details.
    pub fn get_mut_many<T>(&self) -> <T as Cacheable<((),)>>::Mut<'_>
    where
        T: Cacheable<((),)> + Any + Send + Sync,
    {
        T::retrieve_mut(&self.cache)
    }
}

/// # Panic
/// - If you already held a [`CfgMut`], [`Config::get()`] will panic.
pub type CfgRef<'a, T> = CacheRef<'a, T>;

/// # Panic
/// - If you already held a [`CfgMut`], [`Config::get()`] will panic.
pub type CfgMut<'a, T> = CacheMut<'a, T>;

/// This trait is used to retrieve the config value from the cache.
#[allow(private_bounds, private_interfaces)]
pub trait Cacheable<T> {
    /// Immutable reference retrieved from the cache.
    type Ref<'a>;
    /// Mutable reference retrieved from the cache.
    type Mut<'a>;
    /// Retrieve the immutable ref from the cache.
    fn retrieve(cache: &Cache) -> Self::Ref<'_>;
    /// Retrieve the mutable ref from the cache.
    fn retrieve_mut(cache: &Cache) -> Self::Mut<'_>;
}

#[allow(private_bounds, private_interfaces)]
impl<T> Cacheable<()> for T
where
    T: rom_cache::Cacheable + Default,
{
    type Ref<'a> = CfgRef<'a, T>;
    type Mut<'a> = CfgMut<'a, T>;

    fn retrieve(cache: &Cache) -> Self::Ref<'_> {
        cache.get::<T>().unwrap()
    }

    fn retrieve_mut(cache: &Cache) -> Self::Mut<'_> {
        cache.get_mut::<T>().unwrap()
    }
}

macro_rules! impl_cacheable {
    ($($t: ident),+$(,)?) => {
        #[allow(private_bounds, private_interfaces)]
        impl<$($t),+> Cacheable<((),)> for ($($t),+,)
        where
            $($t: Cacheable<()>,)+
        {
            type Ref<'a> = ($(<$t as Cacheable<()>>::Ref<'a>),+,);
            type Mut<'a> = ($(<$t as Cacheable<()>>::Mut<'a>),+,);
            fn retrieve(cache: &Cache) -> Self::Ref<'_> {
                ($(<$t as Cacheable<()>>::retrieve(cache)),+,)
            }

            fn retrieve_mut(cache: &Cache) -> Self::Mut<'_> {
                ($(<$t as Cacheable<()>>::retrieve_mut(cache)),+,)
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
