use libc::{c_void, mmap, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE};
use std::{mem::size_of, ptr::null_mut};

pub struct SharedMemoryObject<T> {
    ptr: *mut T,
}

impl<T: Sync + Send> SharedMemoryObject<T> {
    pub fn new(obj: T) -> std::io::Result<Self> {
        let addr = allocate_shared_memory(size_of::<T>())?;

        let addr = addr as *mut T;
        unsafe { *addr = obj };

        Ok(Self { ptr: addr })
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&self) -> &mut T {
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
        println!("wtf {:?}", std::io::Error::last_os_error());
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

/*
#include <sys/mman.h>

    ~SharedMemoryObject() {
        if (free_shared_memory(ptr_, sizeof(T)) != 0) {
            die(format("Cannot free shared memory at %p: %m", ptr_));
        }
    }

    T *get() {
        return ptr_;
    }

    T *operator->() {
        return ptr_;
    }

private:
    T *ptr_;
};

#endif //LIBSBOX_SHARED_MEMORY_OBJECT_H
*/
