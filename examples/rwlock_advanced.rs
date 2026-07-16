use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let shared = EducationalRwLock::new(vec![10, 20, 30]);
    let read_guard = shared.read().unwrap();

    let write_now = shared.try_with_write(|items| items.push(40));
    assert_eq!(write_now, None);
    println!("un escritor no entra mientras exista un lector activo");

    drop(read_guard);

    let write_later = shared.try_with_write(|items| {
        items.push(40);
        items.len()
    });

    println!("escritura posterior: {write_later:?}");
    println!("observaciones: {:?}", shared.observations());
}
