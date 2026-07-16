use rust_concurrency::epoch_gc::{EpochDomain, ObjectId, ParticipantId};

#[test]
fn pin_records_the_participant_epoch() {
    let domain = EpochDomain::new(2);
    let participant = ParticipantId::new(1);

    let pin = domain.pin(participant);

    assert_eq!(pin.participant(), participant);
    assert_eq!(pin.epoch(), 0);
    assert_eq!(domain.global_epoch(), 0);
    assert!(domain.is_pinned(participant));
    assert_eq!(domain.pinned_participants(), vec![participant]);
}

#[test]
fn unpin_marks_quiescence_and_allows_epoch_advancement() {
    let domain = EpochDomain::new(2);
    let participant = ParticipantId::new(1);

    domain.pin(participant);
    let first_advance = domain.try_advance();
    assert!(first_advance.advanced());
    assert_eq!(first_advance.previous_epoch(), 0);
    assert_eq!(first_advance.current_epoch(), 1);

    let blocked_advance = domain.try_advance();
    assert!(!blocked_advance.advanced());
    assert_eq!(blocked_advance.blocked_by(), &[participant]);

    domain.unpin(participant);
    let second_advance = domain.try_advance();
    assert!(second_advance.advanced());
    assert_eq!(second_advance.current_epoch(), 2);
}

#[test]
fn retired_object_waits_for_epoch_lag_before_reclamation() {
    let domain = EpochDomain::new(2);
    let retired = domain.retire(ObjectId::new(7), "segmento obsoleto");

    assert_eq!(retired.retired_epoch(), 0);
    assert_eq!(domain.retired_objects(), vec![ObjectId::new(7)]);

    let early_scan = domain.scan();
    assert!(early_scan.reclaimed().is_empty());
    assert_eq!(early_scan.delayed(), &[ObjectId::new(7)]);

    assert!(domain.try_advance().advanced());
    assert!(domain.try_advance().advanced());

    let mature_scan = domain.scan();
    assert_eq!(mature_scan.reclaimed(), &[ObjectId::new(7)]);
    assert!(mature_scan.delayed().is_empty());
    assert!(domain.retired_objects().is_empty());
    assert_eq!(domain.reclaimed_objects(), vec![ObjectId::new(7)]);
}

#[test]
fn stalled_participant_blocks_reclamation_until_quiescent() {
    let domain = EpochDomain::new(2);
    let participant = ParticipantId::new(1);

    domain.pin(participant);
    domain.retire(ObjectId::new(11), "nodo retirado");
    assert!(domain.try_advance().advanced());
    assert!(!domain.try_advance().advanced());

    let blocked_scan = domain.scan();
    assert!(blocked_scan.reclaimed().is_empty());
    assert_eq!(blocked_scan.delayed(), &[ObjectId::new(11)]);
    assert_eq!(blocked_scan.blocked_by(), &[participant]);

    domain.unpin(participant);
    assert!(domain.try_advance().advanced());

    let final_scan = domain.scan();
    assert_eq!(final_scan.reclaimed(), &[ObjectId::new(11)]);
    assert!(final_scan.blocked_by().is_empty());
}

#[test]
fn multiple_retired_objects_reclaim_when_each_epoch_is_safe() {
    let domain = EpochDomain::new(2);

    domain.retire(ObjectId::new(1), "a");
    assert!(domain.try_advance().advanced());
    domain.retire(ObjectId::new(2), "b");
    assert!(domain.try_advance().advanced());

    let first_scan = domain.scan();
    assert_eq!(first_scan.reclaimed(), &[ObjectId::new(1)]);
    assert_eq!(first_scan.delayed(), &[ObjectId::new(2)]);

    assert!(domain.try_advance().advanced());
    let second_scan = domain.scan();
    assert_eq!(second_scan.reclaimed(), &[ObjectId::new(2)]);
    assert!(second_scan.delayed().is_empty());
}
