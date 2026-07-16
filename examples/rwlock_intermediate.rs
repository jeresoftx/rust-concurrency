use std::sync::Arc;
use std::thread;

use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let config = Arc::new(EducationalRwLock::new(String::from("modo=lectura")));
    let mut readers = Vec::new();

    for id in 0..3 {
        let config = Arc::clone(&config);
        readers.push(thread::spawn(move || {
            let value = config.with_read(|text| text.clone()).unwrap();
            println!("lector {id}: {value}");
        }));
    }

    for reader in readers {
        reader.join().unwrap();
    }

    config
        .with_write(|text| *text = String::from("modo=escritura"))
        .unwrap();
    println!(
        "configuración final: {}",
        config.with_read(|text| text.clone()).unwrap()
    );
}
