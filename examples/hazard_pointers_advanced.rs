use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, RetiredNode};

fn main() {
    let domain = HazardDomain::new(2);

    let first = domain.retire(RetiredNode::new(NodeId::new(1), "a"));
    let second = domain.retire(RetiredNode::new(NodeId::new(2), "b"));

    println!("primer retiro: {first:?}");
    println!("segundo retiro: {second:?}");
}
