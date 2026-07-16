use rust_concurrency::atomics::AtomicCounter;

fn main() {
    let counter = AtomicCounter::new(10);

    let previous = counter.fetch_add(5);
    let final_value = counter.load();

    println!("valor anterior: {previous}");
    println!("valor final: {final_value}");
    println!("fetch_adds: {}", counter.observations().fetch_adds);
}
