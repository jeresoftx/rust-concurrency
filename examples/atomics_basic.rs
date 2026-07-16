use rust_concurrency::atomics::AtomicCounter;

fn main() {
    let counter = AtomicCounter::new(0);

    counter.fetch_add(1);
    counter.fetch_add(2);

    println!("contador: {}", counter.load());
    println!("observaciones: {:?}", counter.observations());
}
