use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let flags = Arc::new(EducationalRwLock::new(HashMap::from([
        ("checkout_v2".to_string(), true),
        ("legacy_export".to_string(), false),
    ])));

    let mut readers = Vec::new();
    for name in ["checkout_v2", "legacy_export", "checkout_v2"] {
        let flags = Arc::clone(&flags);
        readers.push(thread::spawn(move || {
            flags
                .with_read(|values| {
                    println!("{name}: {}", values.get(name).copied().unwrap_or(false))
                })
                .unwrap();
        }));
    }

    for reader in readers {
        reader.join().unwrap();
    }

    flags
        .with_write(|values| {
            values.insert("legacy_export".to_string(), true);
        })
        .unwrap();

    println!(
        "legacy_export actualizado: {}",
        flags
            .with_read(|values| values.get("legacy_export").copied().unwrap_or(false))
            .unwrap()
    );
}
