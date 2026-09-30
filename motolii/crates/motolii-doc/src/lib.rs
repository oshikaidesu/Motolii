//! The read side of a Motolii work: values, time, evaluation and the stored-work model.
//!
//! This crate opens a saved work and answers "what is the value of this property at
//! this time?". It defines the types (`core`), the evaluator for keyframes, curves and
//! expressions (`eval`), the entity store and its read-only views (`store`: `StoreView`,
//! `Recording`) and path geometry (`vector`). It never changes a work and never draws:
//! changing is `motolii-edit`, drawing is `motolii-render`.
pub mod core;
pub mod eval;
pub mod store;
pub mod vector;

/// 家の中の道は `crate::doc::…` のまま。crate が割れても文は変えない。
pub mod doc {
    pub use crate::{core, eval, store, vector};
}
