use rust_concurrency::epoch_gc::{EpochDomain, ObjectId, ParticipantId};

fn main() {
    let domain = EpochDomain::new(2);
    let participant = ParticipantId::new(1);

    domain.pin(participant);
    domain.retire(ObjectId::new(11), "nodo retirado por CAS");

    let first = domain.try_advance();
    println!("primer avance: {}", first.advanced());

    let second = domain.try_advance();
    println!("segundo avance: {}", second.advanced());
    println!("bloqueado por: {:?}", second.blocked_by());

    let blocked_scan = domain.scan();
    println!("memoria retenida: {:?}", blocked_scan.delayed());

    domain.unpin(participant);
    domain.try_advance();

    let final_scan = domain.scan();
    println!(
        "reclamado al llegar a quiescencia: {:?}",
        final_scan.reclaimed()
    );
}
