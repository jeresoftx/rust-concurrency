use std::sync::Arc;
use std::thread;

use rust_concurrency::memory_ordering::RelaxedCounter;

fn main() {
    let counter = Arc::new(RelaxedCounter::new(0));
    let mut workers = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        workers.push(thread::spawn(move || {
            for _ in 0..250 {
                counter.increment();
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    println!("total relaxed: {}", counter.load());
}
