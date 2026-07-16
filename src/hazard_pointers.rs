//! Hazard pointers.
//!
//! Objetivo de aprendizaje: entender reclamación segura de memoria en
//! estructuras concurrentes, protección de nodos, retiros diferidos y costos de
//! escaneo.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

/// Identificador estable de un participante/hilo.
///
/// # Examples
///
/// ```
/// use rust_concurrency::hazard_pointers::ParticipantId;
///
/// let participant = ParticipantId::new(7);
/// assert_eq!(participant.get(), 7);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticipantId(usize);

impl ParticipantId {
    /// Crea un identificador de participante.
    ///
    /// Complejidad: O(1).
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Devuelve el identificador numérico.
    ///
    /// Complejidad: O(1).
    pub fn get(self) -> usize {
        self.0
    }
}

/// Identificador estable de un nodo retirado o protegido.
///
/// # Examples
///
/// ```
/// use rust_concurrency::hazard_pointers::NodeId;
///
/// let node = NodeId::new(42);
/// assert_eq!(node.get(), 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    /// Crea un identificador de nodo.
    ///
    /// Complejidad: O(1).
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Devuelve el identificador numérico.
    ///
    /// Complejidad: O(1).
    pub fn get(self) -> usize {
        self.0
    }
}

/// Nodo retirado que aún no necesariamente puede reclamarse.
///
/// # Examples
///
/// ```
/// use rust_concurrency::hazard_pointers::{NodeId, RetiredNode};
///
/// let retired = RetiredNode::new(NodeId::new(1), "payload");
/// assert_eq!(retired.id(), NodeId::new(1));
/// assert_eq!(retired.payload(), "payload");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredNode {
    id: NodeId,
    payload: String,
}

impl RetiredNode {
    /// Crea un nodo retirado con payload descriptivo.
    ///
    /// Complejidad: O(1) más la copia del texto.
    pub fn new(id: NodeId, payload: impl Into<String>) -> Self {
        Self {
            id,
            payload: payload.into(),
        }
    }

    /// Devuelve el identificador del nodo.
    ///
    /// Complejidad: O(1).
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Devuelve el payload descriptivo.
    ///
    /// Complejidad: O(1).
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

/// Resultado de un escaneo de hazard pointers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    /// Número de nodos protegidos observados durante el escaneo.
    pub scanned_hazards: usize,
    /// Nodos reclamados en este escaneo.
    pub reclaimed: Vec<NodeId>,
    /// Nodos que siguieron retirados por estar protegidos.
    pub delayed: Vec<NodeId>,
}

#[derive(Debug, Default)]
struct HazardState {
    protected: BTreeMap<ParticipantId, NodeId>,
    retired: Vec<RetiredNode>,
    reclaimed: Vec<NodeId>,
}

/// Dominio educativo de hazard pointers.
///
/// Un dominio registra qué nodo protege cada participante, qué nodos fueron
/// retirados, y cuándo un escaneo puede reclamarlos.
///
/// # Examples
///
/// ```
/// use rust_concurrency::hazard_pointers::{
///     HazardDomain, NodeId, ParticipantId, RetiredNode,
/// };
///
/// let domain = HazardDomain::new(4);
/// let participant = ParticipantId::new(1);
/// let node = NodeId::new(10);
///
/// domain.protect(participant, node);
/// domain.retire(RetiredNode::new(node, "payload"));
/// assert_eq!(domain.scan().delayed, vec![node]);
/// domain.clear(participant);
/// assert_eq!(domain.scan().reclaimed, vec![node]);
/// ```
pub struct HazardDomain {
    retire_scan_threshold: usize,
    state: Mutex<HazardState>,
}

impl HazardDomain {
    /// Crea un dominio con umbral de escaneo.
    ///
    /// Complejidad: O(1).
    pub fn new(retire_scan_threshold: usize) -> Self {
        assert!(
            retire_scan_threshold > 0,
            "retire_scan_threshold debe ser mayor que cero"
        );

        Self {
            retire_scan_threshold,
            state: Mutex::new(HazardState::default()),
        }
    }

    /// Protege `node` para `participant`.
    ///
    /// Complejidad: O(log p), donde `p` es el número de participantes.
    pub fn protect(&self, participant: ParticipantId, node: NodeId) {
        self.state
            .lock()
            .unwrap()
            .protected
            .insert(participant, node);
    }

    /// Limpia el hazard pointer de `participant`.
    ///
    /// Complejidad: O(log p).
    pub fn clear(&self, participant: ParticipantId) {
        self.state.lock().unwrap().protected.remove(&participant);
    }

    /// Indica si un nodo está protegido por cualquier participante.
    ///
    /// Complejidad: O(p).
    pub fn is_protected(&self, node: NodeId) -> bool {
        self.state
            .lock()
            .unwrap()
            .protected
            .values()
            .any(|protected| *protected == node)
    }

    /// Devuelve nodos protegidos, ordenados y sin duplicados.
    ///
    /// Complejidad: O(p log p).
    pub fn protected_nodes(&self) -> Vec<NodeId> {
        self.protected_set().into_iter().collect()
    }

    /// Retira un nodo. Si se alcanza el umbral, escanea automáticamente.
    ///
    /// Complejidad: O(1) para agregar; O(r + p log p) si dispara escaneo.
    pub fn retire(&self, node: RetiredNode) -> Option<ScanReport> {
        let should_scan = {
            let mut state = self.state.lock().unwrap();
            state.retired.push(node);
            state.retired.len() >= self.retire_scan_threshold
        };

        should_scan.then(|| self.scan())
    }

    /// Escanea hazards activos y reclama nodos no protegidos.
    ///
    /// Complejidad: O(r + p log p).
    pub fn scan(&self) -> ScanReport {
        let mut state = self.state.lock().unwrap();
        let hazards = state.protected.values().copied().collect::<BTreeSet<_>>();
        let scanned_hazards = hazards.len();
        let mut delayed_nodes = Vec::new();
        let mut reclaimed_nodes = Vec::new();
        let mut still_retired = Vec::new();

        for retired in state.retired.drain(..) {
            if hazards.contains(&retired.id) {
                delayed_nodes.push(retired.id);
                still_retired.push(retired);
            } else {
                reclaimed_nodes.push(retired.id);
            }
        }

        state.retired = still_retired;
        state.reclaimed.extend(reclaimed_nodes.iter().copied());

        ScanReport {
            scanned_hazards,
            reclaimed: reclaimed_nodes,
            delayed: delayed_nodes,
        }
    }

    /// Devuelve los nodos todavía retirados.
    ///
    /// Complejidad: O(r).
    pub fn retired_nodes(&self) -> Vec<NodeId> {
        self.state
            .lock()
            .unwrap()
            .retired
            .iter()
            .map(RetiredNode::id)
            .collect()
    }

    /// Devuelve los nodos reclamados históricamente.
    ///
    /// Complejidad: O(n).
    pub fn reclaimed_nodes(&self) -> Vec<NodeId> {
        self.state.lock().unwrap().reclaimed.clone()
    }

    fn protected_set(&self) -> BTreeSet<NodeId> {
        self.state
            .lock()
            .unwrap()
            .protected
            .values()
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{HazardDomain, NodeId, ParticipantId, RetiredNode};

    #[test]
    fn threshold_must_be_positive() {
        let result = std::panic::catch_unwind(|| HazardDomain::new(0));

        assert!(result.is_err());
    }

    #[test]
    fn participant_can_replace_protected_node() {
        let domain = HazardDomain::new(8);
        let participant = ParticipantId::new(1);

        domain.protect(participant, NodeId::new(1));
        domain.protect(participant, NodeId::new(2));

        assert_eq!(domain.protected_nodes(), vec![NodeId::new(2)]);
    }

    #[test]
    fn reclaimed_history_accumulates() {
        let domain = HazardDomain::new(8);

        domain.retire(RetiredNode::new(NodeId::new(1), "a"));
        domain.retire(RetiredNode::new(NodeId::new(2), "b"));
        domain.scan();

        assert_eq!(
            domain.reclaimed_nodes(),
            vec![NodeId::new(1), NodeId::new(2)]
        );
    }
}
