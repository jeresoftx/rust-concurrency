use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rust_concurrency::atomics::AtomicCounter;

fn main() {
    let iterations = 50_000usize;
    let worker_count = 4usize;

    let atomic = Arc::new(AtomicCounter::new(0));
    let start = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..worker_count {
        let atomic = Arc::clone(&atomic);
        workers.push(thread::spawn(move || {
            for _ in 0..iterations {
                atomic.fetch_add(1);
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let atomic_elapsed = start.elapsed();

    let mutex = Arc::new(Mutex::new(0usize));
    let start = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..worker_count {
        let mutex = Arc::clone(&mutex);
        workers.push(thread::spawn(move || {
            for _ in 0..iterations {
                *mutex.lock().unwrap() += 1;
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let mutex_elapsed = start.elapsed();

    let start = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..worker_count {
        workers.push(thread::spawn(move || {
            let mut local = 0usize;
            for _ in 0..iterations {
                local = black_box(local + 1);
            }
            local
        }));
    }
    let local_total: usize = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .sum();
    let local_elapsed = start.elapsed();

    println!("atomics benchmark (manual, std::time::Instant)");
    println!("workers: {worker_count}");
    println!("iterations per worker: {iterations}");
    println!("atomic increments: {atomic_elapsed:?} ({})", atomic.load());
    println!(
        "mutex increments: {mutex_elapsed:?} ({})",
        *mutex.lock().unwrap()
    );
    println!("thread-local aggregation: {local_elapsed:?} ({local_total})");
}
