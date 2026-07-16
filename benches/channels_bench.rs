use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rust_concurrency::channels::{bounded_channel, unbounded_channel};

fn main() {
    let iterations = 50_000usize;

    let (producer, consumer) = unbounded_channel();
    let start = Instant::now();
    for value in 0..iterations {
        producer.send(value).unwrap();
    }
    for _ in 0..iterations {
        black_box(consumer.recv().unwrap());
    }
    let unbounded_elapsed = start.elapsed();

    let (producer, consumer) = bounded_channel(64);
    let start = Instant::now();
    let sender = thread::spawn(move || {
        for value in 0..iterations {
            producer.send(value).unwrap();
        }
    });
    for _ in 0..iterations {
        black_box(consumer.recv().unwrap());
    }
    sender.join().unwrap();
    let bounded_elapsed = start.elapsed();

    let shared = Arc::new(Mutex::new(Vec::with_capacity(iterations)));
    let start = Instant::now();
    for value in 0..iterations {
        shared.lock().unwrap().push(value);
    }
    let mut drained = 0usize;
    while shared.lock().unwrap().pop().is_some() {
        drained += 1;
    }
    black_box(drained);
    let mutex_vec_elapsed = start.elapsed();

    println!("channels benchmark (manual, std::time::Instant)");
    println!("iterations: {iterations}");
    println!("unbounded send/recv: {unbounded_elapsed:?}");
    println!("bounded send/recv capacity 64: {bounded_elapsed:?}");
    println!("mutex vec push/pop baseline: {mutex_vec_elapsed:?}");
}
