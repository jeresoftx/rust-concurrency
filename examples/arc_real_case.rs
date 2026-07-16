use std::sync::Mutex;
use std::thread;

use rust_concurrency::arc::Shared;

fn main() {
    let events = Shared::new(Mutex::new(Vec::new()));
    let mut workers = Vec::new();

    for event in ["created", "validated", "confirmed"] {
        let events = events.clone_shared();
        workers.push(thread::spawn(move || {
            events.with_ref(|items| items.lock().unwrap().push(event.to_string()));
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    let mut snapshot = events.with_ref(|items| items.lock().unwrap().clone());
    snapshot.sort();

    println!("eventos: {}", snapshot.join(", "));
}
