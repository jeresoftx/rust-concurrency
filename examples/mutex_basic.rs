use rust_concurrency::mutex::EducationalMutex;

fn main() {
    let counter = EducationalMutex::new(0);

    counter.with_lock(|value| *value += 1).unwrap();
    counter.with_lock(|value| *value += 1).unwrap();

    let final_value = counter.with_lock(|value| *value).unwrap();
    println!("contador final: {final_value}");
}
