#![allow(unused_imports)]

#[cfg(any(not(test), all(not(loom), not(shuttle))))]
pub use core_::*;
#[cfg(all(loom, test))]
pub use loom_::*;
#[cfg(all(shuttle, test))]
pub use shuttle_::*;

#[cfg(all(shuttle, test))]
mod shuttle_ {
    #[allow(unused_imports)]
    pub use shuttle::hint;
    pub use shuttle::sync::atomic;
    pub use shuttle::thread;
}

#[cfg(all(loom, test))]
mod loom_ {
    pub use loom::cell;
    pub use loom::hint;
    pub use loom::sync::Arc;
    pub use loom::sync::atomic;
    pub use loom::thread;
}

#[cfg(any(not(test), all(not(loom), not(shuttle))))]
mod core_ {
    pub use core::hint;
    pub use core::sync::atomic;

    #[cfg(feature = "std")]
    pub use std::thread;
}
