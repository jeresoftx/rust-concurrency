//! RwLock.
//!
//! Objetivo de aprendizaje: entender bloqueo compartido/exclusivo, cargas de
//! lectura dominante, starvation, fairness y cuándo `RwLock` empeora frente a
//! `Mutex`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

/// Error devuelto por operaciones educativas que no exponen guards.
///
/// `std::sync::RwLock` conserva el guard dentro del error de poisoning. Los
/// helpers basados en closures no dejan escapar ese guard, así que devuelven
/// este error pequeño y reservan la reparación explícita para
/// [`EducationalRwLock::recover_write_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RwLockAccessError {
    /// El lock quedó poisoned porque un hilo entró en pánico mientras sostenía
    /// el lock de escritura.
    Poisoned,
}

/// Señales observables de uso de un [`EducationalRwLock`].
///
/// Estas métricas son pedagógicas. Sirven para conversar sobre cargas de
/// lectura, exclusión de escritura, contención y recuperación después de
/// poisoning sin convertir este wrapper en una herramienta de profiling.
///
/// # Examples
///
/// ```
/// use rust_concurrency::rwlock::{EducationalRwLock, RwLockObservation};
///
/// let shared = EducationalRwLock::new(1);
/// assert_eq!(
///     shared.observations(),
///     RwLockObservation {
///         read_attempts: 0,
///         write_attempts: 0,
///         try_read_attempts: 0,
///         try_write_attempts: 0,
///         try_read_contentions: 0,
///         try_write_contentions: 0,
///         poison_recoveries: 0,
///     },
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RwLockObservation {
    /// Número de intentos bloqueantes de lectura.
    pub read_attempts: usize,
    /// Número de intentos bloqueantes de escritura.
    pub write_attempts: usize,
    /// Número de intentos no bloqueantes de lectura.
    pub try_read_attempts: usize,
    /// Número de intentos no bloqueantes de escritura.
    pub try_write_attempts: usize,
    /// Número de intentos de lectura no bloqueante que encontraron contención.
    pub try_read_contentions: usize,
    /// Número de intentos de escritura no bloqueante que encontraron contención.
    pub try_write_contentions: usize,
    /// Número de recuperaciones explícitas después de poisoning.
    pub poison_recoveries: usize,
}

/// RwLock educativo para estudiar lectores compartidos y escritores exclusivos.
///
/// `EducationalRwLock<T>` envuelve [`std::sync::RwLock`] y expone observaciones
/// simples sobre el uso del lock. La intención es enseñar cuándo varias
/// lecturas pueden convivir, cuándo una escritura necesita exclusión total y
/// por qué las cargas de lectura dominante pueden beneficiarse de esta
/// primitiva.
///
/// Complejidad: adquirir un guard de lectura o escritura delega a la biblioteca
/// estándar. Sin contención, el costo esperado es constante; bajo contención,
/// el tiempo depende del scheduler, la política del sistema y la duración de
/// las secciones críticas.
///
/// # Examples
///
/// ```
/// use rust_concurrency::rwlock::EducationalRwLock;
///
/// let shared = EducationalRwLock::new(vec![1, 2, 3]);
/// assert_eq!(shared.with_read(|items| items.len()).unwrap(), 3);
///
/// shared.with_write(|items| items.push(4)).unwrap();
/// assert_eq!(shared.with_read(|items| items.iter().sum::<i32>()).unwrap(), 10);
/// ```
pub struct EducationalRwLock<T> {
    inner: RwLock<T>,
    read_attempts: AtomicUsize,
    write_attempts: AtomicUsize,
    try_read_attempts: AtomicUsize,
    try_write_attempts: AtomicUsize,
    try_read_contentions: AtomicUsize,
    try_write_contentions: AtomicUsize,
    poison_recoveries: AtomicUsize,
}

impl<T> EducationalRwLock<T> {
    /// Crea un RwLock nuevo que protege `value`.
    ///
    /// Complejidad: O(1) tiempo y O(1) espacio adicional, además del espacio de
    /// `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(String::from("lectura"));
    /// assert_eq!(shared.with_read(|value| value.clone()).unwrap(), "lectura");
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(value),
            read_attempts: AtomicUsize::new(0),
            write_attempts: AtomicUsize::new(0),
            try_read_attempts: AtomicUsize::new(0),
            try_write_attempts: AtomicUsize::new(0),
            try_read_contentions: AtomicUsize::new(0),
            try_write_contentions: AtomicUsize::new(0),
            poison_recoveries: AtomicUsize::new(0),
        }
    }

    /// Adquiere un guard de lectura compartida.
    ///
    /// Varios lectores pueden coexistir mientras no haya un escritor activo.
    /// Si el lock está poisoned, devuelve el mismo [`LockResult`] que
    /// [`std::sync::RwLock::read`].
    ///
    /// Complejidad: O(1) sin contención; bajo contención, el tiempo depende de
    /// cuándo puedan entrar lectores nuevos.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(3);
    /// let guard = shared.read().unwrap();
    /// assert_eq!(*guard, 3);
    /// ```
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.read_attempts.fetch_add(1, Ordering::Relaxed);
        self.inner.read()
    }

    /// Adquiere un guard de escritura exclusiva.
    ///
    /// Mientras exista este guard, no puede existir otro lector ni otro
    /// escritor sobre el mismo valor protegido.
    ///
    /// Complejidad: O(1) sin contención; bajo contención, el tiempo depende de
    /// cuándo lectores y escritores liberen sus guards.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(3);
    /// *shared.write().unwrap() += 1;
    /// assert_eq!(shared.with_read(|value| *value).unwrap(), 4);
    /// ```
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.write_attempts.fetch_add(1, Ordering::Relaxed);
        self.inner.write()
    }

    /// Ejecuta `f` dentro de una sección crítica de lectura.
    ///
    /// El closure recibe `&T`, por lo que expresa en el tipo que no hay mutación
    /// del valor protegido.
    ///
    /// Complejidad: la adquisición tiene el costo de [`Self::read`]; el closure
    /// aporta su propio costo.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(vec!["a", "b"]);
    /// assert_eq!(shared.with_read(|items| items.len()).unwrap(), 2);
    /// ```
    pub fn with_read<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, RwLockAccessError> {
        match self.read() {
            Ok(guard) => Ok(f(&guard)),
            Err(_) => Err(RwLockAccessError::Poisoned),
        }
    }

    /// Ejecuta `f` dentro de una sección crítica de escritura.
    ///
    /// El closure recibe `&mut T`; por lo tanto, la operación exige exclusión
    /// frente a todos los lectores y escritores.
    ///
    /// Complejidad: la adquisición tiene el costo de [`Self::write`]; el
    /// closure aporta su propio costo.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(Vec::new());
    /// shared.with_write(|items| items.push("evento")).unwrap();
    /// assert_eq!(shared.with_read(|items| items.len()).unwrap(), 1);
    /// ```
    pub fn with_write<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, RwLockAccessError> {
        match self.write() {
            Ok(mut guard) => Ok(f(&mut guard)),
            Err(_) => Err(RwLockAccessError::Poisoned),
        }
    }

    /// Intenta ejecutar `f` como lector sin bloquear el hilo actual.
    ///
    /// Devuelve `None` si el lock no permite entrar ahora o si está poisoned.
    /// Para reparar poisoning, usa [`Self::recover_write_with`].
    ///
    /// Complejidad: O(1). La operación no espera a que otros guards terminen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(5);
    /// assert_eq!(shared.try_with_read(|value| *value), Some(5));
    /// ```
    pub fn try_with_read<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.try_read_attempts.fetch_add(1, Ordering::Relaxed);

        match self.inner.try_read() {
            Ok(guard) => Some(f(&guard)),
            Err(TryLockError::WouldBlock) => {
                self.try_read_contentions.fetch_add(1, Ordering::Relaxed);
                None
            }
            Err(TryLockError::Poisoned(_)) => None,
        }
    }

    /// Intenta ejecutar `f` como escritor sin bloquear el hilo actual.
    ///
    /// Devuelve `None` si existe cualquier lector o escritor activo, o si el
    /// lock está poisoned.
    ///
    /// Complejidad: O(1). La operación no espera a que otros guards terminen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(5);
    /// assert_eq!(shared.try_with_write(|value| {
    ///     *value += 1;
    ///     *value
    /// }), Some(6));
    /// ```
    pub fn try_with_write<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.try_write_attempts.fetch_add(1, Ordering::Relaxed);

        match self.inner.try_write() {
            Ok(mut guard) => Some(f(&mut guard)),
            Err(TryLockError::WouldBlock) => {
                self.try_write_contentions.fetch_add(1, Ordering::Relaxed);
                None
            }
            Err(TryLockError::Poisoned(_)) => None,
        }
    }

    /// Indica si el lock quedó poisoned.
    ///
    /// En `std::sync::RwLock`, el poisoning ocurre por pánico con lock de
    /// escritura; un pánico con guard de lectura no marca el lock como poisoned.
    ///
    /// Complejidad: O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(1);
    /// assert!(!shared.is_poisoned());
    /// ```
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }

    /// Recupera explícitamente el valor protegido aunque el lock esté poisoned.
    ///
    /// La recuperación requiere acceso de escritura porque reparar una
    /// invariante normalmente implica modificar el valor protegido.
    ///
    /// Complejidad: la adquisición tiene el costo de [`Self::write`]; el
    /// closure aporta su propio costo.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_concurrency::rwlock::EducationalRwLock;
    ///
    /// let shared = EducationalRwLock::new(5);
    /// assert_eq!(shared.recover_write_with(|value| *value), 5);
    /// ```
    pub fn recover_write_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.write_attempts.fetch_add(1, Ordering::Relaxed);
        let mut guard = match self.inner.write() {
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
    pub fn observations(&self) -> RwLockObservation {
        RwLockObservation {
            read_attempts: self.read_attempts.load(Ordering::Relaxed),
            write_attempts: self.write_attempts.load(Ordering::Relaxed),
            try_read_attempts: self.try_read_attempts.load(Ordering::Relaxed),
            try_write_attempts: self.try_write_attempts.load(Ordering::Relaxed),
            try_read_contentions: self.try_read_contentions.load(Ordering::Relaxed),
            try_write_contentions: self.try_write_contentions.load(Ordering::Relaxed),
            poison_recoveries: self.poison_recoveries.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EducationalRwLock;

    #[test]
    fn with_read_allows_repeated_shared_access() {
        let shared = EducationalRwLock::new(vec![1, 2, 3]);

        assert_eq!(shared.with_read(|items| items.len()).unwrap(), 3);
        assert_eq!(
            shared.with_read(|items| items.iter().sum::<i32>()).unwrap(),
            6
        );
    }

    #[test]
    fn try_with_write_reports_reader_contention() {
        let shared = EducationalRwLock::new(1);
        let _reader = shared.read().unwrap();

        assert_eq!(shared.try_with_write(|value| *value), None);
        assert_eq!(shared.observations().try_write_contentions, 1);
    }
}
