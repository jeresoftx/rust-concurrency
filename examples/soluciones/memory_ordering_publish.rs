use rust_concurrency::memory_ordering::PublishedValue;

fn main() {
    let value = PublishedValue::new(10);

    assert_eq!(value.try_read(), None);
    value.publish(99);
    assert_eq!(value.try_read(), Some(99));

    println!("lecturas observadas: {}", value.observed_reads());
}
