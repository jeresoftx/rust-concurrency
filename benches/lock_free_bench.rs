use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rust_concurrency::lock_free::BoundedLockFreeStack;

fn main() {
    let iterations = 50_000usize;

    let stack = BoundedLockFreeStack::new(iterations);
    let start = Instant::now();
    for value in 0..iterations {
        stack.push(value).unwrap();
    }
    for _ in 0..iterations {
        black_box(stack.pop().unwrap());
    }
    let lock_free_sequential = start.elapsed();

    let locked = Mutex::new(Vec::with_capacity(iterations));
    let start = Instant::now();
    for value in 0..iterations {
        locked.lock().unwrap().push(value);
    }
    for _ in 0..iterations {
        black_box(locked.lock().unwrap().pop().unwrap());
    }
    let mutex_sequential = start.elapsed();

    let workers = 4usize;
    let per_worker = 10_000usize;
    let stack = Arc::new(BoundedLockFreeStack::new(workers * per_worker));
    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..workers {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for offset in 0..per_worker {
                stack.push(worker * per_worker + offset).unwrap();
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let lock_free_concurrent_push = start.elapsed();

    let locked = Arc::new(Mutex::new(Vec::with_capacity(workers * per_worker)));
    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..workers {
        let locked = Arc::clone(&locked);
        handles.push(thread::spawn(move || {
            for offset in 0..per_worker {
                locked.lock().unwrap().push(worker * per_worker + offset);
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let mutex_concurrent_push = start.elapsed();

    println!("lock-free benchmark (manual, std::time::Instant)");
    println!("iterations: {iterations}");
    println!("lock-free sequential push/pop: {lock_free_sequential:?}");
    println!("mutex vec sequential push/pop: {mutex_sequential:?}");
    println!("lock-free concurrent push: {lock_free_concurrent_push:?}");
    println!("mutex vec concurrent push: {mutex_concurrent_push:?}");
    println!("lock-free observations: {:?}", stack.observations());
}
