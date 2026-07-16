use rust_concurrency::channels::{bounded_channel, TrySendFailure};

fn main() {
    let (producer, consumer) = bounded_channel(1);

    producer.try_send("queued").unwrap();

    match producer.try_send("delayed") {
        Ok(()) => println!("mensaje aceptado"),
        Err(TrySendFailure::Full(message)) => println!("sin capacidad: {message}"),
        Err(TrySendFailure::Closed(message)) => println!("canal cerrado: {message}"),
    }

    println!("recibido: {}", consumer.recv().unwrap());
}
