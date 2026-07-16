use rust_concurrency::epoch_gc::{EpochDomain, ParticipantId};

fn main() {
    let domain = EpochDomain::new(2);
    let participant = ParticipantId::new(1);

    domain.pin(participant);
    assert!(domain.is_pinned(participant));

    domain.unpin(participant);
    assert!(!domain.is_pinned(participant));
}
