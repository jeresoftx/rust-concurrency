use std::sync::Arc;
use std::thread;

use rust_concurrency::memory_ordering::{PublishedValue, RelaxedCounter};

fn main() {
    let published = Arc::new(PublishedValue::new(0));
    let metrics = Arc::new(RelaxedCounter::new(0));

    let writer_value = Arc::clone(&published);
    let writer_metrics = Arc::clone(&metrics);
    let writer = thread::spawn(move || {
        writer_value.publish(128);
        writer_metrics.increment();
    });

    writer.join().unwrap();

    println!("payload publicado: {:?}", published.try_read());
    println!("eventos de publicación: {}", metrics.load());
}
