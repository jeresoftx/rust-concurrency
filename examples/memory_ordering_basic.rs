use rust_concurrency::memory_ordering::PublishedValue;

fn main() {
    let value = PublishedValue::new(0);

    println!("antes de publicar: {:?}", value.try_read());
    value.publish(42);
    println!("después de publicar: {:?}", value.try_read());
}
