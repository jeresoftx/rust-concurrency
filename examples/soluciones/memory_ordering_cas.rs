use std::sync::atomic::Ordering;

use rust_concurrency::memory_ordering::OrderingCasCell;

fn main() {
    let cell = OrderingCasCell::new(1);

    assert_eq!(
        cell.compare_exchange(1, 2, Ordering::AcqRel, Ordering::Acquire),
        Ok(1)
    );
    assert_eq!(
        cell.compare_exchange(1, 3, Ordering::AcqRel, Ordering::Acquire),
        Err(2)
    );

    println!("{:?}", cell.observation());
}
