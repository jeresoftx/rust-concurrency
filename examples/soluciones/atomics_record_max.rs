use std::sync::Arc;
use std::thread;

use rust_concurrency::atomics::AtomicMax;

fn main() {
    let max = Arc::new(AtomicMax::new(0));
    let mut workers = Vec::new();

    for candidate in [8, 13, 5, 21, 19] {
        let max = Arc::clone(&max);
        workers.push(thread::spawn(move || {
            max.record(candidate);
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    println!("máximo observado: {}", max.load());
}
