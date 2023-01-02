//! Synchronization primitives for using in multiprocess environments.
//!
//! Implementation is based on `pthread` bindings.
//! TODO: write about how they actually work

#![warn(missing_docs)]
// #![deny(missing_doc_code_examples)]

mod condvar;
mod mutex;
mod shared_memory;
mod util;

#[doc(hidden)]
pub mod private {
    pub use crate::shared_memory::SharedMemoryObject;
    pub use crate::util::check_libc_err;
}

pub use condvar::SharedCondvar;
pub use mutex::SharedMutex;
pub use shared_memory::SharedMemoryObject;
