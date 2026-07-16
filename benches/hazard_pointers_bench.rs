use std::hint::black_box;
use std::time::Instant;

use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, ParticipantId, RetiredNode};

fn main() {
    let iterations = 50_000usize;
    let domain = HazardDomain::new(iterations + 1);
    let participant = ParticipantId::new(1);

    let start = Instant::now();
    for id in 0..iterations {
        let node = NodeId::new(id);
        domain.protect(participant, node);
        black_box(domain.is_protected(node));
        domain.clear(participant);
    }
    let protect_clear_elapsed = start.elapsed();

    let domain = HazardDomain::new(iterations + 1);
    for id in 0..iterations {
        domain.retire(RetiredNode::new(NodeId::new(id), "retired"));
    }
    let start = Instant::now();
    let report = domain.scan();
    black_box(report.reclaimed.len());
    let scan_elapsed = start.elapsed();

    let domain = HazardDomain::new(64);
    let start = Instant::now();
    for id in 0..iterations {
        let _ = domain.retire(RetiredNode::new(NodeId::new(id), "threshold"));
    }
    let threshold_elapsed = start.elapsed();

    println!("hazard pointers benchmark (manual, std::time::Instant)");
    println!("iterations: {iterations}");
    println!("protect/clear: {protect_clear_elapsed:?}");
    println!("single scan reclaim: {scan_elapsed:?}");
    println!("threshold retire+scan: {threshold_elapsed:?}");
}
