use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, RetiredNode};

fn main() {
    let domain = HazardDomain::new(2);

    assert!(domain
        .retire(RetiredNode::new(NodeId::new(1), "a"))
        .is_none());
    let report = domain
        .retire(RetiredNode::new(NodeId::new(2), "b"))
        .unwrap();

    assert_eq!(report.reclaimed, vec![NodeId::new(1), NodeId::new(2)]);
}
