#![cfg_attr(not(feature = "std"), no_std)]

use crate::sync::hint;

use thiserror::Error;

mod mutex;
mod rwlock;
mod semaphore;
mod sync;
#[cfg(test)]
mod tests;

pub mod locks {
    #![allow(type_alias_bounds)]
    use crate::{SpinWaiter, WaitStrategy, semaphore::StaticSemaphore};

    pub type GenericMutex<T, S: WaitStrategy> = lock_api::Mutex<StaticSemaphore<1, S>, T>;
    pub type GenericMutexGuard<'a, T, S: WaitStrategy> =
        lock_api::MutexGuard<'a, StaticSemaphore<1, S>, T>;
    pub type GenericRwLock<T, S: WaitStrategy> =
        lock_api::RwLock<StaticSemaphore<{ usize::MAX }, S>, T>;
    pub type GenericRwLockReadGuard<'a, T, S: WaitStrategy> =
        lock_api::RwLockReadGuard<'a, StaticSemaphore<{ usize::MAX }, S>, T>;
    pub type GenericRwLockWriteGuard<'a, T, S: WaitStrategy> =
        lock_api::RwLockWriteGuard<'a, StaticSemaphore<{ usize::MAX }, S>, T>;

    pub type SpinMutex<T> = GenericMutex<T, SpinWaiter>;
    pub type SpinMutexGuard<'a, T> = GenericMutexGuard<'a, T, SpinWaiter>;
    pub type SpinRwLock<T> = GenericRwLock<T, SpinWaiter>;
    pub type SpinRwLockReadGuard<'a, T> = GenericRwLockReadGuard<'a, T, SpinWaiter>;
    pub type SpinRwLockWriteGuard<'a, T> = GenericRwLockWriteGuard<'a, T, SpinWaiter>;

    #[cfg(feature = "std")]
    pub use std_::*;
    #[cfg(feature = "std")]
    mod std_ {
        use super::*;
        use crate::YieldWaiter;

        pub type Mutex<T> = GenericMutex<T, YieldWaiter>;
        pub type MutexGuard<'a, T> = GenericMutexGuard<'a, T, YieldWaiter>;
        pub type RwLock<T> = GenericRwLock<T, YieldWaiter>;
        pub type RwLockReadGuard<'a, T> = GenericRwLockReadGuard<'a, T, YieldWaiter>;
        pub type RwLockWriteGuard<'a, T> = GenericRwLockWriteGuard<'a, T, YieldWaiter>;
    }
}

pub trait StatelessWaitStrategy: Send {
    fn wait();
}

pub trait WaitStrategy: Send {
    const INIT: Self;
    fn wait(&self);
    fn signal(&self) {}
}

impl<S> WaitStrategy for S
where
    S: StatelessWaitStrategy,
{
    /// # SAFETY: StatelessWaitStrategies will/must always be zero sized.
    const INIT: Self = unsafe {
        const { assert!(core::mem::size_of::<S>() == 0) };
        core::mem::zeroed()
    };

    #[inline]
    fn wait(&self) {
        Self::wait();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpinWaiter;

impl StatelessWaitStrategy for SpinWaiter {
    #[inline]
    fn wait() {
        hint::spin_loop();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NoBlock;

impl StatelessWaitStrategy for NoBlock {
    fn wait() {
        panic!("Tried to wait on a NoBlock lock");
    }
}

#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug)]
pub struct YieldWaiter;

#[cfg(feature = "std")]
impl StatelessWaitStrategy for YieldWaiter {
    #[inline]
    fn wait() {
        crate::sync::thread::yield_now()
    }
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum SyncErr {
    #[error("tried to access a contended lock")]
    LockContended,
}
