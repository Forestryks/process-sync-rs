use libc::{c_void, mmap, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE};
use std::{mem::size_of, ptr::null_mut};

/// An object that can be shared between processes.
///
/// After spawning child process (using `fork()`, `clone()`, etc.) updates to this object will be seen by both processes.
/// This is achieved by allocating memory using mmap with `MAP_SHARED` flag.
///
/// For more details see [man page](https://man7.org/linux/man-pages/man2/mmap.2.html).
///
/// # Example
/// ```rust
/// # use std::error::Error;
/// # use std::thread::sleep;
/// # use std::time::Duration;
/// #
/// # use libc::fork;
/// #
/// # use process_sync::private::check_libc_err;
/// # use process_sync::SharedMemoryObject;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// #
/// let mut shared = SharedMemoryObject::new(123)?;
///
/// let pid = unsafe { fork() };
/// assert!(pid >= 0);
///
/// if pid == 0 {
///     assert_eq!(*shared.get(), 123);
///     *shared.get_mut() = 456;
///     sleep(Duration::from_millis(40));
///     assert_eq!(*shared.get(), 789);
/// } else {
///     sleep(Duration::from_millis(20));
///     assert_eq!(*shared.get(), 456);
///     *shared.get_mut() = 789;
/// }
/// #
/// #     Ok(())
/// # }
/// ```
pub struct SharedMemoryObject<T> {
    ptr: *mut T,
}

impl<T: Sync + Send> SharedMemoryObject<T> {
    /// Allocates shared memory and moves `obj` there.
    ///
    /// # Errors
    /// If allocation fails returns error from [`last_os_error`].
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
    pub fn new(obj: T) -> std::io::Result<Self> {
        let addr = allocate_shared_memory(size_of::<T>())?;

        let addr = addr as *mut T;
        unsafe { *addr = obj };

        Ok(Self { ptr: addr })
    }

    /// Returns reference to underlying object.
    ///
    /// # Safety
    /// See [`get_mut`](#method.get_mut).
    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }

    /// Returns mutable reference to underlying object.
    ///
    /// # Safety
    /// This function (and [`get`](#method.get)) is always safe to call, but access to data under returned reference
    /// must be somehow synchronized with another processes to avoid data race.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for SharedMemoryObject<T> {
    fn drop(&mut self) {
        // every process owning shared memory object must free it individually
        free_shared_memory(self.ptr as *mut c_void, size_of::<T>())
            .expect("cannot munmap() shared memory");
    }
}

fn allocate_shared_memory(len: usize) -> std::io::Result<*mut c_void> {
    let addr = unsafe {
        mmap(
            null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if addr == MAP_FAILED {
        return Err(std::io::Error::last_os_error());
    }
    Ok(addr)
}

fn free_shared_memory(addr: *mut c_void, len: usize) -> std::io::Result<()> {
    let ret = unsafe { munmap(addr, len) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
