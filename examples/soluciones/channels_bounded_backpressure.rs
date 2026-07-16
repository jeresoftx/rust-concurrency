use rust_concurrency::channels::{bounded_channel, TrySendFailure};

fn main() {
    let (producer, consumer) = bounded_channel(1);

    producer.try_send("first").unwrap();
    assert_eq!(
        producer.try_send("second"),
        Err(TrySendFailure::Full("second"))
    );

    assert_eq!(consumer.recv(), Ok("first"));
    producer.try_send("second").unwrap();
    assert_eq!(consumer.recv(), Ok("second"));
}
