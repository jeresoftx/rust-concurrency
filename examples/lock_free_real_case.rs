use std::sync::{Arc, Mutex};
use std::thread;

use rust_concurrency::lock_free::BoundedLockFreeStack;

fn main() {
    let stack = Arc::new(BoundedLockFreeStack::new(32));
    let fallback = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for value in 0..40 {
        let stack = Arc::clone(&stack);
        let fallback = Arc::clone(&fallback);
        handles.push(thread::spawn(move || {
            if stack.push(value).is_err() {
                fallback.lock().unwrap().push(value);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("capacidad lock-free: {}", stack.capacity());
    println!("fallback con lock: {}", fallback.lock().unwrap().len());
}
