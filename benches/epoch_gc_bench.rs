use std::hint::black_box;
use std::time::Instant;

use rust_concurrency::epoch_gc::{EpochDomain, ObjectId, ParticipantId};

fn main() {
    let iterations = 50_000usize;
    let participant = ParticipantId::new(1);
    let domain = EpochDomain::new(2);

    let start = Instant::now();
    for _ in 0..iterations {
        let pin = domain.pin(participant);
        black_box(pin.epoch());
        domain.unpin(participant);
    }
    let pin_unpin_elapsed = start.elapsed();

    let domain = EpochDomain::new(iterations + 1);
    let start = Instant::now();
    for id in 0..iterations {
        black_box(domain.retire(ObjectId::new(id), "retired"));
    }
    let retire_elapsed = start.elapsed();

    let domain = EpochDomain::new(2);
    let stalled = ParticipantId::new(99);
    domain.pin(stalled);
    for id in 0..iterations {
        domain.retire(ObjectId::new(id), "blocked");
    }
    domain.try_advance();
    let start = Instant::now();
    let blocked_scan = domain.scan();
    black_box(blocked_scan.delayed().len());
    let blocked_scan_elapsed = start.elapsed();

    domain.unpin(stalled);
    domain.try_advance();
    let start = Instant::now();
    let reclaim_scan = domain.scan();
    black_box(reclaim_scan.reclaimed().len());
    let reclaim_scan_elapsed = start.elapsed();

    println!("epoch gc benchmark (manual, std::time::Instant)");
    println!("iterations: {iterations}");
    println!("pin/unpin: {pin_unpin_elapsed:?}");
    println!("rendimiento de retiro: {retire_elapsed:?}");
    println!(
        "escaneo bloqueado retrasó {} objetos: {blocked_scan_elapsed:?}",
        iterations
    );
    println!(
        "escaneo de reclamación liberó {} objetos: {reclaim_scan_elapsed:?}",
        iterations
    );
}
