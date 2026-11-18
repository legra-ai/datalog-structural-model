//! Termination analysis: weak acyclicity of the position graph.
//!
//! The analysis inspects only the finite authored program. It never reads,
//! derives or counts facts; its cost is bounded by program size.

mod criterion;
mod edge;
mod graph;
mod sources;
mod verdict;
mod violation;

pub use criterion::{LawVersion, TerminationCriterion};
pub use edge::{ArgumentPosition, InventionKind, SpecialEdge};
pub use verdict::WeakAcyclicity;
pub use violation::TerminationViolation;
