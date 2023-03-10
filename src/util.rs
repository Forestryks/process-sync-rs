use libc::pid_t;

#[doc(hidden)]
pub fn check_libc_err<T: Default + Ord>(ret: T) -> std::io::Result<T> {
    if ret < T::default() {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(ret);
}

pub fn getpid() -> pid_t {
    check_libc_err(unsafe { libc::getpid() }).expect("getpid() failed")
}
