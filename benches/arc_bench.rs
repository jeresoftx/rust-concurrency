use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use rust_concurrency::arc::Shared;

fn main() {
    let iterations = 100_000usize;
    let worker_count = 4usize;

    let shared = Arc::new(0usize);
    let start = Instant::now();
    for _ in 0..iterations {
        let cloned = Arc::clone(&shared);
        black_box(cloned);
    }
    let std_clone_drop = start.elapsed();

    let shared = Shared::new(0usize);
    let start = Instant::now();
    for _ in 0..iterations {
        let cloned = shared.clone_shared();
        black_box(cloned);
    }
    let educational_clone_drop = start.elapsed();

    let data = Shared::new(vec![1usize; 1024]);
    let start = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..worker_count {
        let data = data.clone_shared();
        workers.push(thread::spawn(move || {
            let mut total = 0usize;
            for _ in 0..iterations {
                total = black_box(total + data.with_ref(|items| items[0]));
            }
            total
        }));
    }
    let shared_total: usize = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .sum();
    let shared_read = start.elapsed();

    let data = vec![1usize; 1024];
    let start = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..worker_count {
        let data = data.clone();
        workers.push(thread::spawn(move || {
            let mut total = 0usize;
            for _ in 0..iterations {
                total = black_box(total + data[0]);
            }
            total
        }));
    }
    let cloned_total: usize = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .sum();
    let cloned_read = start.elapsed();

    println!("arc benchmark (manual, std::time::Instant)");
    println!("iterations: {iterations}");
    println!("std Arc clone/drop: {std_clone_drop:?}");
    println!("educational Shared clone/drop: {educational_clone_drop:?}");
    println!("shared read across threads: {shared_read:?} ({shared_total})");
    println!("cloned data read across threads: {cloned_read:?} ({cloned_total})");
}
