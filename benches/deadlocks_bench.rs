use std::hint::black_box;
use std::time::Instant;

use rust_concurrency::deadlocks::{BankAccounts, LockOrderTracker, LockRank};

fn main() {
    let iterations = 50_000usize;

    let low = LockRank::new(10, "low");
    let high = LockRank::new(20, "high");

    let start = Instant::now();
    for _ in 0..iterations {
        let mut tracker = LockOrderTracker::new();
        tracker.enter(low).unwrap();
        tracker.enter(high).unwrap();
        black_box(tracker.held_ranks());
    }
    let validation_elapsed = start.elapsed();

    let accounts = BankAccounts::new([1_000_000, 0]);
    let start = Instant::now();
    for _ in 0..iterations {
        accounts.transfer_ordered(0, 1, 1).unwrap();
        accounts.transfer_ordered(1, 0, 1).unwrap();
    }
    let ordered_transfer_elapsed = start.elapsed();

    println!("deadlocks benchmark (manual, std::time::Instant)");
    println!("iterations: {iterations}");
    println!("lock-order validation: {validation_elapsed:?}");
    println!("ordered transfer prevention: {ordered_transfer_elapsed:?}");
    println!(
        "final balances: {}, {}",
        accounts.balance(0),
        accounts.balance(1)
    );
}
