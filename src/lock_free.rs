//! Lock-free structures.
//!
//! Objetivo de aprendizaje: entender progreso lock-free, ABA, CAS loops,
//! contención y por qué estas estructuras son poderosas pero difíciles de
//! verificar.

use std::sync::atomic::{AtomicUsize, Ordering};

const NONE: usize = usize::MAX;

/// Garantía de progreso de una estructura concurrente.
///
/// # Examples
///
/// ```
/// use rust_concurrency::lock_free::ProgressGuarantee;
///
/// assert!(ProgressGuarantee::LockFree.stronger_than(ProgressGuarantee::Blocking));
/// assert!(ProgressGuarantee::WaitFree.stronger_than(ProgressGuarantee::LockFree));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressGuarantee {
    /// Algún hilo puede quedar bloqueado por otro.
    Blocking,
    /// El sistema como conjunto progresa aunque un hilo individual reintente.
    LockFree,
    /// Cada operación termina en un número acotado de pasos.
    WaitFree,
}

impl ProgressGuarantee {
    /// Indica si `self` es una garantía más fuerte que `other`.
    ///
    /// Complejidad: O(1).
    pub fn stronger_than(self, other: Self) -> bool {
        self.rank() > other.rank()
    }

    fn rank(self) -> usize {
        match self {
            Self::Blocking => 0,
            Self::LockFree => 1,
            Self::WaitFree => 2,
        }
    }
}

/// Observaciones de un modelo con CAS loop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CasObservation {
    /// Intentos de `compare_exchange`.
    pub cas_attempts: usize,
    /// CAS exitosos.
    pub cas_successes: usize,
    /// Reintentos causados por contención o interferencia.
    pub retries: usize,
}

/// Error al insertar en una pila acotada.
///
/// # Examples
///
/// ```
/// use rust_concurrency::lock_free::{BoundedLockFreeStack, StackError};
///
/// let stack = BoundedLockFreeStack::new(1);
/// stack.push(1).unwrap();
/// assert_eq!(stack.push(2), Err(StackError::Full(2)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackError {
    /// No hay nodos libres; se devuelve el valor original.
    Full(usize),
}

#[derive(Debug)]
struct Node {
    value: AtomicUsize,
    next: AtomicUsize,
}

impl Node {
    fn new(next: usize) -> Self {
        Self {
            value: AtomicUsize::new(0),
            next: AtomicUsize::new(next),
        }
    }
}

/// Pila lock-free acotada de `usize`, implementada sin `unsafe`.
///
/// Esta estructura usa índices sobre nodos preasignados en vez de punteros
/// crudos. Eso permite enseñar CAS loops, reintentos y reutilización de nodos
/// sin introducir todavía reclamación de memoria.
///
/// # Examples
///
/// ```
/// use rust_concurrency::lock_free::BoundedLockFreeStack;
///
/// let stack = BoundedLockFreeStack::new(2);
/// stack.push(10).unwrap();
/// stack.push(20).unwrap();
///
/// assert_eq!(stack.pop(), Some(20));
/// assert_eq!(stack.pop(), Some(10));
/// assert_eq!(stack.pop(), None);
/// ```
pub struct BoundedLockFreeStack {
    nodes: Vec<Node>,
    head: AtomicUsize,
    free: AtomicUsize,
    cas_attempts: AtomicUsize,
    cas_successes: AtomicUsize,
    retries: AtomicUsize,
}

impl BoundedLockFreeStack {
    /// Crea una pila acotada con `capacity` nodos preasignados.
    ///
    /// Complejidad: O(n). Progreso: inicialización secuencial.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity debe ser mayor que cero");

        let nodes = (0..capacity)
            .map(|index| {
                let next = if index + 1 == capacity {
                    NONE
                } else {
                    index + 1
                };
                Node::new(next)
            })
            .collect();

        Self {
            nodes,
            head: AtomicUsize::new(NONE),
            free: AtomicUsize::new(0),
            cas_attempts: AtomicUsize::new(0),
            cas_successes: AtomicUsize::new(0),
            retries: AtomicUsize::new(0),
        }
    }

    /// Devuelve la capacidad fija de la pila.
    ///
    /// Complejidad: O(1).
    pub fn capacity(&self) -> usize {
        self.nodes.len()
    }

    /// Garantía de progreso del modelo.
    ///
    /// Complejidad: O(1).
    pub fn progress_guarantee(&self) -> ProgressGuarantee {
        ProgressGuarantee::LockFree
    }

    /// Inserta un valor si existe un nodo libre.
    ///
    /// Complejidad: O(1) esperado sin contención; con contención puede
    /// reintentar. Progreso: lock-free, no wait-free.
    pub fn push(&self, value: usize) -> Result<(), StackError> {
        let node = match self.pop_free_node() {
            Some(node) => node,
            None => return Err(StackError::Full(value)),
        };

        self.nodes[node].value.store(value, Ordering::Relaxed);
        self.push_head_node(node);
        Ok(())
    }

    /// Extrae el valor superior de la pila.
    ///
    /// Complejidad: O(1) esperado sin contención; con contención puede
    /// reintentar. Progreso: lock-free, no wait-free.
    pub fn pop(&self) -> Option<usize> {
        let node = self.pop_head_node()?;
        let value = self.nodes[node].value.load(Ordering::Relaxed);
        self.push_free_node(node);
        Some(value)
    }

    /// Devuelve las observaciones acumuladas de CAS.
    ///
    /// Complejidad: O(1).
    pub fn observations(&self) -> CasObservation {
        CasObservation {
            cas_attempts: self.cas_attempts.load(Ordering::Relaxed),
            cas_successes: self.cas_successes.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
        }
    }

    fn pop_free_node(&self) -> Option<usize> {
        loop {
            let current = self.free.load(Ordering::Acquire);
            if current == NONE {
                return None;
            }
            let next = self.nodes[current].next.load(Ordering::Relaxed);
            if self.cas(
                &self.free,
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                return Some(current);
            }
        }
    }

    fn push_free_node(&self, node: usize) {
        loop {
            let current = self.free.load(Ordering::Acquire);
            self.nodes[node].next.store(current, Ordering::Relaxed);
            if self.cas(
                &self.free,
                current,
                node,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                return;
            }
        }
    }

    fn pop_head_node(&self) -> Option<usize> {
        loop {
            let current = self.head.load(Ordering::Acquire);
            if current == NONE {
                return None;
            }
            let next = self.nodes[current].next.load(Ordering::Relaxed);
            if self.cas(
                &self.head,
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                return Some(current);
            }
        }
    }

    fn push_head_node(&self, node: usize) {
        loop {
            let current = self.head.load(Ordering::Acquire);
            self.nodes[node].next.store(current, Ordering::Relaxed);
            if self.cas(
                &self.head,
                current,
                node,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                return;
            }
        }
    }

    fn cas(
        &self,
        target: &AtomicUsize,
        current: usize,
        next: usize,
        success: Ordering,
        failure: Ordering,
    ) -> bool {
        self.cas_attempts.fetch_add(1, Ordering::Relaxed);
        match target.compare_exchange_weak(current, next, success, failure) {
            Ok(_) => {
                self.cas_successes.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(_) => {
                self.retries.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }
}

/// Escenario educativo para describir riesgo ABA.
///
/// # Examples
///
/// ```
/// use rust_concurrency::lock_free::AbaScenario;
///
/// let scenario = AbaScenario::new(3, 1, 3);
/// assert!(scenario.is_aba_risk());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbaScenario {
    observed_before: usize,
    changed_to: usize,
    observed_after: usize,
}

impl AbaScenario {
    /// Crea un escenario con la observación inicial, el valor intermedio y la
    /// observación final.
    ///
    /// Complejidad: O(1).
    pub fn new(observed_before: usize, changed_to: usize, observed_after: usize) -> Self {
        Self {
            observed_before,
            changed_to,
            observed_after,
        }
    }

    /// Indica si la forma A -> B -> A aparece.
    ///
    /// Complejidad: O(1).
    pub fn is_aba_risk(&self) -> bool {
        self.observed_before == self.observed_after && self.observed_before != self.changed_to
    }

    /// Describe el riesgo de manera legible.
    ///
    /// Complejidad: O(1).
    pub fn description(&self) -> String {
        format!(
            "head cambió de {} a {} y regresó a {}; el índice observado parece igual, pero la historia cambió",
            self.observed_before, self.changed_to, self.observed_after
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{AbaScenario, BoundedLockFreeStack, ProgressGuarantee};

    #[test]
    fn progress_ordering_compares_guarantees() {
        assert!(ProgressGuarantee::LockFree.stronger_than(ProgressGuarantee::Blocking));
        assert!(ProgressGuarantee::WaitFree.stronger_than(ProgressGuarantee::LockFree));
    }

    #[test]
    fn non_aba_sequence_is_not_reported_as_risk() {
        let scenario = AbaScenario::new(1, 2, 3);

        assert!(!scenario.is_aba_risk());
    }

    #[test]
    fn capacity_is_fixed() {
        let stack = BoundedLockFreeStack::new(3);

        assert_eq!(stack.capacity(), 3);
    }
}
