//! Atomics.
//!
//! Objetivo de aprendizaje: entender operaciones atómicas, read-modify-write,
//! contadores compartidos, progreso sin locks y los límites de la atomicidad.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Señales observables de las primitivas atómicas educativas.
///
/// Estas métricas no son un profiler. Existen para mostrar cuántas veces se
/// ejecutaron cargas, stores, operaciones read-modify-write y CAS.
///
/// # Examples
///
/// ```
/// use rust_concurrency::atomics::{AtomicCounter, AtomicObservation};
///
/// let counter = AtomicCounter::new(0);
/// assert_eq!(
///     counter.observations(),
///     AtomicObservation {
///         loads: 0,
///         stores: 0,
///         fetch_adds: 0,
///         compare_exchange_attempts: 0,
///         compare_exchange_successes: 0,
///     },
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtomicObservation {
    /// Número de cargas atómicas observadas por la API educativa.
    pub loads: usize,
    /// Número de stores atómicos observados por la API educativa.
    pub stores: usize,
    /// Número de operaciones `fetch_add`.
    pub fetch_adds: usize,
    /// Número de intentos `compare_exchange`.
    pub compare_exchange_attempts: usize,
    /// Número de intentos `compare_exchange` que cambiaron el valor.
    pub compare_exchange_successes: usize,
}

impl AtomicObservation {
    fn from_parts(
        loads: &AtomicUsize,
        stores: &AtomicUsize,
        fetch_adds: &AtomicUsize,
        compare_exchange_attempts: &AtomicUsize,
        compare_exchange_successes: &AtomicUsize,
    ) -> Self {
        Self {
            loads: loads.load(Ordering::Relaxed),
            stores: stores.load(Ordering::Relaxed),
            fetch_adds: fetch_adds.load(Ordering::Relaxed),
            compare_exchange_attempts: compare_exchange_attempts.load(Ordering::Relaxed),
            compare_exchange_successes: compare_exchange_successes.load(Ordering::Relaxed),
        }
    }
}

/// Contador atómico educativo con política de overflow saturante.
///
/// `AtomicCounter` enseña `load`, `store` y `fetch_add`. Usa `Ordering::Relaxed`
/// porque el capítulo se concentra en atomicidad de una variable; las garantías
/// de publicación entre variables viven en el capítulo de memory ordering.
///
/// # Examples
///
/// ```
/// use rust_concurrency::atomics::AtomicCounter;
///
/// let counter = AtomicCounter::new(1);
/// assert_eq!(counter.fetch_add(2), 1);
/// assert_eq!(counter.load(), 3);
/// ```
pub struct AtomicCounter {
    value: AtomicUsize,
    loads: AtomicUsize,
    stores: AtomicUsize,
    fetch_adds: AtomicUsize,
}

impl AtomicCounter {
    /// Crea un contador con valor inicial.
    ///
    /// Complejidad: O(1).
    pub fn new(value: usize) -> Self {
        Self {
            value: AtomicUsize::new(value),
            loads: AtomicUsize::new(0),
            stores: AtomicUsize::new(0),
            fetch_adds: AtomicUsize::new(0),
        }
    }

    /// Lee el valor actual.
    ///
    /// Complejidad: O(1).
    pub fn load(&self) -> usize {
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.value.load(Ordering::Relaxed)
    }

    /// Sobrescribe el valor actual.
    ///
    /// Complejidad: O(1).
    pub fn store(&self, value: usize) {
        self.stores.fetch_add(1, Ordering::Relaxed);
        self.value.store(value, Ordering::Relaxed);
    }

    /// Suma `delta` y devuelve el valor anterior.
    ///
    /// La suma satura en `usize::MAX` para que la política de overflow sea
    /// explícita en el curso. Si no hay overflow, esta operación delega a
    /// `fetch_add`; si puede haber overflow, usa un CAS loop.
    ///
    /// Complejidad: O(1) esperado sin contención; con contención, puede repetir
    /// el CAS loop hasta observar un valor estable.
    pub fn fetch_add(&self, delta: usize) -> usize {
        self.fetch_adds.fetch_add(1, Ordering::Relaxed);

        let mut current = self.value.load(Ordering::Relaxed);
        loop {
            let next = current.saturating_add(delta);
            match self.value.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => return previous,
                Err(observed) => current = observed,
            }
        }
    }

    /// Devuelve las señales observables acumuladas.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> AtomicObservation {
        AtomicObservation::from_parts(
            &self.loads,
            &self.stores,
            &self.fetch_adds,
            &AtomicUsize::new(0),
            &AtomicUsize::new(0),
        )
    }
}

/// Bandera atómica educativa para publicar un estado booleano simple.
///
/// # Examples
///
/// ```
/// use rust_concurrency::atomics::AtomicFlag;
///
/// let flag = AtomicFlag::new(false);
/// flag.set(true);
/// assert!(flag.is_set());
/// ```
pub struct AtomicFlag {
    value: AtomicBool,
    loads: AtomicUsize,
    stores: AtomicUsize,
}

impl AtomicFlag {
    /// Crea una bandera con valor inicial.
    ///
    /// Complejidad: O(1).
    pub fn new(value: bool) -> Self {
        Self {
            value: AtomicBool::new(value),
            loads: AtomicUsize::new(0),
            stores: AtomicUsize::new(0),
        }
    }

    /// Lee la bandera.
    ///
    /// Complejidad: O(1).
    pub fn is_set(&self) -> bool {
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.value.load(Ordering::Relaxed)
    }

    /// Publica un nuevo valor para la bandera.
    ///
    /// Complejidad: O(1).
    pub fn set(&self, value: bool) {
        self.stores.fetch_add(1, Ordering::Relaxed);
        self.value.store(value, Ordering::Relaxed);
    }

    /// Devuelve las señales observables acumuladas.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> AtomicObservation {
        AtomicObservation::from_parts(
            &self.loads,
            &self.stores,
            &AtomicUsize::new(0),
            &AtomicUsize::new(0),
            &AtomicUsize::new(0),
        )
    }
}

/// Celda atómica educativa para estudiar `compare_exchange`.
///
/// # Examples
///
/// ```
/// use rust_concurrency::atomics::CompareExchange;
///
/// let slot = CompareExchange::new(1);
/// assert_eq!(slot.compare_exchange(1, 2), Ok(1));
/// assert_eq!(slot.compare_exchange(1, 3), Err(2));
/// ```
pub struct CompareExchange {
    value: AtomicUsize,
    loads: AtomicUsize,
    stores: AtomicUsize,
    compare_exchange_attempts: AtomicUsize,
    compare_exchange_successes: AtomicUsize,
}

impl CompareExchange {
    /// Crea una celda con valor inicial.
    ///
    /// Complejidad: O(1).
    pub fn new(value: usize) -> Self {
        Self {
            value: AtomicUsize::new(value),
            loads: AtomicUsize::new(0),
            stores: AtomicUsize::new(0),
            compare_exchange_attempts: AtomicUsize::new(0),
            compare_exchange_successes: AtomicUsize::new(0),
        }
    }

    /// Lee el valor actual.
    ///
    /// Complejidad: O(1).
    pub fn load(&self) -> usize {
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.value.load(Ordering::Relaxed)
    }

    /// Sobrescribe el valor actual.
    ///
    /// Complejidad: O(1).
    pub fn store(&self, value: usize) {
        self.stores.fetch_add(1, Ordering::Relaxed);
        self.value.store(value, Ordering::Relaxed);
    }

    /// Cambia el valor si todavía coincide con `current`.
    ///
    /// Devuelve `Ok(valor_anterior)` si el intercambio ocurre y
    /// `Err(valor_observado)` si otro hilo cambió el valor antes.
    ///
    /// Complejidad: O(1) por intento.
    pub fn compare_exchange(&self, current: usize, new: usize) -> Result<usize, usize> {
        self.compare_exchange_attempts
            .fetch_add(1, Ordering::Relaxed);

        match self
            .value
            .compare_exchange(current, new, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(previous) => {
                self.compare_exchange_successes
                    .fetch_add(1, Ordering::Relaxed);
                Ok(previous)
            }
            Err(observed) => Err(observed),
        }
    }

    /// Devuelve las señales observables acumuladas.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> AtomicObservation {
        AtomicObservation::from_parts(
            &self.loads,
            &self.stores,
            &AtomicUsize::new(0),
            &self.compare_exchange_attempts,
            &self.compare_exchange_successes,
        )
    }
}

/// Registro atómico del valor máximo observado.
///
/// Este tipo enseña un CAS loop: leer, calcular un candidato y reintentar si
/// otro hilo cambió el valor entre la lectura y el intercambio.
///
/// # Examples
///
/// ```
/// use rust_concurrency::atomics::AtomicMax;
///
/// let max = AtomicMax::new(10);
/// max.record(12);
/// max.record(11);
/// assert_eq!(max.load(), 12);
/// ```
pub struct AtomicMax {
    value: AtomicUsize,
    loads: AtomicUsize,
    compare_exchange_attempts: AtomicUsize,
    compare_exchange_successes: AtomicUsize,
}

impl AtomicMax {
    /// Crea un registro con máximo inicial.
    ///
    /// Complejidad: O(1).
    pub fn new(value: usize) -> Self {
        Self {
            value: AtomicUsize::new(value),
            loads: AtomicUsize::new(0),
            compare_exchange_attempts: AtomicUsize::new(0),
            compare_exchange_successes: AtomicUsize::new(0),
        }
    }

    /// Lee el máximo observado.
    ///
    /// Complejidad: O(1).
    pub fn load(&self) -> usize {
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.value.load(Ordering::Relaxed)
    }

    /// Registra `candidate` si es mayor que el valor actual.
    ///
    /// Complejidad: O(1) esperado sin contención; con contención, puede repetir
    /// el CAS loop.
    pub fn record(&self, candidate: usize) {
        let mut current = self.value.load(Ordering::Relaxed);
        while candidate > current {
            self.compare_exchange_attempts
                .fetch_add(1, Ordering::Relaxed);
            match self.value.compare_exchange_weak(
                current,
                candidate,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.compare_exchange_successes
                        .fetch_add(1, Ordering::Relaxed);
                    return;
                }
                Err(observed) => current = observed,
            }
        }
    }

    /// Devuelve las señales observables acumuladas.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> AtomicObservation {
        AtomicObservation::from_parts(
            &self.loads,
            &AtomicUsize::new(0),
            &AtomicUsize::new(0),
            &self.compare_exchange_attempts,
            &self.compare_exchange_successes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{AtomicCounter, AtomicFlag, CompareExchange};

    #[test]
    fn counter_store_replaces_value() {
        let counter = AtomicCounter::new(1);

        counter.store(9);

        assert_eq!(counter.load(), 9);
    }

    #[test]
    fn flag_observations_track_loads_and_stores() {
        let flag = AtomicFlag::new(false);

        flag.set(true);
        assert!(flag.is_set());

        let observations = flag.observations();
        assert_eq!(observations.loads, 1);
        assert_eq!(observations.stores, 1);
    }

    #[test]
    fn compare_exchange_store_replaces_value() {
        let slot = CompareExchange::new(1);

        slot.store(3);

        assert_eq!(slot.load(), 3);
    }
}
