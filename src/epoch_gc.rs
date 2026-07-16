//! Epoch GC.
//!
//! Objetivo de aprendizaje: entender reclamación basada en épocas, secciones
//! críticas, avance global, memoria retirada y comparación con hazard pointers.

use std::collections::BTreeMap;
use std::sync::Mutex;

/// Identificador estable de un participante del dominio.
///
/// # Examples
///
/// ```
/// use rust_concurrency::epoch_gc::ParticipantId;
///
/// let participant = ParticipantId::new(3);
/// assert_eq!(participant.get(), 3);
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

/// Identificador estable de un objeto retirado.
///
/// # Examples
///
/// ```
/// use rust_concurrency::epoch_gc::ObjectId;
///
/// let object = ObjectId::new(42);
/// assert_eq!(object.get(), 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(usize);

impl ObjectId {
    /// Crea un identificador de objeto.
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

/// Marca devuelta cuando un participante entra a una sección crítica.
///
/// En un recolector por épocas real, esta marca representa que el hilo puede
/// leer objetos retirados antes de que el dominio pueda liberarlos.
///
/// # Examples
///
/// ```
/// use rust_concurrency::epoch_gc::{EpochDomain, ParticipantId};
///
/// let domain = EpochDomain::new(2);
/// let pin = domain.pin(ParticipantId::new(1));
/// assert_eq!(pin.epoch(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pin {
    participant: ParticipantId,
    epoch: usize,
}

impl Pin {
    fn new(participant: ParticipantId, epoch: usize) -> Self {
        Self { participant, epoch }
    }

    /// Devuelve el participante asociado a esta marca.
    ///
    /// Complejidad: O(1).
    pub fn participant(self) -> ParticipantId {
        self.participant
    }

    /// Devuelve la época observada al fijar el participante.
    ///
    /// Complejidad: O(1).
    pub fn epoch(self) -> usize {
        self.epoch
    }
}

/// Objeto retirado que aún puede estar retenido por participantes antiguos.
///
/// # Examples
///
/// ```
/// use rust_concurrency::epoch_gc::{ObjectId, RetiredObject};
///
/// let retired = RetiredObject::new(ObjectId::new(5), 2, "nodo viejo");
/// assert_eq!(retired.id(), ObjectId::new(5));
/// assert_eq!(retired.retired_epoch(), 2);
/// assert_eq!(retired.payload(), "nodo viejo");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredObject {
    id: ObjectId,
    retired_epoch: usize,
    payload: String,
}

impl RetiredObject {
    /// Crea un objeto retirado con payload descriptivo.
    ///
    /// Complejidad: O(1) más la copia del texto.
    pub fn new(id: ObjectId, retired_epoch: usize, payload: impl Into<String>) -> Self {
        Self {
            id,
            retired_epoch,
            payload: payload.into(),
        }
    }

    /// Devuelve el identificador del objeto.
    ///
    /// Complejidad: O(1).
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Devuelve la época en que el objeto fue retirado.
    ///
    /// Complejidad: O(1).
    pub fn retired_epoch(&self) -> usize {
        self.retired_epoch
    }

    /// Devuelve el payload descriptivo.
    ///
    /// Complejidad: O(1).
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

/// Resultado de intentar avanzar la época global.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochAdvance {
    previous_epoch: usize,
    current_epoch: usize,
    advanced: bool,
    blocked_by: Vec<ParticipantId>,
}

impl EpochAdvance {
    fn advanced_result(previous_epoch: usize, current_epoch: usize) -> Self {
        Self {
            previous_epoch,
            current_epoch,
            advanced: true,
            blocked_by: Vec::new(),
        }
    }

    fn blocked(current_epoch: usize, blocked_by: Vec<ParticipantId>) -> Self {
        Self {
            previous_epoch: current_epoch,
            current_epoch,
            advanced: false,
            blocked_by,
        }
    }

    /// Indica si la época global avanzó.
    ///
    /// Complejidad: O(1).
    pub fn advanced(&self) -> bool {
        self.advanced
    }

    /// Devuelve la época previa al intento.
    ///
    /// Complejidad: O(1).
    pub fn previous_epoch(&self) -> usize {
        self.previous_epoch
    }

    /// Devuelve la época global después del intento.
    ///
    /// Complejidad: O(1).
    pub fn current_epoch(&self) -> usize {
        self.current_epoch
    }

    /// Devuelve participantes que impidieron el avance.
    ///
    /// Complejidad: O(1).
    pub fn blocked_by(&self) -> &[ParticipantId] {
        &self.blocked_by
    }
}

/// Resultado de escanear objetos retirados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReclaimReport {
    global_epoch: usize,
    reclaimed: Vec<ObjectId>,
    delayed: Vec<ObjectId>,
    blocked_by: Vec<ParticipantId>,
}

impl ReclaimReport {
    /// Devuelve la época global observada durante el escaneo.
    ///
    /// Complejidad: O(1).
    pub fn global_epoch(&self) -> usize {
        self.global_epoch
    }

    /// Devuelve objetos reclamados en este escaneo.
    ///
    /// Complejidad: O(1).
    pub fn reclaimed(&self) -> &[ObjectId] {
        &self.reclaimed
    }

    /// Devuelve objetos que siguieron retenidos.
    ///
    /// Complejidad: O(1).
    pub fn delayed(&self) -> &[ObjectId] {
        &self.delayed
    }

    /// Devuelve participantes que podrían retener memoria retirada.
    ///
    /// Complejidad: O(1).
    pub fn blocked_by(&self) -> &[ParticipantId] {
        &self.blocked_by
    }
}

#[derive(Debug, Default)]
struct EpochState {
    global_epoch: usize,
    pinned: BTreeMap<ParticipantId, usize>,
    retired: Vec<RetiredObject>,
    reclaimed: Vec<ObjectId>,
}

/// Dominio educativo de recolección basada en épocas.
///
/// El dominio registra participantes fijados, objetos retirados y una época
/// global. Esta versión no administra memoria real; representa las invariantes
/// que un crate de bajo nivel necesitaría proteger con punteros y barreras.
///
/// # Examples
///
/// ```
/// use rust_concurrency::epoch_gc::{EpochDomain, ObjectId, ParticipantId};
///
/// let domain = EpochDomain::new(2);
/// let participant = ParticipantId::new(1);
///
/// domain.pin(participant);
/// domain.retire(ObjectId::new(10), "nodo retirado");
/// assert!(domain.try_advance().advanced());
/// assert!(!domain.try_advance().advanced());
///
/// domain.unpin(participant);
/// assert!(domain.try_advance().advanced());
/// assert_eq!(domain.scan().reclaimed(), &[ObjectId::new(10)]);
/// ```
pub struct EpochDomain {
    reclaim_lag: usize,
    state: Mutex<EpochState>,
}

impl EpochDomain {
    /// Crea un dominio con el rezago mínimo antes de reclamar.
    ///
    /// Complejidad: O(1).
    pub fn new(reclaim_lag: usize) -> Self {
        assert!(reclaim_lag > 0, "reclaim_lag debe ser mayor que cero");

        Self {
            reclaim_lag,
            state: Mutex::new(EpochState::default()),
        }
    }

    /// Devuelve la época global actual.
    ///
    /// Complejidad: O(1).
    pub fn global_epoch(&self) -> usize {
        self.state.lock().unwrap().global_epoch
    }

    /// Fija `participant` en la época global actual.
    ///
    /// Complejidad: O(log p), donde `p` es el número de participantes.
    pub fn pin(&self, participant: ParticipantId) -> Pin {
        let mut state = self.state.lock().unwrap();
        let epoch = state.global_epoch;
        state.pinned.insert(participant, epoch);
        Pin::new(participant, epoch)
    }

    /// Marca a `participant` como quiescente.
    ///
    /// Complejidad: O(log p).
    pub fn unpin(&self, participant: ParticipantId) {
        self.state.lock().unwrap().pinned.remove(&participant);
    }

    /// Indica si `participant` está fijado.
    ///
    /// Complejidad: O(log p).
    pub fn is_pinned(&self, participant: ParticipantId) -> bool {
        self.state.lock().unwrap().pinned.contains_key(&participant)
    }

    /// Devuelve participantes fijados en orden estable.
    ///
    /// Complejidad: O(p).
    pub fn pinned_participants(&self) -> Vec<ParticipantId> {
        self.state.lock().unwrap().pinned.keys().copied().collect()
    }

    /// Retira un objeto en la época global actual.
    ///
    /// Complejidad: O(1) más la copia del texto.
    pub fn retire(&self, object: ObjectId, payload: impl Into<String>) -> RetiredObject {
        let mut state = self.state.lock().unwrap();
        let retired = RetiredObject::new(object, state.global_epoch, payload);
        state.retired.push(retired.clone());
        retired
    }

    /// Intenta avanzar la época global.
    ///
    /// Participantes fijados en la época actual todavía permiten un avance:
    /// todos observaron la época que estaba vigente. Si permanecen fijados
    /// después de ese avance, se vuelven participantes estancados y bloquean el
    /// siguiente intento.
    ///
    /// Complejidad: O(p).
    pub fn try_advance(&self) -> EpochAdvance {
        let mut state = self.state.lock().unwrap();
        let blocked_by = stale_participants(&state.pinned, state.global_epoch);

        if blocked_by.is_empty() {
            let previous_epoch = state.global_epoch;
            state.global_epoch += 1;
            EpochAdvance::advanced_result(previous_epoch, state.global_epoch)
        } else {
            EpochAdvance::blocked(state.global_epoch, blocked_by)
        }
    }

    /// Escanea objetos retirados y reclama los que ya son seguros.
    ///
    /// Un objeto es reclamable si cumplió el rezago de épocas y ningún
    /// participante sigue fijado en la época donde ese objeto fue retirado o en
    /// una anterior.
    ///
    /// Complejidad: O(r * p), donde `r` es el número de objetos retirados.
    pub fn scan(&self) -> ReclaimReport {
        let mut state = self.state.lock().unwrap();
        let mut reclaimed = Vec::new();
        let mut delayed = Vec::new();
        let mut still_retired = Vec::new();
        let mut blocked_by = Vec::new();

        let pinned = state.pinned.clone();
        let global_epoch = state.global_epoch;

        for retired in state.retired.drain(..) {
            let object_blockers = object_blockers(&pinned, retired.retired_epoch);
            let epoch_mature = retired.retired_epoch + self.reclaim_lag <= global_epoch;

            if epoch_mature && object_blockers.is_empty() {
                reclaimed.push(retired.id);
            } else {
                delayed.push(retired.id);
                still_retired.push(retired);
                extend_unique(&mut blocked_by, object_blockers);
            }
        }

        state.retired = still_retired;
        state.reclaimed.extend(reclaimed.iter().copied());

        ReclaimReport {
            global_epoch,
            reclaimed,
            delayed,
            blocked_by,
        }
    }

    /// Devuelve objetos todavía retirados.
    ///
    /// Complejidad: O(r).
    pub fn retired_objects(&self) -> Vec<ObjectId> {
        self.state
            .lock()
            .unwrap()
            .retired
            .iter()
            .map(RetiredObject::id)
            .collect()
    }

    /// Devuelve objetos reclamados históricamente.
    ///
    /// Complejidad: O(n).
    pub fn reclaimed_objects(&self) -> Vec<ObjectId> {
        self.state.lock().unwrap().reclaimed.clone()
    }
}

fn stale_participants(
    pinned: &BTreeMap<ParticipantId, usize>,
    global_epoch: usize,
) -> Vec<ParticipantId> {
    pinned
        .iter()
        .filter_map(|(participant, epoch)| (*epoch < global_epoch).then_some(*participant))
        .collect()
}

fn object_blockers(
    pinned: &BTreeMap<ParticipantId, usize>,
    retired_epoch: usize,
) -> Vec<ParticipantId> {
    pinned
        .iter()
        .filter_map(|(participant, epoch)| (*epoch <= retired_epoch).then_some(*participant))
        .collect()
}

fn extend_unique(target: &mut Vec<ParticipantId>, source: Vec<ParticipantId>) {
    for participant in source {
        if !target.contains(&participant) {
            target.push(participant);
        }
    }
}
