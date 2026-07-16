use rust_concurrency::atomics::CompareExchange;

fn main() {
    let slot = CompareExchange::new(41);

    match slot.compare_exchange(41, 42) {
        Ok(previous) => println!("cambio aplicado desde {previous}"),
        Err(observed) => println!("otro hilo dejó {observed}"),
    }

    println!("valor final: {}", slot.load());
}
