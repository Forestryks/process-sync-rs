mod mutex;
mod shared_memory;
mod util;

pub mod private {
    pub use crate::shared_memory::SharedMemoryObject;
    pub use crate::util::check_libc_err;
}

pub use mutex::SharedMutex;
