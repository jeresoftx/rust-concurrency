use std::sync::Arc;
use std::thread;

use rust_concurrency::mutex::EducationalMutex;

#[derive(Debug, Default)]
struct ReservationMetrics {
    confirmed: usize,
    rejected: usize,
}

fn main() {
    let metrics = Arc::new(EducationalMutex::new(ReservationMetrics::default()));
    let mut workers = Vec::new();

    for worker_id in 0..6 {
        let metrics = Arc::clone(&metrics);
        workers.push(thread::spawn(move || {
            for request_id in 0..100 {
                let accepted = (worker_id + request_id) % 4 != 0;
                metrics
                    .with_lock(|metrics| {
                        if accepted {
                            metrics.confirmed += 1;
                        } else {
                            metrics.rejected += 1;
                        }
                    })
                    .unwrap();
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    let snapshot = metrics
        .with_lock(|metrics| (metrics.confirmed, metrics.rejected))
        .unwrap();
    println!("reservas confirmadas: {}", snapshot.0);
    println!("reservas rechazadas: {}", snapshot.1);
}
