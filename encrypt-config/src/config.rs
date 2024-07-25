//! # Config
//! This module provides a [`Config`] struct that can be used to cache configuration values.

use crate::{error::ConfigResult, source::Source};
use enumflags2::bitflags;
#[cfg(loom)]
use loom::sync::atomic::AtomicU8;
#[cfg(not(loom))]
use std::sync::atomic::AtomicU8;
use std::{
    any::type_name,
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum CacheFlags {
    Valid = 1,
    Dirty = 1 << 1,
    Writing = 1 << 2,
}

struct CacheValue {
    inner: UnsafeCell<Box<dyn Any + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    write_back_fn: Box<dyn Fn(&Box<dyn Any + Send + Sync>)>,
    flags: AtomicU8,
}

impl CacheValue {
    fn write_back(&mut self) {
        let inner = unsafe { &*self.inner.get() };
        (self.write_back_fn)(inner);
        self.flags
            .fetch_and((!CacheFlags::Dirty).bits(), Ordering::Release); // set_dirty after can not be reordered
    }
}

impl Drop for CacheValue {
    fn drop(&mut self) {
        let mask = (CacheFlags::Dirty | CacheFlags::Valid).bits();
        let flag = self.flags.load(Ordering::Acquire);
        if flag & mask == mask {
            self.write_back();
        }
    }
}

#[derive(Default)]
struct Cache {
    inner: UnsafeCell<HashMap<TypeId, CacheValue>>,
}
unsafe impl Sync for Cache {}
unsafe impl Send for Cache {}

impl Cache {
    fn get_or_default<T: Source + Any + Send + Sync>(&self) -> &CacheValue {
        // SAFETY:
        match unsafe { &*self.inner.get() }.get(&TypeId::of::<T>()) {
            Some(value) if value.flags.load(Ordering::Acquire) & CacheFlags::Valid as u8 != 0 => {
                if value.flags.load(Ordering::Acquire) & CacheFlags::Writing as u8 != 0 {
                    panic!("Cannot get a value <{}> while writing.", type_name::<T>());
                }
                value
            }
            Some(value) => {
                unsafe {
                    value.inner.get().write(Box::new(T::load_or_default()));
                }
                value
                    .flags
                    .fetch_or(CacheFlags::Valid as u8, Ordering::Release);
                value
            }
            // SAFETY: This is safe since cache is missing
            None => unsafe { &mut *self.inner.get() }
                .entry(TypeId::of::<T>())
                .or_insert(CacheValue {
                    inner: UnsafeCell::new(Box::new(T::load_or_default())),
                    write_back_fn: Box::new(|this: &Box<dyn Any + Send + Sync>| {
                        this.downcast_ref::<T>().unwrap().save().ok();
                    }),
                    flags: AtomicU8::new(CacheFlags::Valid as u8),
                }),
        }
    }

    fn take_or_default<T: Source + Any + Send + Sync>(&self) -> T {
        // SAFETY:
        let map = unsafe { &mut *self.inner.get() };
        match map.remove(&TypeId::of::<T>()) {
            Some(value) if value.flags.load(Ordering::Acquire) & CacheFlags::Valid as u8 != 0 => {
                if value.flags.load(Ordering::Acquire) & CacheFlags::Writing as u8 != 0 {
                    panic!("Cannot take a value <{}> while writing.", type_name::<T>());
                }
                unsafe {
                    std::mem::transmute_copy((*value.inner.get()).downcast_ref::<T>().unwrap())
                }
            }
            _ => T::load_or_default(),
        }
    }
}

/// A struct that can be used to **cache** configuration values.
/// This behaves like a native cache in CPU:
/// 1. If cache hit, reading returns the cached value, while writing upgrades the cached value then set cache flag dirty.
/// 2. If cache miss, reading loads the value from the source to cache, while writing saves the value to source then loads it to cache.
/// 3. All caches values dirty will be written back when Config dropped.
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

    /// Get an immutable ref from the config.
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
    ///
    /// If the value was marked as writing, it would panic like `RefCell`.
    /// See [`ConfigRef`] for more details.
    pub fn get<T>(&self) -> ConfigRef<'_, T>
    where
        T: Source + Any + Send + Sync,
    {
        T::retrieve(&self.cache)
    }

    /// Get a mutable ref from the config.
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
    ///
    /// If the value was marked as writing, it would panic like `RefCell`.
    /// See [`ConfigMut`] for more details.
    pub fn get_mut<T>(&self) -> ConfigMut<'_, T>
    where
        T: Source + Any + Send + Sync,
    {
        T::retrieve_mut(&self.cache)
    }

    /// Take the ownership of the config value.
    ///
    /// If the value was not valid, it would try creating from source, and fell back to the default value.
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
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
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
    /// If the value was not valid, it would try loading from source, and fell back to the default value.
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
    /// If the value was not valid, it would try creating from source, and fell back to the default value.
    /// This will remove the value from the config.
    pub fn take_many<T>(&self) -> <T as Cacheable<((),)>>::Owned
    where
        T: Cacheable<((),)> + Any + Send + Sync,
    {
        T::take(&self.cache)
    }

    /// Save the config value manually.
    ///
    /// Ideally, it's better to change cache first and then set dirty flag when writing,
    /// and save the value when the cache drops. However, this method is provided for manual control.
    ///
    /// Caution that this will not update the cache value.
    pub fn save<T>(&self, value: T) -> ConfigResult<()>
    where
        T: Source + Any + Send + Sync,
    {
        // SAFETY:
        match unsafe { (*self.cache.inner.get()).get(&TypeId::of::<T>()) } {
            Some(cache) => {
                unsafe {
                    cache.inner.get().write(Box::new(value));
                }
                cache
                    .flags
                    .fetch_or(CacheFlags::Dirty as u8, Ordering::Release);
            }
            None => {
                value.save()?;
                T::retrieve(&self.cache);
            }
        }
        Ok(())
    }
}

/// # Panic
/// - If you already held a [`ConfigMut`], [`Config::get()`] will panic.
pub struct ConfigRef<'a, T>
where
    T: Source + Any,
{
    inner: &'a Box<dyn Any + Send + Sync>,
    _marker: PhantomData<&'a T>,
}

/// # Panic
/// - If you already held a [`ConfigRef`] or [`ConfigMut`], [`Config::get_mut()`] will panic.
pub struct ConfigMut<'a, T>
where
    T: Source + Any,
{
    inner: &'a mut Box<dyn Any + Send + Sync>,
    flags: &'a AtomicU8,
    _marker: PhantomData<&'a mut T>,
}

impl<T> Deref for ConfigRef<'_, T>
where
    T: Source + Any,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.downcast_ref().unwrap()
    }
}

impl<T> Deref for ConfigMut<'_, T>
where
    T: Source + Any,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.downcast_ref().unwrap()
    }
}

impl<T> DerefMut for ConfigMut<'_, T>
where
    T: Source + Any,
{
    fn deref_mut(&mut self) -> &mut T {
        self.flags
            .fetch_or(CacheFlags::Dirty as u8, Ordering::Release);
        self.inner.downcast_mut().unwrap()
    }
}

impl<T> Drop for ConfigMut<'_, T>
where
    T: Source + Any,
{
    fn drop(&mut self) {
        self.flags
            .fetch_and((!CacheFlags::Writing).bits(), Ordering::Release);
    }
}

/// This trait is used to retrieve the config value from the cache.
#[allow(private_bounds, private_interfaces)]
pub trait Cacheable<T>
where
    Self: Any,
{
    /// Immutable reference retrieved from the cache.
    type Ref<'a>;
    /// Mutable reference retrieved from the cache.
    type Mut<'a>;
    /// Owned value retrieved from the cache.
    type Owned;
    /// Retrieve the immutable ref from the cache.
    fn retrieve(cache: &Cache) -> Self::Ref<'_>;
    /// Retrieve the mutable ref from the cache.
    fn retrieve_mut(cache: &Cache) -> Self::Mut<'_>;
    /// Take the ownership of the value from the cache.
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
            inner: unsafe { &*cache.get_or_default::<T>().inner.get() },
            _marker: PhantomData,
        }
    }

    fn retrieve_mut(cache: &Cache) -> Self::Mut<'_> {
        let value = cache.get_or_default::<T>();
        value
            .flags
            .fetch_or(CacheFlags::Writing as u8, Ordering::Release);
        ConfigMut {
            inner: unsafe { &mut *value.inner.get() },
            flags: &value.flags,
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
