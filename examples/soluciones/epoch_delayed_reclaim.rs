use rust_concurrency::epoch_gc::{EpochDomain, ObjectId};

fn main() {
    let domain = EpochDomain::new(2);
    let object = ObjectId::new(1);

    domain.retire(object, "objeto retirado");

    let early = domain.scan();
    assert_eq!(early.delayed(), &[object]);
    assert!(early.reclaimed().is_empty());

    assert!(domain.try_advance().advanced());
    assert!(domain.try_advance().advanced());

    let mature = domain.scan();
    assert_eq!(mature.reclaimed(), &[object]);
}
