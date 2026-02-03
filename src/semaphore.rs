use core::fmt::Debug;

use crate::sync::atomic::{AtomicUsize, Ordering};
use crate::{SyncErr, WaitStrategy};
use lock_api::GuardSend;

pub(crate) unsafe trait RawSemaphore {
    type GuardMaker;
    fn try_down(&self) -> Result<(), SyncErr>;
    fn down(&self);
    fn try_down_n(&self, n: usize) -> Result<(), SyncErr>;
    fn down_n(&self, n: usize);
    unsafe fn up(&self);
    unsafe fn up_n(&self, n: usize);
}

pub struct DynamicSemaphore<S: WaitStrategy> {
    counter: AtomicUsize,
    strategy: S,
}

impl<S: WaitStrategy> DynamicSemaphore<S> {
    pub const fn new(counter: usize) -> Self {
        Self {
            counter: AtomicUsize::new(counter),
            strategy: S::INIT,
        }
    }
}

unsafe impl<S: WaitStrategy> RawSemaphore for DynamicSemaphore<S> {
    type GuardMaker = GuardSend;

    fn try_down(&self) -> Result<(), SyncErr> {
        self.counter
            .fetch_update(Ordering::Acquire, Ordering::Relaxed, |counter| {
                counter.checked_sub(1)
            })
            .map_err(|_| SyncErr::LockContended)?;

        Ok(())
    }

    fn down(&self) {
        loop {
            if self.try_down().is_ok() {
                return;
            }
            self.strategy.wait();
        }
    }

    unsafe fn up(&self) {
        self.counter.fetch_add(1, Ordering::Release);
        self.strategy.signal();
    }

    fn try_down_n(&self, n: usize) -> Result<(), SyncErr> {
        self.counter
            .fetch_update(Ordering::Release, Ordering::Relaxed, |counter| {
                counter.checked_sub(n)
            })
            .map_err(|_| SyncErr::LockContended)?;

        Ok(())
    }

    fn down_n(&self, n: usize) {
        loop {
            if self.try_down_n(n).is_ok() {
                return;
            }
            self.strategy.wait();
        }
    }

    unsafe fn up_n(&self, n: usize) {
        self.counter.fetch_add(n, Ordering::Release);
        self.strategy.signal();
    }
}

impl<S: Clone + WaitStrategy> Clone for DynamicSemaphore<S> {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter.load(Ordering::Acquire).into(),
            strategy: self.strategy.clone(),
        }
    }
}

impl<S: WaitStrategy> Debug for DynamicSemaphore<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Dynamic Sema with count {}",
            self.counter.load(Ordering::Relaxed),
        )
    }
}

pub struct StaticSemaphore<const N: usize, S: WaitStrategy> {
    inner: DynamicSemaphore<S>,
}

impl<const N: usize, S: WaitStrategy> StaticSemaphore<N, S> {
    pub const fn new() -> Self {
        Self {
            inner: DynamicSemaphore::new(N),
        }
    }
}

unsafe impl<const N: usize, S: WaitStrategy> RawSemaphore for StaticSemaphore<N, S> {
    type GuardMaker = GuardSend;

    fn try_down(&self) -> Result<(), SyncErr> {
        self.inner.try_down()
    }

    fn down(&self) {
        self.inner.down();
    }

    unsafe fn up(&self) {
        debug_assert!(self.inner.counter.load(Ordering::Relaxed) < N);
        unsafe { self.inner.up() };
    }

    fn try_down_n(&self, n: usize) -> Result<(), SyncErr> {
        self.inner.try_down_n(n)
    }

    fn down_n(&self, n: usize) {
        self.inner.down_n(n);
    }

    unsafe fn up_n(&self, n: usize) {
        debug_assert!(self.inner.counter.load(Ordering::Relaxed) < N - n);
        unsafe {
            self.inner.up_n(n);
        }
    }
}

impl<const N: usize, S: Clone + WaitStrategy> Clone for StaticSemaphore<N, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<const N: usize, S: WaitStrategy> Debug for StaticSemaphore<N, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Static Sema with inner {:?}", self.inner)
    }
}
