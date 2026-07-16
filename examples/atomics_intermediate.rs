use std::sync::Arc;
use std::thread;

use rust_concurrency::atomics::AtomicFlag;

fn main() {
    let ready = Arc::new(AtomicFlag::new(false));
    let worker_flag = Arc::clone(&ready);

    let worker = thread::spawn(move || {
        while !worker_flag.is_set() {
            thread::yield_now();
        }
        println!("bandera observada");
    });

    ready.set(true);
    worker.join().unwrap();
}
