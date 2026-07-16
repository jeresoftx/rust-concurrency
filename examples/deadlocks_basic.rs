use rust_concurrency::deadlocks::{LockOrderTracker, LockRank};

fn main() {
    let mut tracker = LockOrderTracker::new();
    let account = LockRank::new(10, "account");
    let ledger = LockRank::new(20, "ledger");

    tracker.enter(account).unwrap();
    tracker.enter(ledger).unwrap();

    println!("orden válido: {:?}", tracker.held_ranks());
}
