use std::collections::BTreeSet;
use std::sync::Arc;
use std::thread;

use rust_concurrency::lock_free::{
    AbaScenario, BoundedLockFreeStack, ProgressGuarantee, StackError,
};

#[test]
fn stack_push_pop_is_lifo() {
    let stack = BoundedLockFreeStack::new(4);

    stack.push(10).unwrap();
    stack.push(20).unwrap();
    stack.push(30).unwrap();

    assert_eq!(stack.pop(), Some(30));
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), None);
}

#[test]
fn stack_reports_full_capacity() {
    let stack = BoundedLockFreeStack::new(1);

    stack.push(7).unwrap();

    assert_eq!(stack.push(8), Err(StackError::Full(8)));
}

#[test]
fn stack_tracks_progress_guarantee_and_observations() {
    let stack = BoundedLockFreeStack::new(2);

    assert_eq!(stack.progress_guarantee(), ProgressGuarantee::LockFree);
    stack.push(1).unwrap();
    stack.push(2).unwrap();
    assert_eq!(stack.pop(), Some(2));

    let observations = stack.observations();
    assert!(observations.cas_attempts >= 3);
    assert!(observations.cas_successes >= 3);
}

#[test]
fn concurrent_push_and_pop_preserves_all_values() {
    let stack = Arc::new(BoundedLockFreeStack::new(64));
    let mut producers = Vec::new();

    for worker in 0..4 {
        let stack = Arc::clone(&stack);
        producers.push(thread::spawn(move || {
            for offset in 0..8 {
                stack.push(worker * 10 + offset).unwrap();
            }
        }));
    }

    for producer in producers {
        producer.join().unwrap();
    }

    let mut values = BTreeSet::new();
    while let Some(value) = stack.pop() {
        values.insert(value);
    }

    let expected = (0..4)
        .flat_map(|worker| (0..8).map(move |offset| worker * 10 + offset))
        .collect::<BTreeSet<_>>();

    assert_eq!(values, expected);
}

#[test]
fn aba_scenario_describes_reused_head_index() {
    let scenario = AbaScenario::new(3, 1, 3);

    assert!(scenario.is_aba_risk());
    assert_eq!(
        scenario.description(),
        "head cambió de 3 a 1 y regresó a 3; el índice observado parece igual, pero la historia cambió"
    );
}
