use rust_concurrency::deadlocks::{LockOrderTracker, LockRank};

fn main() {
    let mut tracker = LockOrderTracker::new();
    let low = LockRank::new(10, "low");
    let high = LockRank::new(20, "high");

    tracker.enter(high).unwrap();

    match tracker.enter(low) {
        Ok(()) => println!("orden aceptado"),
        Err(error) => println!(
            "violación: sostenía {} y pidió {}",
            error.held.name(),
            error.requested.name()
        ),
    }
}
