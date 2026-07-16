use std::sync::Arc;
use std::thread;

use rust_concurrency::atomics::{AtomicCounter, AtomicMax};

fn main() {
    let processed = Arc::new(AtomicCounter::new(0));
    let largest_batch = Arc::new(AtomicMax::new(0));
    let batches = [32usize, 48, 24, 64];
    let mut workers = Vec::new();

    for batch in batches {
        let processed = Arc::clone(&processed);
        let largest_batch = Arc::clone(&largest_batch);
        workers.push(thread::spawn(move || {
            processed.fetch_add(batch);
            largest_batch.record(batch);
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    println!("eventos procesados: {}", processed.load());
    println!("lote más grande: {}", largest_batch.load());
}
