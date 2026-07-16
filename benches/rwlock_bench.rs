use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let iterations = 50_000usize;
    let worker_count = 4usize;

    let read_heavy = Arc::new(EducationalRwLock::new(10usize));
    let start = Instant::now();
    let mut readers = Vec::new();
    for _ in 0..worker_count {
        let read_heavy = Arc::clone(&read_heavy);
        readers.push(thread::spawn(move || {
            let mut local = 0usize;
            for _ in 0..iterations {
                local = black_box(local + read_heavy.with_read(|value| *value).unwrap());
            }
            local
        }));
    }
    let read_total: usize = readers
        .into_iter()
        .map(|reader| reader.join().unwrap())
        .sum();
    let read_heavy_elapsed = start.elapsed();

    let write_heavy = Arc::new(EducationalRwLock::new(0usize));
    let start = Instant::now();
    let mut writers = Vec::new();
    for _ in 0..worker_count {
        let write_heavy = Arc::clone(&write_heavy);
        writers.push(thread::spawn(move || {
            for _ in 0..iterations {
                write_heavy.with_write(|value| *value += 1).unwrap();
            }
        }));
    }
    for writer in writers {
        writer.join().unwrap();
    }
    let write_heavy_elapsed = start.elapsed();

    let balanced = Arc::new(EducationalRwLock::new(0usize));
    let start = Instant::now();
    let mut workers = Vec::new();
    for id in 0..worker_count {
        let balanced = Arc::clone(&balanced);
        workers.push(thread::spawn(move || {
            for step in 0..iterations {
                if (step + id) % 4 == 0 {
                    balanced.with_write(|value| *value += 1).unwrap();
                } else {
                    black_box(balanced.with_read(|value| *value).unwrap());
                }
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let balanced_elapsed = start.elapsed();

    println!("rwlock benchmark (manual, std::time::Instant)");
    println!("workers: {worker_count}");
    println!("iterations per worker: {iterations}");
    println!("read-heavy workload: {read_heavy_elapsed:?} ({read_total})");
    println!("write-heavy workload: {write_heavy_elapsed:?}");
    println!("balanced workload: {balanced_elapsed:?}");
}
