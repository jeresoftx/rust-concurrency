use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let shared = EducationalRwLock::new(vec![1, 2, 3]);

    let len = shared.with_read(|items| items.len()).unwrap();
    let sum = shared.with_read(|items| items.iter().sum::<i32>()).unwrap();
    let observations = shared.observations();

    println!("longitud: {len}");
    println!("suma: {sum}");
    println!("intentos de lectura: {}", observations.read_attempts);
}
