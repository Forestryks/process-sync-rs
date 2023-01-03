mod common;

use libc::fork;
pub use process_sync::private::SharedMemoryObject;
use process_sync::{private::check_libc_err, SharedCondvar, SharedMutex};

use common::{sleep, TestOutput};

fn child0(test_output: &mut TestOutput, mutex: &mut SharedMutex, condvar: &mut SharedCondvar) -> ! {
    test_output.write_line("child0 lock()");
    mutex.lock().expect("lock() failed");
    test_output.write_line("child0 wait()");
    condvar.wait(mutex).expect("wait() failed");
    mutex.unlock().expect("unlock() failed");
    test_output.write_line("child0 unlocked");
    sleep(5);

    test_output.write_line("child0 FIRST_TEST_END");
    sleep(100);

    test_output.write_line("child0 lock()");
    mutex.lock().expect("lock() failed");
    test_output.write_line("child0 wait()");
    condvar.wait(mutex).expect("wait() failed");
    test_output.write_line("child0 unlock()");
    mutex.unlock().expect("unlock() failed");
    test_output.write_line("child0 unlocked");

    std::process::exit(0);
}

fn child1(test_output: &mut TestOutput, mutex: &mut SharedMutex, condvar: &mut SharedCondvar) {
    sleep(10);
    test_output.write_line("child1 lock()");
    mutex.lock().expect("lock() failed");
    test_output.write_line("child1 wait()");
    condvar.wait(mutex).expect("wait() failed");
    mutex.unlock().expect("unlock() failed");
    sleep(10);
    test_output.write_line("child1 unlocked");

    test_output.write_line("child1 FIRST_TEST_END");
    sleep(50);

    test_output.write_line("child1 lock()");
    mutex.lock().expect("lock() failed");
    test_output.write_line("child1 wait()");
    condvar.wait(mutex).expect("wait() failed");
    test_output.write_line("child1 unlock()");
    mutex.unlock().expect("unlock() failed");
    test_output.write_line("child1 unlocked");

    std::process::exit(0);
}

fn parent(test_output: &mut TestOutput, _mutex: &mut SharedMutex, condvar: &mut SharedCondvar) {
    sleep(40);
    test_output.write_line("parent notify_all()");
    condvar.notify_all().expect("notify_all() failed");

    test_output.write_line("parent FIRST_TEST_END");
    sleep(300);

    test_output.write_line("parent notify_one()");
    condvar.notify_one().expect("notify_one() failed");
    sleep(10);
    test_output.write_line("parent notify_one()");
    condvar.notify_one().expect("notify_one() failed");

    sleep(40);
}

fn main() {
    let mut test_output = TestOutput::new(&[
        "child0 lock()",
        "child0 wait()",
        "child1 lock()",
        "child1 wait()",
        "parent notify_all()",
        "parent FIRST_TEST_END",
        "child0 unlocked",
        "child0 FIRST_TEST_END",
        "child1 unlocked",
        "child1 FIRST_TEST_END",
        "child1 lock()",
        "child1 wait()",
        "child0 lock()",
        "child0 wait()",
        "parent notify_one()",
        "child1 unlock()",
        "child1 unlocked",
        "parent notify_one()",
        "child0 unlock()",
        "child0 unlocked",
    ]);

    let mut mutex = SharedMutex::new().expect("cannot create SharedMutex");
    let mut condvar = SharedCondvar::new().expect("cannot create SharedCondvar");

    let pid = check_libc_err(unsafe { fork() }).expect("fork failed");
    if pid == 0 {
        child0(&mut test_output, &mut mutex, &mut condvar);
    } else {
        let pid = check_libc_err(unsafe { fork() }).expect("fork failed");
        if pid == 0 {
            child1(&mut test_output, &mut mutex, &mut condvar);
        } else {
            parent(&mut test_output, &mut mutex, &mut condvar);
        }
    }
}
