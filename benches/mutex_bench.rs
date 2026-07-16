use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rust_concurrency::mutex::EducationalMutex;

fn main() {
    let iterations = 50_000usize;
    let worker_count = 4usize;

    let start = Instant::now();
    let mut plain = 0usize;
    for _ in 0..iterations {
        plain = black_box(plain + 1);
    }
    let plain_elapsed = start.elapsed();

    let start = Instant::now();
    let educational = EducationalMutex::new(0usize);
    for _ in 0..iterations {
        educational.with_lock(|value| *value += 1).unwrap();
    }
    let educational_uncontended = start.elapsed();

    let start = Instant::now();
    let shared = Arc::new(EducationalMutex::new(0usize));
    let mut workers = Vec::new();
    for _ in 0..worker_count {
        let shared = Arc::clone(&shared);
        workers.push(thread::spawn(move || {
            for _ in 0..iterations {
                shared.with_lock(|value| *value += 1).unwrap();
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let educational_contended = start.elapsed();

    let start = Instant::now();
    let std_shared = Arc::new(Mutex::new(0usize));
    let mut workers = Vec::new();
    for _ in 0..worker_count {
        let std_shared = Arc::clone(&std_shared);
        workers.push(thread::spawn(move || {
            for _ in 0..iterations {
                *std_shared.lock().unwrap() += 1;
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let std_contended = start.elapsed();

    println!("mutex benchmark (manual, std::time::Instant)");
    println!("iterations per worker: {iterations}");
    println!("plain sequential increment: {plain_elapsed:?} ({plain})");
    println!("educational mutex uncontended: {educational_uncontended:?}");
    println!("educational mutex contended: {educational_contended:?}");
    println!("std mutex contended baseline: {std_contended:?}");
}
