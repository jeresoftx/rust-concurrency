use std::thread;

use rust_concurrency::arc::Shared;

fn main() {
    let catalog = Shared::new(vec![
        "mutex".to_string(),
        "rwlock".to_string(),
        "atomics".to_string(),
        "arc".to_string(),
    ]);
    let mut readers = Vec::new();

    for _ in 0..4 {
        let catalog = catalog.clone_shared();
        readers.push(thread::spawn(move || catalog.with_ref(|items| items.len())));
    }

    for reader in readers {
        println!("capítulos visibles: {}", reader.join().unwrap());
    }
}
