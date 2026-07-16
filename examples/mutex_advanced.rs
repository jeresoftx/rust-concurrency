use std::sync::Arc;
use std::thread;

use rust_concurrency::mutex::EducationalMutex;

fn main() {
    let shared = Arc::new(EducationalMutex::new(String::from("estado parcial")));
    let poisoned = Arc::clone(&shared);

    let _ = thread::spawn(move || {
        let mut guard = poisoned.lock().unwrap();
        guard.push_str(" escrito antes del pánico");
        panic!("simulamos un fallo dentro de la sección crítica");
    })
    .join();

    if shared.is_poisoned() {
        let repaired = shared.recover_with(|value| {
            value.clear();
            value.push_str("estado reparado");
            value.clone()
        });
        println!("recuperado: {repaired}");
    }

    println!("poisoned después de recuperar: {}", shared.is_poisoned());
}
