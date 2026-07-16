use std::sync::Arc;
use std::thread;

use rust_concurrency::mutex::{EducationalMutex, MutexObservation};

#[test]
fn protects_shared_state_across_threads() {
    let counter = Arc::new(EducationalMutex::new(0usize));
    let mut workers = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        workers.push(thread::spawn(move || {
            for _ in 0..250 {
                counter.with_lock(|value| *value += 1).unwrap();
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    assert_eq!(counter.with_lock(|value| *value).unwrap(), 1_000);
    assert_eq!(counter.observations().lock_attempts, 1_001);
}

#[test]
fn lock_is_released_when_guard_scope_ends() {
    let shared = EducationalMutex::new(Vec::new());

    shared.with_lock(|items| items.push("first")).unwrap();
    shared.with_lock(|items| items.push("second")).unwrap();

    let snapshot = shared.with_lock(|items| items.clone()).unwrap();
    assert_eq!(snapshot, vec!["first", "second"]);
}

#[test]
fn poisoning_is_observable_and_recoverable() {
    let shared = Arc::new(EducationalMutex::new(41usize));
    let poisoned = Arc::clone(&shared);

    let _ = thread::spawn(move || {
        let _guard = poisoned.lock().unwrap();
        panic!("panic while holding the mutex");
    })
    .join();

    assert!(shared.is_poisoned());

    let recovered = shared.recover_with(|value| {
        *value += 1;
        *value
    });

    assert_eq!(recovered, 42);
    assert_eq!(shared.with_lock(|value| *value).unwrap(), 42);
}

#[test]
fn observations_track_contention_signals() {
    let shared = EducationalMutex::new(7usize);

    let first = shared.lock().unwrap();
    let try_result = shared.try_with_lock(|value| *value);
    assert_eq!(try_result, None);
    drop(first);

    assert_eq!(shared.try_with_lock(|value| *value), Some(7));
    assert_eq!(
        shared.observations(),
        MutexObservation {
            lock_attempts: 1,
            try_lock_attempts: 2,
            try_lock_contentions: 1,
            poison_recoveries: 0,
        },
    );
}
