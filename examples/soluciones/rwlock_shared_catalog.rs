use std::sync::Arc;
use std::thread;

use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let catalog = Arc::new(EducationalRwLock::new(vec!["mutex".to_string()]));

    catalog
        .with_write(|items| items.push("rwlock".to_string()))
        .unwrap();

    let mut readers = Vec::new();
    for _ in 0..3 {
        let catalog = Arc::clone(&catalog);
        readers.push(thread::spawn(move || {
            catalog.with_read(|items| items.join(", ")).unwrap()
        }));
    }

    for reader in readers {
        println!("{}", reader.join().unwrap());
    }
}
