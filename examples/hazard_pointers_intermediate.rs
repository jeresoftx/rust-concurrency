use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, ParticipantId, RetiredNode};

fn main() {
    let domain = HazardDomain::new(4);
    let participant = ParticipantId::new(1);
    let node = NodeId::new(20);

    domain.protect(participant, node);
    domain.retire(RetiredNode::new(node, "payload"));

    println!("primer scan: {:?}", domain.scan());
    domain.clear(participant);
    println!("segundo scan: {:?}", domain.scan());
}
