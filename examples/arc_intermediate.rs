use std::thread;

use rust_concurrency::arc::Shared;

fn main() {
    let chapters = Shared::new(vec!["mutex", "rwlock", "atomics", "arc"]);
    let mut workers = Vec::new();

    for _ in 0..3 {
        let chapters = chapters.clone_shared();
        workers.push(thread::spawn(move || {
            chapters.with_ref(|items| items.join(", "))
        }));
    }

    for worker in workers {
        println!("{}", worker.join().unwrap());
    }
}
