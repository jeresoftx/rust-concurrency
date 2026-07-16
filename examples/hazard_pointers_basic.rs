use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, ParticipantId};

fn main() {
    let domain = HazardDomain::new(4);
    let participant = ParticipantId::new(1);
    let node = NodeId::new(10);

    domain.protect(participant, node);

    println!("protegido: {}", domain.is_protected(node));
}
