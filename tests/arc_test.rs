use std::sync::Mutex;
use std::thread;

use rust_concurrency::arc::{ArcObservation, Shared, SharedWeak};

#[test]
fn clone_counts_track_shared_ownership() {
    let shared = Shared::new(String::from("academy"));

    assert_eq!(
        shared.observations(),
        ArcObservation {
            strong_count: 1,
            weak_count: 0,
        },
    );

    let cloned = shared.clone_shared();

    assert_eq!(shared.strong_count(), 2);
    assert_eq!(cloned.strong_count(), 2);
    drop(cloned);
    assert_eq!(shared.strong_count(), 1);
}

#[test]
fn weak_upgrade_fails_after_last_strong_drop() {
    let weak: SharedWeak<String>;

    {
        let shared = Shared::new(String::from("temporal"));
        weak = shared.downgrade();
        assert!(weak.upgrade().is_some());
        assert_eq!(shared.weak_count(), 1);
    }

    assert!(weak.upgrade().is_none());
}

#[test]
fn shared_immutable_data_can_cross_threads() {
    let shared = Shared::new(vec![1, 2, 3, 4]);
    let mut workers = Vec::new();

    for _ in 0..4 {
        let shared = shared.clone_shared();
        workers.push(thread::spawn(move || {
            shared.with_ref(|items| items.iter().sum::<i32>())
        }));
    }

    for worker in workers {
        assert_eq!(worker.join().unwrap(), 10);
    }
}

#[test]
fn shared_mutability_requires_interior_lock() {
    let shared = Shared::new(Mutex::new(Vec::new()));
    let mut workers = Vec::new();

    for item in ["mutex", "rwlock", "atomics"] {
        let shared = shared.clone_shared();
        workers.push(thread::spawn(move || {
            shared.with_ref(|items| items.lock().unwrap().push(item.to_string()));
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    let mut snapshot = shared.with_ref(|items| items.lock().unwrap().clone());
    snapshot.sort();

    assert_eq!(snapshot, vec!["atomics", "mutex", "rwlock"]);
}

#[test]
fn strong_and_weak_observations_are_available_from_weak_handle() {
    let shared = Shared::new(42usize);
    let weak = shared.downgrade();

    assert_eq!(
        weak.observations(),
        ArcObservation {
            strong_count: 1,
            weak_count: 1,
        },
    );

    drop(shared);

    assert_eq!(
        weak.observations(),
        ArcObservation {
            strong_count: 0,
            weak_count: 0,
        },
    );
}
