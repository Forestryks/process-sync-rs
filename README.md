# Process Sync [![Crates.io](https://img.shields.io/crates/v/process-sync.svg)](https://crates.io/crates/process-sync) [![Documentation](https://docs.rs/process-sync/badge.svg)](https://docs.rs/process-sync/)

Adds synchronization primitives that can be shared between multiple processes

## Shared memory

```rust
let mut shared = SharedMemoryObject::new(123)?;

let pid = unsafe { fork() };
assert!(pid >= 0);

if pid == 0 {
    assert_eq!(*shared.get(), 123);
    *shared.get_mut() = 456;
    sleep(Duration::from_millis(40));
    assert_eq!(*shared.get(), 789);
} else {
    sleep(Duration::from_millis(20));
    assert_eq!(*shared.get(), 456);
    *shared.get_mut() = 789;
}
```

## Mutex

```rust
let mut mutex = SharedMutex::new()?;

let pid = unsafe { fork() };
assert!(pid >= 0);

if pid == 0 {
    println!("child lock()");
    mutex.lock()?;
    println!("child locked");
    sleep(Duration::from_millis(40));
    println!("child unlock()");
    mutex.unlock()?;
} else {
    sleep(Duration::from_millis(20));
    println!("parent lock()");
    mutex.lock()?;
    println!("parent locked");
    sleep(Duration::from_millis(20));
    println!("parent unlock()");
    mutex.unlock()?;
}
```

## Condvar

```rust
let mut mutex = SharedMutex::new()?;
let mut condvar = SharedCondvar::new()?;

let pid = unsafe { fork() };
assert!(pid >= 0);

if pid == 0 {
    println!("child lock()");
    mutex.lock()?;
    println!("child wait()");
    condvar.wait(&mut mutex)?;
    println!("child notified");
    mutex.unlock()?;
    println!("child unlocked");
} else {
    sleep(Duration::from_millis(40));
    println!("parent notify()");
    condvar.notify_one()?;
}
```
