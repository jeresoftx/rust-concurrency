use rust_concurrency::epoch_gc::{EpochDomain, ParticipantId};

fn main() {
    let domain = EpochDomain::new(2);
    let participant = ParticipantId::new(1);

    let pin = domain.pin(participant);
    println!(
        "participante {} fijado en época {}",
        participant.get(),
        pin.epoch()
    );

    domain.unpin(participant);
    println!("¿sigue fijado?: {}", domain.is_pinned(participant));
}
