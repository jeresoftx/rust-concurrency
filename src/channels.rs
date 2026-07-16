//! Channels.
//!
//! Objetivo de aprendizaje: entender comunicación por mensajes, productores,
//! consumidores, backpressure, cierre de canal y tradeoffs frente a memoria
//! compartida.

use std::sync::mpsc::{
    channel, sync_channel, Receiver, RecvError, SendError as StdSendError, Sender, SyncSender,
    TrySendError,
};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Error al enviar cuando el consumidor ya cerró.
///
/// # Examples
///
/// ```
/// use rust_concurrency::channels::{unbounded_channel, SendFailure};
///
/// let (producer, consumer) = unbounded_channel();
/// drop(consumer);
///
/// assert_eq!(producer.send(7), Err(SendFailure::Closed(7)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendFailure<T> {
    /// El consumidor ya no existe; se devuelve el mensaje original.
    Closed(T),
}

impl<T> From<StdSendError<T>> for SendFailure<T> {
    fn from(error: StdSendError<T>) -> Self {
        Self::Closed(error.0)
    }
}

/// Error de recepción bloqueante.
///
/// # Examples
///
/// ```
/// use rust_concurrency::channels::{unbounded_channel, ChannelReceiveError};
///
/// let (producer, consumer) = unbounded_channel::<i32>();
/// drop(producer);
///
/// assert_eq!(consumer.recv(), Err(ChannelReceiveError::Closed));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelReceiveError {
    /// Todos los productores fueron cerrados.
    Closed,
}

impl From<RecvError> for ChannelReceiveError {
    fn from(_: RecvError) -> Self {
        Self::Closed
    }
}

/// Error de envío no bloqueante en un canal acotado.
///
/// # Examples
///
/// ```
/// use rust_concurrency::channels::{bounded_channel, TrySendFailure};
///
/// let (producer, _consumer) = bounded_channel(1);
/// producer.try_send("first").unwrap();
///
/// assert_eq!(
///     producer.try_send("second"),
///     Err(TrySendFailure::Full("second"))
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrySendFailure<T> {
    /// El canal está lleno; se devuelve el mensaje original.
    Full(T),
    /// El consumidor ya no existe; se devuelve el mensaje original.
    Closed(T),
}

impl<T> From<TrySendError<T>> for TrySendFailure<T> {
    fn from(error: TrySendError<T>) -> Self {
        match error {
            TrySendError::Full(value) => Self::Full(value),
            TrySendError::Disconnected(value) => Self::Closed(value),
        }
    }
}

/// Productor de un canal no acotado.
pub struct Producer<T> {
    sender: Sender<T>,
}

impl<T> Clone for Producer<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<T> Producer<T> {
    /// Envía un mensaje al consumidor.
    ///
    /// Complejidad: O(1) amortizado. Progreso: puede fallar si el consumidor
    /// cerró, pero no espera por capacidad porque el canal no es acotado.
    pub fn send(&self, value: T) -> Result<(), SendFailure<T>> {
        self.sender.send(value).map_err(Into::into)
    }
}

/// Consumidor de un canal no acotado.
pub struct Consumer<T> {
    receiver: Receiver<T>,
}

impl<T> Consumer<T> {
    /// Espera hasta recibir un mensaje o hasta que todos los productores cierren.
    ///
    /// Complejidad: O(1) por mensaje. Progreso: puede bloquear mientras exista
    /// al menos un productor vivo y no haya mensajes disponibles.
    pub fn recv(&self) -> Result<T, ChannelReceiveError> {
        self.receiver.recv().map_err(Into::into)
    }
}

/// Crea un canal no acotado de múltiples productores y un consumidor.
///
/// # Examples
///
/// ```
/// use rust_concurrency::channels::unbounded_channel;
///
/// let (producer, consumer) = unbounded_channel();
/// producer.send("hola").unwrap();
/// assert_eq!(consumer.recv(), Ok("hola"));
/// ```
///
/// Complejidad: O(1). Progreso: los envíos no esperan por capacidad.
pub fn unbounded_channel<T>() -> (Producer<T>, Consumer<T>) {
    let (sender, receiver) = channel();
    (Producer { sender }, Consumer { receiver })
}

/// Productor de un canal acotado.
pub struct BoundedProducer<T> {
    sender: SyncSender<T>,
}

impl<T> Clone for BoundedProducer<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<T> BoundedProducer<T> {
    /// Envía un mensaje, esperando si el canal está lleno.
    ///
    /// Complejidad: O(1) por mensaje. Progreso: puede bloquear por backpressure
    /// si el consumidor no libera capacidad.
    pub fn send(&self, value: T) -> Result<(), SendFailure<T>> {
        self.sender.send(value).map_err(Into::into)
    }

    /// Intenta enviar sin bloquear.
    ///
    /// Complejidad: O(1). Progreso: no bloquea; devuelve `Full` si no hay
    /// capacidad disponible.
    pub fn try_send(&self, value: T) -> Result<(), TrySendFailure<T>> {
        self.sender.try_send(value).map_err(Into::into)
    }
}

/// Consumidor de un canal acotado.
pub struct BoundedConsumer<T> {
    receiver: Receiver<T>,
}

impl<T> BoundedConsumer<T> {
    /// Espera hasta recibir un mensaje o hasta que todos los productores cierren.
    ///
    /// Complejidad: O(1) por mensaje. Progreso: puede bloquear si el canal está
    /// vacío y todavía hay productores vivos.
    pub fn recv(&self) -> Result<T, ChannelReceiveError> {
        self.receiver.recv().map_err(Into::into)
    }
}

/// Crea un canal acotado con capacidad fija.
///
/// # Examples
///
/// ```
/// use rust_concurrency::channels::{bounded_channel, TrySendFailure};
///
/// let (producer, consumer) = bounded_channel(1);
/// producer.try_send(1).unwrap();
/// assert_eq!(producer.try_send(2), Err(TrySendFailure::Full(2)));
/// assert_eq!(consumer.recv(), Ok(1));
/// ```
///
/// Complejidad: O(1). Progreso: una capacidad de cero modela rendezvous; una
/// capacidad mayor permite buffering finito.
pub fn bounded_channel<T>(capacity: usize) -> (BoundedProducer<T>, BoundedConsumer<T>) {
    let (sender, receiver) = sync_channel(capacity);
    (BoundedProducer { sender }, BoundedConsumer { receiver })
}

/// Worker pool educativo basado en channels.
///
/// El pool implementa fan-out de trabajos hacia varios workers y fan-in de
/// resultados hacia un solo consumidor interno.
///
/// # Examples
///
/// ```
/// use rust_concurrency::channels::WorkerPool;
///
/// let pool = WorkerPool::new(2, |value: i32| value + 1);
/// pool.submit(1).unwrap();
/// pool.submit(2).unwrap();
///
/// let mut outputs = pool.shutdown();
/// outputs.sort();
/// assert_eq!(outputs, vec![2, 3]);
/// ```
pub struct WorkerPool<I, O> {
    input: Option<Sender<I>>,
    output: Receiver<O>,
    handles: Vec<JoinHandle<()>>,
}

impl<I, O> WorkerPool<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    /// Crea un pool con `worker_count` workers.
    ///
    /// Complejidad: O(w), donde `w` es el número de workers. Progreso: cada
    /// worker bloquea esperando mensajes hasta que el pool se cierra.
    pub fn new<F>(worker_count: usize, job: F) -> Self
    where
        F: Fn(I) -> O + Send + Sync + 'static,
    {
        assert!(worker_count > 0, "worker_count debe ser mayor que cero");

        let (input_sender, input_receiver) = channel::<I>();
        let (output_sender, output_receiver) = channel::<O>();
        let input_receiver = Arc::new(Mutex::new(input_receiver));
        let job = Arc::new(job);

        let handles = (0..worker_count)
            .map(|_| {
                let input_receiver = Arc::clone(&input_receiver);
                let output_sender = output_sender.clone();
                let job = Arc::clone(&job);

                thread::spawn(move || loop {
                    let message = input_receiver.lock().unwrap().recv();
                    let input = match message {
                        Ok(input) => input,
                        Err(_) => break,
                    };

                    if output_sender.send(job(input)).is_err() {
                        break;
                    }
                })
            })
            .collect();

        drop(output_sender);

        Self {
            input: Some(input_sender),
            output: output_receiver,
            handles,
        }
    }

    /// Envía trabajo al pool.
    ///
    /// Complejidad: O(1) amortizado. Progreso: no espera por capacidad porque
    /// el canal interno no es acotado.
    pub fn submit(&self, input: I) -> Result<(), SendFailure<I>> {
        match &self.input {
            Some(sender) => sender.send(input).map_err(Into::into),
            None => Err(SendFailure::Closed(input)),
        }
    }

    /// Cierra el pool, espera a los workers y recolecta todos los resultados.
    ///
    /// Complejidad: O(j + w), donde `j` es el número de trabajos procesados y
    /// `w` el número de workers.
    pub fn shutdown(mut self) -> Vec<O> {
        drop(self.input.take());

        for handle in self.handles {
            handle.join().expect("worker panic durante shutdown");
        }

        self.output.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{bounded_channel, unbounded_channel, ChannelReceiveError};

    #[test]
    fn empty_channel_closes_after_sender_drop() {
        let (producer, consumer) = unbounded_channel::<i32>();
        drop(producer);

        assert_eq!(consumer.recv(), Err(ChannelReceiveError::Closed));
    }

    #[test]
    fn bounded_zero_capacity_supports_rendezvous() {
        let (producer, consumer) = bounded_channel(0);
        let handle = std::thread::spawn(move || producer.send(42).unwrap());

        assert_eq!(consumer.recv(), Ok(42));
        handle.join().unwrap();
    }
}
