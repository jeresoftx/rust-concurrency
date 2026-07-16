//! Deadlocks.
//!
//! Objetivo de aprendizaje: entender las condiciones de Coffman, detección,
//! prevención, ordenamiento de locks y diseño de APIs que reduzcan esperas
//! circulares.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

/// Rango de adquisición para un lock dentro de un orden total.
///
/// # Examples
///
/// ```
/// use rust_concurrency::deadlocks::LockRank;
///
/// let account = LockRank::new(10, "account");
/// assert_eq!(account.rank(), 10);
/// assert_eq!(account.name(), "account");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LockRank {
    rank: usize,
    name: &'static str,
}

impl LockRank {
    /// Crea un rango nombrado.
    ///
    /// Complejidad: O(1).
    pub fn new(rank: usize, name: &'static str) -> Self {
        Self { rank, name }
    }

    /// Devuelve el rango numérico.
    ///
    /// Complejidad: O(1).
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// Devuelve el nombre pedagógico del lock.
    ///
    /// Complejidad: O(1).
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Error al intentar adquirir un lock fuera de orden.
///
/// # Examples
///
/// ```
/// use rust_concurrency::deadlocks::{LockOrderTracker, LockRank};
///
/// let mut tracker = LockOrderTracker::new();
/// let high = LockRank::new(20, "high");
/// let low = LockRank::new(10, "low");
///
/// tracker.enter(high).unwrap();
/// assert!(tracker.enter(low).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockOrderViolation {
    /// Lock ya sostenido con rango mayor.
    pub held: LockRank,
    /// Lock solicitado con rango menor.
    pub requested: LockRank,
}

/// Rastreador educativo de orden de locks.
///
/// Este tipo no adquiere locks reales. Valida el protocolo: mientras un hilo
/// sostiene locks, solo puede pedir locks con rango igual o mayor al máximo
/// sostenido.
///
/// # Examples
///
/// ```
/// use rust_concurrency::deadlocks::{LockOrderTracker, LockRank};
///
/// let mut tracker = LockOrderTracker::new();
/// tracker.enter(LockRank::new(10, "account")).unwrap();
/// tracker.enter(LockRank::new(20, "ledger")).unwrap();
/// assert_eq!(tracker.held_ranks().len(), 2);
/// ```
#[derive(Debug, Default, Clone)]
pub struct LockOrderTracker {
    held: Vec<LockRank>,
}

impl LockOrderTracker {
    /// Crea un rastreador vacío.
    ///
    /// Complejidad: O(1).
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra la entrada a un lock si respeta el orden total.
    ///
    /// Complejidad: O(1), porque solo compara contra el último rango máximo
    /// sostenido por este modelo.
    pub fn enter(&mut self, requested: LockRank) -> Result<(), LockOrderViolation> {
        if let Some(held) = self.held.iter().max().copied() {
            if requested < held {
                return Err(LockOrderViolation { held, requested });
            }
        }

        self.held.push(requested);
        Ok(())
    }

    /// Libera la última ocurrencia de `rank` registrada.
    ///
    /// Complejidad: O(n), por búsqueda en la pila educativa.
    pub fn exit(&mut self, rank: LockRank) -> Option<LockRank> {
        let position = self.held.iter().rposition(|held| *held == rank)?;
        Some(self.held.remove(position))
    }

    /// Devuelve los locks sostenidos en orden de adquisición.
    ///
    /// Complejidad: O(n).
    pub fn held_ranks(&self) -> Vec<LockRank> {
        self.held.clone()
    }
}

/// Grafo wait-for para detectar ciclos de espera.
///
/// Un borde `A -> B` significa: A espera a B.
///
/// # Examples
///
/// ```
/// use rust_concurrency::deadlocks::WaitForGraph;
///
/// let mut graph = WaitForGraph::new();
/// graph.add_wait("a", "b");
/// graph.add_wait("b", "a");
/// assert!(graph.has_cycle());
/// ```
#[derive(Debug, Default, Clone)]
pub struct WaitForGraph {
    edges: BTreeMap<String, BTreeSet<String>>,
}

impl WaitForGraph {
    /// Crea un grafo vacío.
    ///
    /// Complejidad: O(1).
    pub fn new() -> Self {
        Self::default()
    }

    /// Agrega un borde de espera.
    ///
    /// Complejidad: O(log n).
    pub fn add_wait(&mut self, waiter: impl Into<String>, owner: impl Into<String>) {
        self.edges
            .entry(waiter.into())
            .or_default()
            .insert(owner.into());
    }

    /// Indica si el grafo contiene un ciclo.
    ///
    /// Complejidad: O(V + E), ignorando el costo logarítmico de los mapas.
    pub fn has_cycle(&self) -> bool {
        self.cycle_path().is_some()
    }

    /// Devuelve un ciclo si existe.
    ///
    /// Complejidad: O(V + E), ignorando el costo logarítmico de los mapas.
    pub fn cycle_path(&self) -> Option<Vec<String>> {
        let mut visited = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut stack = Vec::new();

        for node in self.nodes() {
            if !visited.contains(&node) {
                if let Some(cycle) = self.visit(&node, &mut visited, &mut active, &mut stack) {
                    return Some(cycle);
                }
            }
        }

        None
    }

    fn nodes(&self) -> BTreeSet<String> {
        let mut nodes = BTreeSet::new();
        for (waiter, owners) in &self.edges {
            nodes.insert(waiter.clone());
            nodes.extend(owners.iter().cloned());
        }
        nodes
    }

    fn visit(
        &self,
        node: &str,
        visited: &mut BTreeSet<String>,
        active: &mut BTreeSet<String>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        active.insert(node.to_string());
        stack.push(node.to_string());

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.visit(neighbor, visited, active, stack) {
                        return Some(cycle);
                    }
                } else if active.contains(neighbor) {
                    let start = stack.iter().position(|entry| entry == neighbor)?;
                    let mut cycle = stack[start..].to_vec();
                    cycle.push(neighbor.clone());
                    return Some(cycle);
                }
            }
        }

        active.remove(node);
        stack.pop();
        None
    }
}

/// Error de transferencia educativa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferError {
    /// La cuenta no existe.
    UnknownAccount,
    /// El saldo no alcanza.
    InsufficientFunds,
    /// La cuenta origen y destino son iguales.
    SameAccount,
}

/// Cuentas protegidas por locks, con transferencia en orden total.
///
/// # Examples
///
/// ```
/// use rust_concurrency::deadlocks::BankAccounts;
///
/// let accounts = BankAccounts::new([10, 0]);
/// accounts.transfer_ordered(0, 1, 5).unwrap();
/// assert_eq!(accounts.balance(0), 5);
/// assert_eq!(accounts.balance(1), 5);
/// ```
pub struct BankAccounts {
    balances: Vec<Mutex<i64>>,
    last_lock_order: Mutex<Vec<usize>>,
}

impl BankAccounts {
    /// Crea cuentas con saldos iniciales.
    ///
    /// Complejidad: O(n).
    pub fn new<const N: usize>(balances: [i64; N]) -> Self {
        Self {
            balances: balances.into_iter().map(Mutex::new).collect(),
            last_lock_order: Mutex::new(Vec::new()),
        }
    }

    /// Transfiere `amount` adquiriendo locks por índice ascendente.
    ///
    /// Complejidad: O(1) para dos cuentas, además del costo de adquisición de
    /// locks.
    pub fn transfer_ordered(
        &self,
        from: usize,
        to: usize,
        amount: i64,
    ) -> Result<(), TransferError> {
        if from == to {
            return Err(TransferError::SameAccount);
        }
        if from >= self.balances.len() || to >= self.balances.len() {
            return Err(TransferError::UnknownAccount);
        }

        let first = from.min(to);
        let second = from.max(to);

        let mut first_guard = self.balances[first].lock().unwrap();
        let mut second_guard = self.balances[second].lock().unwrap();

        *self.last_lock_order.lock().unwrap() = vec![first, second];

        if from == first {
            if *first_guard < amount {
                return Err(TransferError::InsufficientFunds);
            }
            *first_guard -= amount;
            *second_guard += amount;
        } else {
            if *second_guard < amount {
                return Err(TransferError::InsufficientFunds);
            }
            *second_guard -= amount;
            *first_guard += amount;
        }

        Ok(())
    }

    /// Lee el saldo de una cuenta.
    ///
    /// Complejidad: O(1).
    pub fn balance(&self, account: usize) -> i64 {
        *self.balances[account].lock().unwrap()
    }

    /// Devuelve el último orden de adquisición de locks.
    ///
    /// Complejidad: O(1), con copia de dos índices en el caso normal.
    pub fn last_lock_order(&self) -> Vec<usize> {
        self.last_lock_order.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{LockOrderTracker, LockRank, WaitForGraph};

    #[test]
    fn empty_wait_for_graph_has_no_cycle() {
        assert!(!WaitForGraph::new().has_cycle());
    }

    #[test]
    fn equal_rank_is_allowed_for_reentrant_modeling() {
        let mut tracker = LockOrderTracker::new();
        let rank = LockRank::new(10, "resource");

        tracker.enter(rank).unwrap();
        tracker.enter(rank).unwrap();

        assert_eq!(tracker.held_ranks(), vec![rank, rank]);
    }
}
