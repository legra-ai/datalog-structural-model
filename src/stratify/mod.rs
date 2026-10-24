//! Rule-level stratification over complete tuple patterns.
//!
//! A producer precedes every consumer whose body can match its head. Negative
//! and aggregate dependencies require a strictly earlier stratum. A component
//! containing a strict dependency is rejected. Pure EDB inputs are closed before
//! rule evaluation starts. The dependency traversal uses a heap-backed stack.

mod entry;
mod graph;
mod pattern;
mod tarjan;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use entry::stratify;
pub use types::{Stratification, StratificationError, Stratum};
