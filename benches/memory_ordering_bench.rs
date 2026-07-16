use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn run_fetch_add(
    ordering: Ordering,
    iterations: usize,
    worker_count: usize,
) -> (usize, std::time::Duration) {
    let counter = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut workers = Vec::new();

    for _ in 0..worker_count {
        let counter = Arc::clone(&counter);
        workers.push(thread::spawn(move || {
            for _ in 0..iterations {
                counter.fetch_add(1, ordering);
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    (counter.load(Ordering::Relaxed), start.elapsed())
}

fn run_load(
    ordering: Ordering,
    iterations: usize,
    worker_count: usize,
) -> (usize, std::time::Duration) {
    let value = Arc::new(AtomicUsize::new(1));
    let start = Instant::now();
    let mut workers = Vec::new();

    for _ in 0..worker_count {
        let value = Arc::clone(&value);
        workers.push(thread::spawn(move || {
            let mut local = 0usize;
            for _ in 0..iterations {
                local = black_box(local + value.load(ordering));
            }
            local
        }));
    }

    let total = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .sum();
    (total, start.elapsed())
}

fn main() {
    let iterations = 50_000usize;
    let worker_count = 4usize;

    let (relaxed_total, relaxed_add) = run_fetch_add(Ordering::Relaxed, iterations, worker_count);
    let (acqrel_total, acqrel_add) = run_fetch_add(Ordering::AcqRel, iterations, worker_count);
    let (seqcst_total, seqcst_add) = run_fetch_add(Ordering::SeqCst, iterations, worker_count);

    let (relaxed_load_total, relaxed_load) = run_load(Ordering::Relaxed, iterations, worker_count);
    let (acquire_load_total, acquire_load) = run_load(Ordering::Acquire, iterations, worker_count);
    let (seqcst_load_total, seqcst_load) = run_load(Ordering::SeqCst, iterations, worker_count);

    println!("memory ordering benchmark (manual, std::time::Instant)");
    println!("workers: {worker_count}");
    println!("iterations per worker: {iterations}");
    println!("fetch_add Relaxed: {relaxed_add:?} ({relaxed_total})");
    println!("fetch_add AcqRel: {acqrel_add:?} ({acqrel_total})");
    println!("fetch_add SeqCst: {seqcst_add:?} ({seqcst_total})");
    println!("load Relaxed: {relaxed_load:?} ({relaxed_load_total})");
    println!("load Acquire: {acquire_load:?} ({acquire_load_total})");
    println!("load SeqCst: {seqcst_load:?} ({seqcst_load_total})");
}
