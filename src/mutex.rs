//! Mutex.
//!
//! Objetivo de aprendizaje: entender exclusión mutua, secciones críticas,
//! contención, poisoning, invariantes protegidas y los costos de serializar
//! acceso a estado compartido.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LockResult, Mutex, MutexGuard, TryLockError};

/// Error devuelto por operaciones educativas que no exponen el guard.
///
/// `std::sync::Mutex` conserva el guard dentro del error de poisoning para que
/// el programa pueda decidir si recupera el valor. Métodos como
/// [`EducationalMutex::with_lock`] no dejan escapar el guard, así que usan este
/// error pequeño y empujan la recuperación explícita hacia
/// [`EducationalMutex::recover_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutexAccessError {
    /// El mutex quedó poisoned porque otro hilo entró en pánico mientras
    /// sostenía el lock.
    Poisoned,
}

/// Señales observables de uso de un [`EducationalMutex`].
///
/// Estas métricas no reemplazan a un profiler. Existen para que el capítulo
/// pueda discutir qué operaciones intentaron adquirir el lock, cuántas veces
/// una adquisición no bloqueante encontró contención y cuántas veces se decidió
/// recuperar estado después de poisoning.
///
/// # Examples
///
/// ```
/// use rust_concurrency::mutex::{EducationalMutex, MutexObservation};
///
/// let shared = EducationalMutex::new(1);
/// assert_eq!(
///     shared.observations(),
///     MutexObservation {
///         lock_attempts: 0,
///         try_lock_attempts: 0,
///         try_lock_contentions: 0,
///         poison_recoveries: 0,
///     },
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutexObservation {
    /// Número de intentos bloqueantes hechos con [`EducationalMutex::lock`] o
    /// [`EducationalMutex::with_lock`].
    pub lock_attempts: usize,
    /// Número de intentos no bloqueantes hechos con
    /// [`EducationalMutex::try_with_lock`].
    pub try_lock_attempts: usize,
    /// Número de intentos no bloqueantes que encontraron el mutex ocupado.
    pub try_lock_contentions: usize,
    /// Número de recuperaciones explícitas después de poisoning.
    pub poison_recoveries: usize,
}

/// Mutex educativo para estudiar exclusión mutua en Rust.
///
/// `EducationalMutex<T>` envuelve [`std::sync::Mutex`] y conserva algunas
/// observaciones simples para el capítulo. La intención no es reemplazar a
/// `std::sync::Mutex`, sino ofrecer una API pequeña donde las invariantes sean
/// visibles:
///
/// - solo un hilo puede mutar `T` dentro de una sección crítica;
/// - el guard libera el lock al salir de scope;
/// - el poisoning es observable cuando un hilo entra en pánico con el lock;
/// - la recuperación de poisoning debe ser una decisión explícita.
///
/// Complejidad: adquirir o liberar el mutex delega al sistema operativo o a las
/// primitivas de sincronización de la biblioteca estándar. En ausencia de
/// contención, el costo esperado es constante; bajo contención, el tiempo de
/// espera depende del scheduler y de la duración de las secciones críticas.
///
/// # Examples
///
/// ```
/// use rust_concurrency::mutex::EducationalMutex;
///
/// let shared = EducationalMutex::new(0);
/// shared.with_lock(|value| *value += 1).unwrap();
///
/// assert_eq!(shared.with_lock(|value| *value).unwrap(), 1);
/// ```
pub struct EducationalMutex<T> {
    inner: Mutex<T>,
    lock_attempts: AtomicUsize,
    try_lock_attempts: AtomicUsize,
    try_lock_contentions: AtomicUsize,
    poison_recoveries: AtomicUsize,
}

impl<T> EducationalMutex<T> {
    /// Crea un mutex nuevo que protege `value`.
    ///
    /// Complejidad: O(1) tiempo y O(1) espacio adicional, además del espacio de
    /// `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::mutex::EducationalMutex;
    ///
    /// let shared = EducationalMutex::new(String::from("dato"));
    /// assert_eq!(shared.with_lock(|value| value.clone()).unwrap(), "dato");
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
            lock_attempts: AtomicUsize::new(0),
            try_lock_attempts: AtomicUsize::new(0),
            try_lock_contentions: AtomicUsize::new(0),
            poison_recoveries: AtomicUsize::new(0),
        }
    }

    /// Adquiere el lock y devuelve el guard de la sección crítica.
    ///
    /// Si otro hilo entró en pánico mientras sostenía el lock, la biblioteca
    /// estándar marca el mutex como poisoned y este método devuelve el mismo
    /// [`LockResult`] que [`std::sync::Mutex::lock`].
    ///
    /// Complejidad: O(1) sin contención; bajo contención, el tiempo depende de
    /// cuándo se libere el lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::mutex::EducationalMutex;
    ///
    /// let shared = EducationalMutex::new(10);
    /// let mut guard = shared.lock().unwrap();
    /// *guard += 5;
    /// drop(guard);
    ///
    /// assert_eq!(shared.with_lock(|value| *value).unwrap(), 15);
    /// ```
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        self.lock_attempts.fetch_add(1, Ordering::Relaxed);
        self.inner.lock()
    }

    /// Ejecuta `f` dentro de una sección crítica bloqueante.
    ///
    /// El guard no escapa del closure, así que el lock se libera al terminar la
    /// llamada. Esta forma hace visible una práctica importante: mantener las
    /// secciones críticas pequeñas.
    ///
    /// Complejidad: la adquisición tiene el costo de [`Self::lock`]; el closure
    /// aporta su propio costo.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::mutex::EducationalMutex;
    ///
    /// let shared = EducationalMutex::new(Vec::new());
    /// shared.with_lock(|items| items.push("evento")).unwrap();
    ///
    /// assert_eq!(shared.with_lock(|items| items.len()).unwrap(), 1);
    /// ```
    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, MutexAccessError> {
        match self.lock() {
            Ok(mut guard) => Ok(f(&mut guard)),
            Err(_) => Err(MutexAccessError::Poisoned),
        }
    }

    /// Intenta ejecutar `f` sin bloquear el hilo actual.
    ///
    /// Devuelve `None` si el mutex está ocupado o poisoned. Para enseñar
    /// poisoning con una decisión explícita, usa [`Self::recover_with`].
    ///
    /// Complejidad: O(1). La operación no espera a que otro hilo libere el lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::mutex::EducationalMutex;
    ///
    /// let shared = EducationalMutex::new(3);
    /// assert_eq!(shared.try_with_lock(|value| *value + 1), Some(4));
    /// ```
    pub fn try_with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.try_lock_attempts.fetch_add(1, Ordering::Relaxed);

        match self.inner.try_lock() {
            Ok(mut guard) => Some(f(&mut guard)),
            Err(TryLockError::WouldBlock) => {
                self.try_lock_contentions.fetch_add(1, Ordering::Relaxed);
                None
            }
            Err(TryLockError::Poisoned(_)) => None,
        }
    }

    /// Indica si el mutex quedó poisoned.
    ///
    /// Complejidad: O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::mutex::EducationalMutex;
    ///
    /// let shared = EducationalMutex::new(1);
    /// assert!(!shared.is_poisoned());
    /// ```
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }

    /// Recupera explícitamente el valor protegido aunque el mutex esté poisoned.
    ///
    /// Este método enseña que recuperarse de poisoning no debe ser automático:
    /// el programa debe mirar la invariante protegida y decidir cómo dejarla en
    /// un estado válido.
    ///
    /// Complejidad: la adquisición tiene el costo de [`Self::lock`]; el closure
    /// aporta su propio costo.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::mutex::EducationalMutex;
    ///
    /// let shared = EducationalMutex::new(5);
    /// assert_eq!(shared.recover_with(|value| *value), 5);
    /// ```
    pub fn recover_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.lock_attempts.fetch_add(1, Ordering::Relaxed);
        let mut guard = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                self.poison_recoveries.fetch_add(1, Ordering::Relaxed);
                poisoned.into_inner()
            }
        };

        let result = f(&mut guard);
        self.inner.clear_poison();
        result
    }

    /// Devuelve las señales observables acumuladas.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> MutexObservation {
        MutexObservation {
            lock_attempts: self.lock_attempts.load(Ordering::Relaxed),
            try_lock_attempts: self.try_lock_attempts.load(Ordering::Relaxed),
            try_lock_contentions: self.try_lock_contentions.load(Ordering::Relaxed),
            poison_recoveries: self.poison_recoveries.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EducationalMutex;

    #[test]
    fn with_lock_keeps_mutation_inside_critical_section() {
        let shared = EducationalMutex::new(1);

        shared.with_lock(|value| *value += 1).unwrap();

        assert_eq!(shared.with_lock(|value| *value).unwrap(), 2);
    }

    #[test]
    fn try_with_lock_reports_contention_without_blocking() {
        let shared = EducationalMutex::new(1);
        let _guard = shared.lock().unwrap();

        assert_eq!(shared.try_with_lock(|value| *value), None);
        assert_eq!(shared.observations().try_lock_contentions, 1);
    }
}
