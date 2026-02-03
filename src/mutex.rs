use lock_api::{GuardSend, RawMutex};

use crate::{
    WaitStrategy,
    semaphore::{RawSemaphore, StaticSemaphore},
};

unsafe impl<S: WaitStrategy> RawMutex for StaticSemaphore<1, S> {
    type GuardMarker = GuardSend;

    const INIT: Self = Self::new();

    fn try_lock(&self) -> bool {
        self.try_down().is_ok()
    }

    fn lock(&self) {
        self.down();
    }

    unsafe fn unlock(&self) {
        unsafe { self.up() };
    }
}
