use std::sync::Arc;
use std::thread;

use rust_concurrency::atomics::AtomicFlag;

fn main() {
    let ready = Arc::new(AtomicFlag::new(false));
    let reader = Arc::clone(&ready);

    let worker = thread::spawn(move || {
        while !reader.is_set() {
            thread::yield_now();
        }
        "publicado"
    });

    ready.set(true);

    println!("{}", worker.join().unwrap());
}
