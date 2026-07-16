//! Curso de concurrencia en Rust para Jeresoft Academy.
//!
//! Este crate acompana el curso `rust-concurrency`. Cada modulo representa una
//! primitiva o tecnica canonica del curso y existe para ensenar invariantes,
//! garantias, modos de falla y tradeoffs. Las implementaciones completas se
//! agregan capitulo por capitulo.

pub mod arc;
pub mod atomics;
pub mod channels;
pub mod deadlocks;
pub mod epoch_gc;
pub mod hazard_pointers;
pub mod lock_free;
pub mod memory_ordering;
pub mod mutex;
pub mod rwlock;
