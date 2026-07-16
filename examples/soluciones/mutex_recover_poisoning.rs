use std::sync::Arc;
use std::thread;

use rust_concurrency::mutex::EducationalMutex;

fn main() {
    let shared = Arc::new(EducationalMutex::new(vec!["inicio".to_string()]));
    let poisoned = Arc::clone(&shared);

    let _ = thread::spawn(move || {
        let mut guard = poisoned.lock().unwrap();
        guard.push("incompleto".to_string());
        panic!("fallo antes de terminar la actualización");
    })
    .join();

    assert!(shared.is_poisoned());

    let repaired_len = shared.recover_with(|entries| {
        entries.retain(|entry| entry != "incompleto");
        entries.push("reparado".to_string());
        entries.len()
    });

    assert_eq!(repaired_len, 2);
    assert!(!shared.is_poisoned());
    println!("estado reparado con {repaired_len} entradas");
}
