use std::sync::Arc;
use std::thread;

use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let shared = Arc::new(EducationalRwLock::new(40usize));
    let poisoned = Arc::clone(&shared);

    let _ = thread::spawn(move || {
        let mut guard = poisoned.write().unwrap();
        *guard += 1;
        panic!("pánico con lock de escritura");
    })
    .join();

    assert!(shared.is_poisoned());

    let recovered = shared.recover_write_with(|value| {
        *value += 1;
        *value
    });

    assert_eq!(recovered, 42);
    assert!(!shared.is_poisoned());
    println!("valor recuperado: {recovered}");
}
