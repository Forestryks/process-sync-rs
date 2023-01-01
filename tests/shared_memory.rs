mod common;

use libc::fork;
use process_sync::private::check_libc_err;
pub use process_sync::private::SharedMemoryObject;

use common::{sleep, TestOutput};

fn main() {
    let mut test_output = TestOutput::new(&["123", "123", "456", "789"]);

    let value = SharedMemoryObject::new(123).expect("cannot create SharedMemoryObject");

    let pid = check_libc_err(unsafe { fork() }).expect("fork failed");
    if pid == 0 {
        // child
        test_output.write_line(&format!("{}", value.get()));
        sleep(20);
        *value.get_mut() = 456;
        sleep(40);
        test_output.write_line(&format!("{}", value.get()));
        std::process::exit(0);
    }

    // parent
    test_output.write_line(&format!("{}", value.get()));
    sleep(40);
    test_output.write_line(&format!("{}", value.get()));
    *value.get_mut() = 789;
    sleep(40);
}
