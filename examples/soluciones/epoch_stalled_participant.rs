use rust_concurrency::epoch_gc::{EpochDomain, ObjectId, ParticipantId};

fn main() {
    let domain = EpochDomain::new(2);
    let participant = ParticipantId::new(1);
    let object = ObjectId::new(9);

    domain.pin(participant);
    domain.retire(object, "retenido por lector antiguo");

    assert!(domain.try_advance().advanced());
    let blocked = domain.try_advance();
    assert!(!blocked.advanced());
    assert_eq!(blocked.blocked_by(), &[participant]);

    let scan = domain.scan();
    assert_eq!(scan.delayed(), &[object]);
    assert_eq!(scan.blocked_by(), &[participant]);

    domain.unpin(participant);
    assert!(domain.try_advance().advanced());
    assert_eq!(domain.scan().reclaimed(), &[object]);
}
