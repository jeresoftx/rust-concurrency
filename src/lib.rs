//! Curso de concurrencia en Rust para Jeresoft Academy.
//!
//! Este crate acompaña el curso `rust-concurrency`. Cada módulo representa una
//! primitiva o técnica canónica del curso y existe para enseñar invariantes,
//! garantías, modos de falla y tradeoffs. Las implementaciones completas se
//! agregan capítulo por capítulo.

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
