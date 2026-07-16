use rust_concurrency::mutex::EducationalMutex;

fn main() {
    let counter = EducationalMutex::new(0);

    counter.with_lock(|value| *value += 1).unwrap();
    counter.with_lock(|value| *value += 2).unwrap();
    counter.with_lock(|value| *value += 3).unwrap();

    let final_value = counter.with_lock(|value| *value).unwrap();

    assert_eq!(final_value, 6);
    assert_eq!(counter.observations().lock_attempts, 4);
    println!("valor final: {final_value}");
}
