use rust_concurrency::channels::unbounded_channel;

fn main() {
    let (producer, consumer) = unbounded_channel();

    producer.send("first").unwrap();
    producer.send("second").unwrap();

    println!("{}", consumer.recv().unwrap());
    println!("{}", consumer.recv().unwrap());
}
