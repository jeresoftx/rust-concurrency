use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, ParticipantId, RetiredNode};

fn main() {
    let domain = HazardDomain::new(3);
    let reader = ParticipantId::new(1);

    for id in 0..3 {
        let node = NodeId::new(id);
        if id == 1 {
            domain.protect(reader, node);
        }
        domain.retire(RetiredNode::new(node, format!("node-{id}")));
    }

    println!("retirados después del umbral: {:?}", domain.retired_nodes());
    domain.clear(reader);
    println!("reclamados finales: {:?}", domain.scan().reclaimed);
}
