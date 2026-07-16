use std::sync::Arc;
use std::thread;

use rust_concurrency::atomics::{
    AtomicCounter, AtomicFlag, AtomicMax, AtomicObservation, CompareExchange,
};

#[test]
fn counter_fetch_add_returns_previous_value_and_tracks_updates() {
    let counter = AtomicCounter::new(10);

    assert_eq!(counter.fetch_add(5), 10);
    assert_eq!(counter.load(), 15);
    assert_eq!(
        counter.observations(),
        AtomicObservation {
            loads: 1,
            stores: 0,
            fetch_adds: 1,
            compare_exchange_attempts: 0,
            compare_exchange_successes: 0,
        },
    );
}

#[test]
fn counter_saturates_on_overflow() {
    let counter = AtomicCounter::new(usize::MAX - 1);

    assert_eq!(counter.fetch_add(10), usize::MAX - 1);
    assert_eq!(counter.load(), usize::MAX);
}

#[test]
fn flag_can_be_published_and_observed() {
    let flag = AtomicFlag::new(false);

    assert!(!flag.is_set());
    flag.set(true);
    assert!(flag.is_set());
}

#[test]
fn compare_exchange_reports_success_and_failure() {
    let slot = CompareExchange::new(7);

    assert_eq!(slot.compare_exchange(7, 11), Ok(7));
    assert_eq!(slot.load(), 11);
    assert_eq!(slot.compare_exchange(7, 13), Err(11));
    assert_eq!(slot.load(), 11);

    let observations = slot.observations();
    assert_eq!(observations.compare_exchange_attempts, 2);
    assert_eq!(observations.compare_exchange_successes, 1);
}

#[test]
fn concurrent_counter_aggregation_does_not_lose_updates() {
    let counter = Arc::new(AtomicCounter::new(0));
    let mut workers = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        workers.push(thread::spawn(move || {
            for _ in 0..250 {
                counter.fetch_add(1);
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    assert_eq!(counter.load(), 1_000);
}

#[test]
fn cas_loop_records_largest_value() {
    let max = Arc::new(AtomicMax::new(10));
    let mut workers = Vec::new();

    for candidate in [8, 12, 11, 15, 14] {
        let max = Arc::clone(&max);
        workers.push(thread::spawn(move || {
            max.record(candidate);
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    assert_eq!(max.load(), 15);
    assert!(max.observations().compare_exchange_attempts >= 2);
}
