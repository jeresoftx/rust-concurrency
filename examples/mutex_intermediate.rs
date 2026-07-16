use std::sync::Arc;
use std::thread;

use rust_concurrency::mutex::EducationalMutex;

fn main() {
    let counter = Arc::new(EducationalMutex::new(0usize));
    let mut workers = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        workers.push(thread::spawn(move || {
            for _ in 0..1_000 {
                counter.with_lock(|value| *value += 1).unwrap();
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    let final_value = counter.with_lock(|value| *value).unwrap();
    println!("contador final: {final_value}");
    println!("observaciones: {:?}", counter.observations());
}
