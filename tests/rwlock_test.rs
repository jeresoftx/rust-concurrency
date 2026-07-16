use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rust_concurrency::rwlock::{EducationalRwLock, RwLockObservation};

#[test]
fn allows_multiple_readers_to_observe_the_same_snapshot() {
    let shared = Arc::new(EducationalRwLock::new(vec![1, 2, 3]));
    let first = Arc::clone(&shared);
    let second = Arc::clone(&shared);

    let first_reader = thread::spawn(move || first.with_read(|items| items.iter().sum::<i32>()));
    let second_reader = thread::spawn(move || second.with_read(|items| items.len()));

    assert_eq!(first_reader.join().unwrap().unwrap(), 6);
    assert_eq!(second_reader.join().unwrap().unwrap(), 3);
    assert_eq!(shared.observations().read_attempts, 2);
}

#[test]
fn writer_waits_until_read_guard_is_released() {
    let shared = EducationalRwLock::new(10usize);
    let read_guard = shared.read().unwrap();

    assert_eq!(shared.try_with_write(|value| *value += 1), None);
    drop(read_guard);

    assert_eq!(
        shared.try_with_write(|value| {
            *value += 1;
            *value
        }),
        Some(11)
    );
}

#[test]
fn writes_are_visible_to_later_readers() {
    let shared = EducationalRwLock::new(String::from("draft"));

    shared
        .with_write(|value| value.push_str("-reviewed"))
        .unwrap();

    assert_eq!(
        shared.with_read(|value| value.clone()).unwrap(),
        "draft-reviewed"
    );
}

#[test]
fn poisoning_is_observable_and_recoverable_after_writer_panic() {
    let shared = Arc::new(EducationalRwLock::new(40usize));
    let poisoned = Arc::clone(&shared);

    let _ = thread::spawn(move || {
        let mut guard = poisoned.write().unwrap();
        *guard += 1;
        panic!("panic while holding the write lock");
    })
    .join();

    assert!(shared.is_poisoned());

    let recovered = shared.recover_write_with(|value| {
        *value += 1;
        *value
    });

    assert_eq!(recovered, 42);
    assert_eq!(shared.with_read(|value| *value).unwrap(), 42);
    assert!(!shared.is_poisoned());
}

#[test]
fn observations_track_read_write_and_contention_signals() {
    let shared = EducationalRwLock::new(7usize);
    let read_guard = shared.read().unwrap();

    assert_eq!(shared.try_with_write(|value| *value), None);
    drop(read_guard);
    assert_eq!(shared.try_with_read(|value| *value), Some(7));

    assert_eq!(
        shared.observations(),
        RwLockObservation {
            read_attempts: 1,
            write_attempts: 0,
            try_read_attempts: 1,
            try_write_attempts: 1,
            try_read_contentions: 0,
            try_write_contentions: 1,
            poison_recoveries: 0,
        },
    );
}

#[test]
fn read_heavy_updates_keep_consistent_state() {
    let shared = Arc::new(EducationalRwLock::new(0usize));
    let mut workers = Vec::new();

    for _ in 0..4 {
        let shared = Arc::clone(&shared);
        workers.push(thread::spawn(move || {
            for _ in 0..50 {
                shared
                    .with_read(|value| {
                        assert!(*value <= 20);
                    })
                    .unwrap();
                thread::sleep(Duration::from_micros(10));
            }
        }));
    }

    for _ in 0..20 {
        shared.with_write(|value| *value += 1).unwrap();
    }

    for worker in workers {
        worker.join().unwrap();
    }

    assert_eq!(shared.with_read(|value| *value).unwrap(), 20);
}
