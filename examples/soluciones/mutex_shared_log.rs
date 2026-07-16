use std::sync::Arc;
use std::thread;

use rust_concurrency::mutex::EducationalMutex;

fn main() {
    let log = Arc::new(EducationalMutex::new(Vec::new()));
    let mut workers = Vec::new();

    for worker_id in 0..4 {
        let log = Arc::clone(&log);
        workers.push(thread::spawn(move || {
            log.with_lock(|entries| entries.push(format!("worker-{worker_id}")))
                .unwrap();
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    let mut entries = log.with_lock(|entries| entries.clone()).unwrap();
    entries.sort();

    assert_eq!(entries, ["worker-0", "worker-1", "worker-2", "worker-3"]);
    println!("{entries:?}");
}
