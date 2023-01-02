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

pub struct SharedCondvar {
    condvar: SharedMemoryObject<pthread_cond_t>,
    owner_pid: pid_t,
}

impl SharedCondvar {
    pub fn new() -> std::io::Result<Self> {
        let mut condvar = SharedMemoryObject::new(PTHREAD_COND_INITIALIZER)?;
        initialize_condvar(condvar.get_mut())?;

        let owner_pid = getpid();
        Ok(Self { condvar, owner_pid })
    }

    pub fn wait(&mut self, mutex: &mut SharedMutex) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_cond_wait(self.condvar.get_mut(), mutex.get_mut()) })?;
        Ok(())
    }

    pub fn notify_one(&mut self) -> std::io::Result<()> {
        check_libc_err(unsafe { pthread_cond_signal(self.condvar.get_mut()) })?;
        Ok(())
    }

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
