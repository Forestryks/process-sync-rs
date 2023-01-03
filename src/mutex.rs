use libc::{
    pid_t, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t,
    pthread_mutex_unlock, pthread_mutexattr_destroy, pthread_mutexattr_init,
    pthread_mutexattr_setpshared, pthread_mutexattr_t, PTHREAD_MUTEX_INITIALIZER,
    PTHREAD_PROCESS_SHARED,
};

use crate::{
    shared_memory::SharedMemoryObject,
    util::{check_libc_err, getpid},
};

/// Simple mutex that can be shared between processes.
///
/// This mutex is **NOT** recursive, so it will deadlock on relock.
///
/// Dropping mutex in creating process while mutex being locked or waited will cause undefined behaviour.
/// It is recommended to drop this mutex in creating process only after no other process has access to it.
///
/// For more information see [`pthread_mutex_init`](https://man7.org/linux/man-pages/man3/pthread_mutex_destroy.3p.html), [`pthread_mutex_lock`](https://man7.org/linux/man-pages/man3/pthread_mutex_lock.3p.html) and [`SharedMemoryObject`].
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
/// # use process_sync::SharedMutex;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// #
/// let mut mutex = SharedMutex::new()?;
///
/// let pid = unsafe { fork() };
/// assert!(pid >= 0);
///
/// if pid == 0 {
///     println!("child lock()");
///     mutex.lock()?;
///     println!("child locked");
///     sleep(Duration::from_millis(40));
///     println!("child unlock()");
///     mutex.unlock()?;
/// } else {
///     sleep(Duration::from_millis(20));
///     println!("parent lock()");
///     mutex.lock()?;
///     println!("parent locked");
///     sleep(Duration::from_millis(20));
///     println!("parent unlock()");
///     mutex.unlock()?;
/// }
/// #
/// #     Ok(())
/// # }
/// ```
///
/// Output:
/// ```txt
/// child lock()
/// child locked
/// parent lock()
/// child unlock()
/// parent locked
/// parent unlock()
/// ```
pub struct SharedMutex {
    mutex: SharedMemoryObject<pthread_mutex_t>,
    owner_pid: pid_t,
}

impl SharedMutex {
    /// Creates new [`SharedMutex`]
    ///
    /// # Errors
    /// If allocation or initialization fails returns error from [`last_os_error`].
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error.
    pub fn new() -> std::io::Result<Self> {
        let mut mutex = SharedMemoryObject::new(PTHREAD_MUTEX_INITIALIZER)?;
        initialize_mutex(mutex.get_mut())?;

        let owner_pid = getpid();
        Ok(Self { mutex, owner_pid })
    }

    /// Locks mutex.
    ///
    /// This function will block until mutex is locked.
    ///
    /// # Errors
    /// If any pthread call fails, returns error from [`last_os_error`]. For possible errors see [`pthread_mutex_lock`](https://man7.org/linux/man-pages/man3/pthread_mutex_lock.3p.html).
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
    pub fn lock(&mut self) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_mutex_lock(self.mutex.get_mut()) })?;
        Ok(())
    }

    /// Unlocks mutex.
    ///
    /// This function must be called from the same process that called [`lock`](#method.lock) previously.
    ///
    /// # Errors
    /// If any pthread call fails, returns error from [`last_os_error`]. For possible errors see [`pthread_mutex_lock`](https://man7.org/linux/man-pages/man3/pthread_mutex_lock.3p.html).
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
    pub fn unlock(&mut self) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_mutex_unlock(self.mutex.get_mut()) })?;
        Ok(())
    }

    pub(crate) fn get_mut(&mut self) -> *mut pthread_mutex_t {
        self.mutex.get_mut()
    }
}

// TODO: document drop behaviour
impl Drop for SharedMutex {
    fn drop(&mut self) {
        if getpid() == self.owner_pid {
            check_libc_err(unsafe { pthread_mutex_destroy(self.mutex.get_mut()) })
                .expect("cannot destroy mutex");
        }
    }
}

fn initialize_mutex(mutex: &mut pthread_mutex_t) -> std::io::Result<()> {
    let mut attr: pthread_mutexattr_t = unsafe { std::mem::zeroed() };
    check_libc_err(unsafe { pthread_mutexattr_init(&mut attr) })?;

    check_libc_err(unsafe { pthread_mutexattr_setpshared(&mut attr, PTHREAD_PROCESS_SHARED) })
        .expect("cannot set PTHREAD_PROCESS_SHARED");

    let ret = check_libc_err(unsafe { pthread_mutex_init(mutex, &mut attr) });

    destroy_mutexattr(attr).expect("cannot destroy mutexattr");

    ret.map(|_| ())
}

fn destroy_mutexattr(mut attr: pthread_mutexattr_t) -> std::io::Result<()> {
    check_libc_err(unsafe { pthread_mutexattr_destroy(&mut attr) })?;
    Ok(())
}
