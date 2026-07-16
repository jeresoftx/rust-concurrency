use std::thread;

use rust_concurrency::channels::unbounded_channel;

fn main() {
    let (producer, consumer) = unbounded_channel();
    let mut handles = Vec::new();

    for source in ["api", "worker", "scheduler"] {
        let producer = producer.clone();
        handles.push(thread::spawn(move || {
            producer.send(format!("evento desde {source}")).unwrap();
        }));
    }

    drop(producer);

    for handle in handles {
        handle.join().unwrap();
    }

    while let Ok(event) = consumer.recv() {
        println!("{event}");
    }
}
