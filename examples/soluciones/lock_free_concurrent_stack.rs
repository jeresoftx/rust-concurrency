use std::collections::BTreeSet;
use std::sync::Arc;
use std::thread;

use rust_concurrency::lock_free::BoundedLockFreeStack;

fn main() {
    let stack = Arc::new(BoundedLockFreeStack::new(16));
    let mut handles = Vec::new();

    for worker in 0..4 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for offset in 0..4 {
                stack.push(worker * 10 + offset).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut values = BTreeSet::new();
    while let Some(value) = stack.pop() {
        values.insert(value);
    }

    assert_eq!(values.len(), 16);
}
