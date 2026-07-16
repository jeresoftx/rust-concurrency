use std::thread;

use rust_concurrency::channels::{
    bounded_channel, unbounded_channel, ChannelReceiveError, TrySendFailure, WorkerPool,
};

#[test]
fn unbounded_channel_preserves_fifo_order() {
    let (producer, consumer) = unbounded_channel();

    producer.send("first").unwrap();
    producer.send("second").unwrap();
    producer.send("third").unwrap();

    assert_eq!(consumer.recv(), Ok("first"));
    assert_eq!(consumer.recv(), Ok("second"));
    assert_eq!(consumer.recv(), Ok("third"));
}

#[test]
fn receiver_observes_close_after_all_producers_drop() {
    let (producer, consumer) = unbounded_channel::<i32>();
    let producer_clone = producer.clone();

    producer.send(1).unwrap();

    drop(producer);
    assert_eq!(consumer.recv(), Ok(1));

    drop(producer_clone);
    assert_eq!(consumer.recv(), Err(ChannelReceiveError::Closed));
}

#[test]
fn cloned_producers_can_feed_one_consumer() {
    let (producer, consumer) = unbounded_channel();
    let mut handles = Vec::new();

    for worker in 0..4 {
        let producer = producer.clone();
        handles.push(thread::spawn(move || {
            producer.send(worker).unwrap();
        }));
    }

    drop(producer);

    for handle in handles {
        handle.join().unwrap();
    }

    let mut received = Vec::new();
    while let Ok(value) = consumer.recv() {
        received.push(value);
    }
    received.sort();

    assert_eq!(received, vec![0, 1, 2, 3]);
}

#[test]
fn bounded_channel_reports_backpressure_without_blocking() {
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

#[test]
fn bounded_channel_reports_closed_when_consumer_drops() {
    let (producer, consumer) = bounded_channel::<i32>(1);

    drop(consumer);

    assert_eq!(producer.try_send(7), Err(TrySendFailure::Closed(7)));
}

#[test]
fn worker_pool_processes_jobs_and_closes_outputs() {
    let pool = WorkerPool::new(3, |value: i32| value * value);

    for value in 1..=5 {
        pool.submit(value).unwrap();
    }

    let mut outputs = pool.shutdown();
    outputs.sort();

    assert_eq!(outputs, vec![1, 4, 9, 16, 25]);
}
