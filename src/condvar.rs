use libc::{
    pid_t, pthread_cond_broadcast, pthread_cond_destroy, pthread_cond_init, pthread_cond_signal,
    pthread_cond_t, pthread_cond_wait, pthread_condattr_destroy, pthread_condattr_init,
    pthread_condattr_setpshared, pthread_condattr_t, PTHREAD_COND_INITIALIZER,
    PTHREAD_PROCESS_SHARED,
};

use crate::{
    shared_memory::SharedMemoryObject,
    util::{check_libc_err, getpid},
    SharedMutex,
};

/// Simple conditional variable that can be shared between processes and used with [`SharedMutex`]
///
/// Dropping conditional variable in creating process while it being used by another process will cause undefined behaviour.
/// It is recommended to drop this conditional variable in creating process only after no other process has access to it.
///
/// For more information see [`pthread_cond_init`](https://man7.org/linux/man-pages/man3/pthread_cond_init.3p.html), [`pthread_cond_wait`](https://man7.org/linux/man-pages/man3/pthread_cond_wait.3p.html), [`SharedMutex`] and [`SharedMemoryObject`].
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
/// # use process_sync::SharedCondvar;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// #
/// let mut mutex = SharedMutex::new()?;
/// let mut condvar = SharedCondvar::new()?;
///
/// let pid = unsafe { fork() };
/// assert!(pid >= 0);
///
/// if pid == 0 {
///     println!("child lock()");
///     mutex.lock()?;
///     println!("child wait()");
///     condvar.wait(&mut mutex)?;
///     println!("child notified");
///     mutex.unlock()?;
///     println!("child unlocked");
/// } else {
///     sleep(Duration::from_millis(40));
///     println!("parent notify()");
///     condvar.notify_one()?;
/// }
/// #
/// #     Ok(())
/// # }
/// ```
///
/// Output:
/// ```txt
/// child lock()
/// child wait()
/// parent notify()
/// child notified
/// child unlocked
/// ```
pub struct SharedCondvar {
    condvar: SharedMemoryObject<pthread_cond_t>,
    owner_pid: pid_t,
}

impl SharedCondvar {
    /// Creates new [`SharedCondvar`]
    ///
    /// # Errors
    /// If allocation or initialization fails returns error from [`last_os_error`].
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
    pub fn new() -> std::io::Result<Self> {
        let mut condvar = SharedMemoryObject::new(PTHREAD_COND_INITIALIZER)?;
        initialize_condvar(condvar.get_mut())?;

        let owner_pid = getpid();
        Ok(Self { condvar, owner_pid })
    }

    /// Waits on given mutex
    ///
    /// This function will block until notified by another process
    ///
    /// # Errors
    /// If any pthread call fails, returns error from [`last_os_error`]. For possible errors see [`pthread_cond_wait`](https://man7.org/linux/man-pages/man3/pthread_cond_wait.3p.html).
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
    pub fn wait(&mut self, mutex: &mut SharedMutex) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_cond_wait(self.condvar.get_mut(), mutex.get_mut()) })?;
        Ok(())
    }

    /// Notifies one of processes that are waiting on this condvar
    ///
    /// # Errors
    /// If any pthread call fails, returns error from [`last_os_error`]. For possible errors see [`pthread_cond_signal`](https://man7.org/linux/man-pages/man3/pthread_cond_broadcast.3p.html).
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
    pub fn notify_one(&mut self) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_cond_signal(self.condvar.get_mut()) })?;
        Ok(())
    }

    /// Notifies all processes that are waiting on this condvar
    ///
    /// # Errors
    /// If any pthread call fails, returns error from [`last_os_error`]. For possible errors see [`pthread_cond_broadcast`](https://man7.org/linux/man-pages/man3/pthread_cond_broadcast.3p.html).
    ///
    /// [`last_os_error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
    pub fn notify_all(&mut self) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_cond_broadcast(self.condvar.get_mut()) })?;
        Ok(())
    }
}

impl Drop for SharedCondvar {
    fn drop(&mut self) {
        if getpid() == self.owner_pid {
            check_libc_err(unsafe { pthread_cond_destroy(self.condvar.get_mut()) })
                .expect("cannot destroy mutex");
        }
    }
}

fn initialize_condvar(condvar: &mut pthread_cond_t) -> std::io::Result<()> {
    let mut attr: pthread_condattr_t = unsafe { std::mem::zeroed() };
    check_libc_err(unsafe { pthread_condattr_init(&mut attr) })?;

    check_libc_err(unsafe { pthread_condattr_setpshared(&mut attr, PTHREAD_PROCESS_SHARED) })
        .expect("cannot set PTHREAD_PROCESS_SHARED");

    let ret = check_libc_err(unsafe { pthread_cond_init(condvar, &mut attr) });

    destroy_condattr(attr).expect("cannot destroy condattr");

    ret.map(|_| ())
}

fn destroy_condattr(mut attr: pthread_condattr_t) -> std::io::Result<()> {
    check_libc_err(unsafe { pthread_condattr_destroy(&mut attr) })?;
    Ok(())
}
