use rust_concurrency::channels::unbounded_channel;

fn main() {
    let (producer, consumer) = unbounded_channel();

    producer.send("first").unwrap();
    producer.send("second").unwrap();
    producer.send("third").unwrap();

    assert_eq!(consumer.recv(), Ok("first"));
    assert_eq!(consumer.recv(), Ok("second"));
    assert_eq!(consumer.recv(), Ok("third"));
}
