use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, ParticipantId};

fn main() {
    let domain = HazardDomain::new(4);
    let participant = ParticipantId::new(1);
    let node = NodeId::new(42);

    domain.protect(participant, node);

    assert!(domain.is_protected(node));
    assert_eq!(domain.protected_nodes(), vec![node]);
}
