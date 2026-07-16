use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use rust_concurrency::memory_ordering::{
    describe_ordering, OrderingCasCell, OrderingGuarantee, PublishedValue, RelaxedCounter,
};

#[test]
fn release_acquire_publication_makes_payload_visible() {
    let published = Arc::new(PublishedValue::new(0));
    let writer = Arc::clone(&published);

    let handle = thread::spawn(move || {
        writer.publish(42);
    });

    handle.join().unwrap();

    assert_eq!(published.try_read(), Some(42));
    assert_eq!(published.observed_reads(), 1);
}

#[test]
fn relaxed_counter_aggregates_without_publication_claims() {
    let counter = Arc::new(RelaxedCounter::new(0));
    let mut workers = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        workers.push(thread::spawn(move || {
            for _ in 0..250 {
                counter.increment();
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    assert_eq!(counter.load(), 1_000);
    assert_eq!(counter.ordering(), Ordering::Relaxed);
}

#[test]
fn compare_exchange_uses_explicit_success_and_failure_orderings() {
    let cell = OrderingCasCell::new(10);

    assert_eq!(
        cell.compare_exchange(10, 11, Ordering::AcqRel, Ordering::Acquire),
        Ok(10)
    );
    assert_eq!(
        cell.compare_exchange(10, 12, Ordering::AcqRel, Ordering::Acquire),
        Err(11)
    );

    let observation = cell.observation();
    assert_eq!(observation.value, 11);
    assert_eq!(observation.attempts, 2);
    assert_eq!(observation.successes, 1);
    assert_eq!(observation.last_success_ordering, Some(Ordering::AcqRel));
    assert_eq!(observation.last_failure_ordering, Some(Ordering::Acquire));
}

#[test]
fn ordering_descriptions_name_their_guarantees() {
    assert_eq!(
        describe_ordering(Ordering::Relaxed),
        OrderingGuarantee {
            ordering: Ordering::Relaxed,
            synchronizes_with_other_threads: false,
            preserves_single_variable_atomicity: true,
            participates_in_global_order: false,
        },
    );
    assert!(describe_ordering(Ordering::Acquire).synchronizes_with_other_threads);
    assert!(describe_ordering(Ordering::Release).synchronizes_with_other_threads);
    assert!(describe_ordering(Ordering::SeqCst).participates_in_global_order);
}

#[test]
fn acquire_read_before_release_publication_returns_none() {
    let published = PublishedValue::new(7);

    assert_eq!(published.try_read(), None);
    published.publish(9);
    assert_eq!(published.try_read(), Some(9));
}
