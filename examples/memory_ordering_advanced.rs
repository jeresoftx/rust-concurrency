use std::sync::atomic::Ordering;

use rust_concurrency::memory_ordering::OrderingCasCell;

fn main() {
    let cell = OrderingCasCell::new(7);

    let first = cell.compare_exchange(7, 9, Ordering::AcqRel, Ordering::Acquire);
    let second = cell.compare_exchange(7, 11, Ordering::AcqRel, Ordering::Acquire);

    println!("primer CAS: {first:?}");
    println!("segundo CAS: {second:?}");
    println!("observación: {:?}", cell.observation());
}
