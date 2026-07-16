use rust_concurrency::hazard_pointers::{HazardDomain, NodeId, ParticipantId, RetiredNode};

#[test]
fn protecting_a_node_makes_it_visible_to_scans() {
    let domain = HazardDomain::new(4);
    let participant = ParticipantId::new(1);
    let node = NodeId::new(42);

    domain.protect(participant, node);

    assert!(domain.is_protected(node));
    assert_eq!(domain.protected_nodes(), vec![node]);
}

#[test]
fn retired_unprotected_node_is_reclaimed_on_scan() {
    let domain = HazardDomain::new(4);
    let node = NodeId::new(7);

    domain.retire(RetiredNode::new(node, "payload"));
    let report = domain.scan();

    assert_eq!(report.scanned_hazards, 0);
    assert_eq!(report.reclaimed, vec![node]);
    assert!(domain.retired_nodes().is_empty());
    assert_eq!(domain.reclaimed_nodes(), vec![node]);
}

#[test]
fn protected_retired_node_is_delayed_until_unprotected() {
    let domain = HazardDomain::new(4);
    let participant = ParticipantId::new(2);
    let node = NodeId::new(9);

    domain.protect(participant, node);
    domain.retire(RetiredNode::new(node, "still read"));

    let first = domain.scan();
    assert_eq!(first.delayed, vec![node]);
    assert_eq!(domain.retired_nodes(), vec![node]);

    domain.clear(participant);

    let second = domain.scan();
    assert_eq!(second.reclaimed, vec![node]);
    assert!(domain.retired_nodes().is_empty());
}

#[test]
fn retire_threshold_triggers_scan() {
    let domain = HazardDomain::new(2);

    let first = domain.retire(RetiredNode::new(NodeId::new(1), "a"));
    let second = domain.retire(RetiredNode::new(NodeId::new(2), "b"));

    assert!(first.is_none());
    assert_eq!(
        second.expect("threshold should scan").reclaimed,
        vec![NodeId::new(1), NodeId::new(2)]
    );
}

#[test]
fn per_thread_records_are_independent() {
    let domain = HazardDomain::new(8);
    let a = ParticipantId::new(1);
    let b = ParticipantId::new(2);
    let node_a = NodeId::new(10);
    let node_b = NodeId::new(20);

    domain.protect(a, node_a);
    domain.protect(b, node_b);
    domain.clear(a);

    assert!(!domain.is_protected(node_a));
    assert!(domain.is_protected(node_b));
    assert_eq!(domain.protected_nodes(), vec![node_b]);
}
