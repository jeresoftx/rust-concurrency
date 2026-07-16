use rust_concurrency::epoch_gc::{EpochDomain, ObjectId};

fn main() {
    let domain = EpochDomain::new(2);
    let object = ObjectId::new(7);

    domain.retire(object, "versión vieja");
    println!("retirados antes de avanzar: {:?}", domain.retired_objects());

    let early = domain.scan();
    println!("reclamados temprano: {:?}", early.reclaimed());
    println!("retrasados temprano: {:?}", early.delayed());

    domain.try_advance();
    domain.try_advance();

    let mature = domain.scan();
    println!("reclamados después del rezago: {:?}", mature.reclaimed());
}
