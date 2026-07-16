//! Arc.
//!
//! Objetivo de aprendizaje: entender conteo atómico de referencias, ownership
//! compartido entre hilos, ciclos, costos y la frontera entre compartir datos y
//! compartir mutabilidad.

use std::ops::Deref;
use std::sync::{Arc, Weak};

/// Observación puntual de conteos de un [`Shared`] o [`SharedWeak`].
///
/// `strong_count` indica cuántos dueños fuertes mantienen vivo el valor.
/// `weak_count` indica cuántas referencias débiles pueden intentar observarlo
/// sin extender su vida.
///
/// # Examples
///
/// ```
/// use rust_concurrency::arc::{ArcObservation, Shared};
///
/// let shared = Shared::new("dato");
/// assert_eq!(
///     shared.observations(),
///     ArcObservation {
///         strong_count: 1,
///         weak_count: 0,
///     },
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcObservation {
    /// Número de referencias fuertes.
    pub strong_count: usize,
    /// Número de referencias débiles asociadas al valor mientras está vivo.
    pub weak_count: usize,
}

/// Wrapper educativo sobre [`std::sync::Arc`].
///
/// `Shared<T>` no reemplaza a `Arc<T>`. Solo nombra operaciones relevantes para
/// el capítulo: clonar ownership compartido, crear referencias débiles, leer
/// conteos y acceder al valor sin moverlo.
///
/// # Examples
///
/// ```
/// use rust_concurrency::arc::Shared;
///
/// let shared = Shared::new(vec![1, 2, 3]);
/// let cloned = shared.clone_shared();
///
/// assert_eq!(shared.strong_count(), 2);
/// assert_eq!(cloned.with_ref(|items| items.len()), 3);
/// ```
pub struct Shared<T> {
    inner: Arc<T>,
}

impl<T> Shared<T> {
    /// Crea un dueño compartido nuevo.
    ///
    /// Complejidad: O(1), además de mover `value` al heap administrado por
    /// `Arc`.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    /// Clona el dueño fuerte del mismo valor.
    ///
    /// Complejidad: O(1); internamente incrementa un contador atómico.
    pub fn clone_shared(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Crea una referencia débil que no mantiene vivo el valor.
    ///
    /// Complejidad: O(1); internamente incrementa el contador débil.
    pub fn downgrade(&self) -> SharedWeak<T> {
        SharedWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Ejecuta `f` con una referencia compartida al valor.
    ///
    /// Complejidad: O(1) más el costo de `f`.
    pub fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.inner)
    }

    /// Devuelve el número actual de referencias fuertes.
    ///
    /// Complejidad: O(1).
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Devuelve el número actual de referencias débiles.
    ///
    /// Complejidad: O(1).
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.inner)
    }

    /// Devuelve ambos conteos en una sola estructura.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> ArcObservation {
        ArcObservation {
            strong_count: self.strong_count(),
            weak_count: self.weak_count(),
        }
    }

    /// Devuelve una referencia al `Arc<T>` interno para comparaciones con APIs
    /// estándar.
    ///
    /// Complejidad: O(1).
    pub fn as_arc(&self) -> &Arc<T> {
        &self.inner
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        self.clone_shared()
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Referencia débil educativa asociada a un [`Shared`].
///
/// Una referencia débil no mantiene vivo el valor. Para usarlo, primero debe
/// intentar convertirse en [`Shared`] con [`SharedWeak::upgrade`].
///
/// # Examples
///
/// ```
/// use rust_concurrency::arc::Shared;
///
/// let weak = {
///     let shared = Shared::new(7);
///     let weak = shared.downgrade();
///     assert!(weak.upgrade().is_some());
///     weak
/// };
///
/// assert!(weak.upgrade().is_none());
/// ```
pub struct SharedWeak<T> {
    inner: Weak<T>,
}

impl<T> SharedWeak<T> {
    /// Intenta recuperar un dueño fuerte del valor.
    ///
    /// Devuelve `None` si ya no quedan referencias fuertes.
    ///
    /// Complejidad: O(1).
    pub fn upgrade(&self) -> Option<Shared<T>> {
        self.inner.upgrade().map(|inner| Shared { inner })
    }

    /// Devuelve el número actual de referencias fuertes observables desde weak.
    ///
    /// Complejidad: O(1).
    pub fn strong_count(&self) -> usize {
        self.inner.strong_count()
    }

    /// Devuelve el número actual de referencias débiles observables desde weak.
    ///
    /// Cuando ya no quedan referencias fuertes, la biblioteca estándar reporta
    /// cero referencias débiles asociadas al valor, aunque esta handle débil
    /// todavía exista como objeto.
    ///
    /// Complejidad: O(1).
    pub fn weak_count(&self) -> usize {
        self.inner.weak_count()
    }

    /// Devuelve ambos conteos en una sola estructura.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> ArcObservation {
        ArcObservation {
            strong_count: self.strong_count(),
            weak_count: self.weak_count(),
        }
    }
}

impl<T> Clone for SharedWeak<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Shared;

    #[test]
    fn clone_shared_increments_strong_count() {
        let shared = Shared::new(1);
        let cloned = shared.clone_shared();

        assert_eq!(shared.strong_count(), 2);
        assert_eq!(cloned.strong_count(), 2);
    }

    #[test]
    fn downgrade_increments_weak_count() {
        let shared = Shared::new(1);
        let _weak = shared.downgrade();

        assert_eq!(shared.weak_count(), 1);
    }
}
