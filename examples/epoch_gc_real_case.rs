use rust_concurrency::epoch_gc::{EpochDomain, ObjectId, ParticipantId};

fn main() {
    let domain = EpochDomain::new(2);
    let reader = ParticipantId::new(1);
    let writer = ParticipantId::new(2);

    let read_epoch = domain.pin(reader);
    println!("lector entra en época {}", read_epoch.epoch());

    let write_epoch = domain.pin(writer);
    println!("escritor inspecciona head en época {}", write_epoch.epoch());
    domain.unpin(writer);

    domain.retire(ObjectId::new(100), "head removido de una pila lock-free");
    domain.try_advance();

    let blocked = domain.scan();
    println!("reclamación con lector activo: {:?}", blocked.reclaimed());
    println!("retenido por lector: {:?}", blocked.blocked_by());

    domain.unpin(reader);
    domain.try_advance();

    let reclaimed = domain.scan();
    println!("reclamación final: {:?}", reclaimed.reclaimed());
}
