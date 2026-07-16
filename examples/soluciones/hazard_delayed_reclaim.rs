use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, ParticipantId, RetiredNode};

fn main() {
    let domain = HazardDomain::new(4);
    let participant = ParticipantId::new(2);
    let node = NodeId::new(9);

    domain.protect(participant, node);
    domain.retire(RetiredNode::new(node, "still read"));

    assert_eq!(domain.scan().delayed, vec![node]);
    domain.clear(participant);
    assert_eq!(domain.scan().reclaimed, vec![node]);
}
