mod common;

use libc::fork;
pub use process_sync::private::SharedMemoryObject;
use process_sync::{private::check_libc_err, SharedMutex};

use common::{sleep, TestOutput};

fn main() {
    let mut test_output = TestOutput::new(&[
        "child lock()",
        "child locked",
        "parent lock()",
        "child unlock()",
        "parent locked",
        "parent unlock()",
    ]);

    let mut mutex = SharedMutex::new().expect("cannot create SharedMutex");

    let pid = check_libc_err(unsafe { fork() }).expect("fork failed");
    if pid == 0 {
        // child
        test_output.write_line("child lock()");
        mutex.lock().expect("cannot lock child");
        test_output.write_line("child locked");
        sleep(60);
        test_output.write_line("child unlock()");
        mutex.unlock().expect("cannot unlock child");
        std::process::exit(0);
    }

    // parent
    sleep(20);
    test_output.write_line("parent lock()");
    mutex.lock().expect("cannot lock parent");
    test_output.write_line("parent locked");
    sleep(20);
    test_output.write_line("parent unlock()");
    mutex.unlock().expect("cannot unlock parent");
}
