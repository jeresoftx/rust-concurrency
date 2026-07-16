//! Memory ordering.
//!
//! Objetivo de aprendizaje: entender por qué la atomicidad no basta, qué
//! garantizan `Relaxed`, `Acquire`, `Release`, `AcqRel` y `SeqCst`, y cómo
//! razonar sobre happens-before.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Descripción educativa de las garantías de un [`Ordering`].
///
/// Esta estructura no intenta reemplazar la documentación estándar. Resume las
/// preguntas que el capítulo usa para comparar orderings.
///
/// # Examples
///
/// ```
/// use std::sync::atomic::Ordering;
/// use rust_concurrency::memory_ordering::{OrderingGuarantee, describe_ordering};
///
/// assert_eq!(
///     describe_ordering(Ordering::Relaxed),
///     OrderingGuarantee {
///         ordering: Ordering::Relaxed,
///         synchronizes_with_other_threads: false,
///         preserves_single_variable_atomicity: true,
///         participates_in_global_order: false,
///     },
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderingGuarantee {
    /// Ordering descrito.
    pub ordering: Ordering,
    /// Indica si puede formar sincronización entre hilos con una operación
    /// complementaria.
    pub synchronizes_with_other_threads: bool,
    /// Indica si conserva atomicidad sobre la celda individual.
    pub preserves_single_variable_atomicity: bool,
    /// Indica si participa en el orden global único de `SeqCst`.
    pub participates_in_global_order: bool,
}

/// Describe un ordering con tres preguntas pedagógicas.
///
/// Complejidad: O(1).
pub fn describe_ordering(ordering: Ordering) -> OrderingGuarantee {
    OrderingGuarantee {
        ordering,
        synchronizes_with_other_threads: matches!(
            ordering,
            Ordering::Acquire | Ordering::Release | Ordering::AcqRel | Ordering::SeqCst
        ),
        preserves_single_variable_atomicity: true,
        participates_in_global_order: ordering == Ordering::SeqCst,
    }
}

/// Valor publicado con store `Release` y leído con load `Acquire`.
///
/// Este modelo usa un payload atómico para evitar `unsafe`; el punto pedagógico
/// es que la bandera `ready` establece la relación happens-before cuando el
/// lector observa `true` con `Acquire`.
///
/// # Examples
///
/// ```
/// use rust_concurrency::memory_ordering::PublishedValue;
///
/// let value = PublishedValue::new(0);
/// assert_eq!(value.try_read(), None);
/// value.publish(42);
/// assert_eq!(value.try_read(), Some(42));
/// ```
pub struct PublishedValue {
    payload: AtomicUsize,
    ready: AtomicBool,
    observed_reads: AtomicUsize,
}

impl PublishedValue {
    /// Crea un valor no publicado.
    ///
    /// Complejidad: O(1).
    pub fn new(initial: usize) -> Self {
        Self {
            payload: AtomicUsize::new(initial),
            ready: AtomicBool::new(false),
            observed_reads: AtomicUsize::new(0),
        }
    }

    /// Publica `value` con payload `Relaxed` seguido de bandera `Release`.
    ///
    /// Complejidad: O(1).
    pub fn publish(&self, value: usize) {
        self.payload.store(value, Ordering::Relaxed);
        self.ready.store(true, Ordering::Release);
    }

    /// Intenta leer el valor si la bandera `Acquire` ya observa publicación.
    ///
    /// Complejidad: O(1).
    pub fn try_read(&self) -> Option<usize> {
        self.observed_reads.fetch_add(1, Ordering::Relaxed);
        if self.ready.load(Ordering::Acquire) {
            Some(self.payload.load(Ordering::Relaxed))
        } else {
            None
        }
    }

    /// Devuelve cuántos intentos de lectura observó el modelo.
    ///
    /// Complejidad: O(1).
    pub fn observed_reads(&self) -> usize {
        self.observed_reads.load(Ordering::Relaxed)
    }
}

/// Contador relaxed para enseñar atomicidad sin publicación entre variables.
///
/// # Examples
///
/// ```
/// use std::sync::atomic::Ordering;
/// use rust_concurrency::memory_ordering::RelaxedCounter;
///
/// let counter = RelaxedCounter::new(0);
/// counter.increment();
/// assert_eq!(counter.load(), 1);
/// assert_eq!(counter.ordering(), Ordering::Relaxed);
/// ```
pub struct RelaxedCounter {
    value: AtomicUsize,
}

impl RelaxedCounter {
    /// Crea un contador relaxed.
    ///
    /// Complejidad: O(1).
    pub fn new(initial: usize) -> Self {
        Self {
            value: AtomicUsize::new(initial),
        }
    }

    /// Incrementa el contador con `Relaxed`.
    ///
    /// Complejidad: O(1).
    pub fn increment(&self) -> usize {
        self.value.fetch_add(1, Ordering::Relaxed)
    }

    /// Lee el contador con `Relaxed`.
    ///
    /// Complejidad: O(1).
    pub fn load(&self) -> usize {
        self.value.load(Ordering::Relaxed)
    }

    /// Devuelve el ordering usado por este modelo.
    ///
    /// Complejidad: O(1).
    pub fn ordering(&self) -> Ordering {
        Ordering::Relaxed
    }
}

/// Observación de una celda CAS con orderings explícitos.
///
/// # Examples
///
/// ```
/// use std::sync::atomic::Ordering;
/// use rust_concurrency::memory_ordering::OrderingCasCell;
///
/// let cell = OrderingCasCell::new(1);
/// assert_eq!(
///     cell.compare_exchange(1, 2, Ordering::AcqRel, Ordering::Acquire),
///     Ok(1),
/// );
/// assert_eq!(cell.observation().value, 2);
/// ```
pub struct OrderingCasCell {
    value: AtomicUsize,
    attempts: AtomicUsize,
    successes: AtomicUsize,
    last_success_ordering: AtomicUsize,
    last_failure_ordering: AtomicUsize,
}

/// Snapshot observable de [`OrderingCasCell`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderingCasObservation {
    /// Valor actual.
    pub value: usize,
    /// Intentos CAS.
    pub attempts: usize,
    /// Intentos exitosos.
    pub successes: usize,
    /// Último ordering de éxito registrado.
    pub last_success_ordering: Option<Ordering>,
    /// Último ordering de fallo registrado.
    pub last_failure_ordering: Option<Ordering>,
}

impl OrderingCasCell {
    /// Crea una celda CAS.
    ///
    /// Complejidad: O(1).
    pub fn new(initial: usize) -> Self {
        Self {
            value: AtomicUsize::new(initial),
            attempts: AtomicUsize::new(0),
            successes: AtomicUsize::new(0),
            last_success_ordering: AtomicUsize::new(usize::MAX),
            last_failure_ordering: AtomicUsize::new(usize::MAX),
        }
    }

    /// Ejecuta `compare_exchange` con orderings explícitos.
    ///
    /// Complejidad: O(1) por intento.
    pub fn compare_exchange(
        &self,
        current: usize,
        new: usize,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, usize> {
        self.attempts.fetch_add(1, Ordering::Relaxed);
        self.last_success_ordering
            .store(encode_ordering(success), Ordering::Relaxed);
        self.last_failure_ordering
            .store(encode_ordering(failure), Ordering::Relaxed);

        match self.value.compare_exchange(current, new, success, failure) {
            Ok(previous) => {
                self.successes.fetch_add(1, Ordering::Relaxed);
                Ok(previous)
            }
            Err(observed) => Err(observed),
        }
    }

    /// Devuelve una observación de la celda.
    ///
    /// Complejidad: O(1).
    pub fn observation(&self) -> OrderingCasObservation {
        OrderingCasObservation {
            value: self.value.load(Ordering::Relaxed),
            attempts: self.attempts.load(Ordering::Relaxed),
            successes: self.successes.load(Ordering::Relaxed),
            last_success_ordering: decode_ordering(
                self.last_success_ordering.load(Ordering::Relaxed),
            ),
            last_failure_ordering: decode_ordering(
                self.last_failure_ordering.load(Ordering::Relaxed),
            ),
        }
    }
}

fn encode_ordering(ordering: Ordering) -> usize {
    match ordering {
        Ordering::Relaxed => 0,
        Ordering::Release => 1,
        Ordering::Acquire => 2,
        Ordering::AcqRel => 3,
        Ordering::SeqCst => 4,
        _ => 5,
    }
}

fn decode_ordering(value: usize) -> Option<Ordering> {
    match value {
        0 => Some(Ordering::Relaxed),
        1 => Some(Ordering::Release),
        2 => Some(Ordering::Acquire),
        3 => Some(Ordering::AcqRel),
        4 => Some(Ordering::SeqCst),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::{describe_ordering, PublishedValue};

    #[test]
    fn seqcst_is_global_ordering_in_description() {
        let guarantee = describe_ordering(Ordering::SeqCst);

        assert!(guarantee.synchronizes_with_other_threads);
        assert!(guarantee.participates_in_global_order);
    }

    #[test]
    fn unpublished_value_is_not_read() {
        let value = PublishedValue::new(5);

        assert_eq!(value.try_read(), None);
    }
}
