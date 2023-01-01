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

pub struct SharedMutex {
    mutex: SharedMemoryObject<pthread_mutex_t>,
    owner_pid: pid_t,
}

impl SharedMutex {
    pub fn new() -> std::io::Result<Self> {
        let mutex = SharedMemoryObject::new(PTHREAD_MUTEX_INITIALIZER)?;
        initialize_mutex(mutex.get_mut())?;

        let owner_pid = getpid();
        Ok(Self { mutex, owner_pid })
    }

    pub fn lock(&mut self) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_mutex_lock(self.mutex.get_mut()) })?;
        Ok(())
    }

    pub fn unlock(&mut self) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_mutex_unlock(self.mutex.get_mut()) })?;
        Ok(())
    }
}

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
