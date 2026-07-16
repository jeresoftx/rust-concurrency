use rust_concurrency::deadlocks::{LockOrderTracker, LockRank};

fn main() {
    let mut tracker = LockOrderTracker::new();
    let low = LockRank::new(10, "low");
    let high = LockRank::new(20, "high");

    tracker.enter(low).unwrap();
    tracker.enter(high).unwrap();

    println!("locks sostenidos: {:?}", tracker.held_ranks());
}
