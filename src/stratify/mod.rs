//! Stratification analysis for Datalog¬ programs.
//!
//! A Datalog¬ program is *stratified* when its predicate
//! dependency graph contains no cycle that passes through a
//! negation edge. Stratification assigns each predicate to a
//! stratum such that every negation edge runs from a lower
//! stratum to a strictly higher one; the evaluator then closes
//! each stratum to a fixed point before moving on to the next, so
//! every negated atom is evaluated against an already-closed fact
//! set.
//!
//! The algorithm:
//!
//! 1. Build the predicate dependency graph — for each rule `H :- B_1, ...,
//!    B_n`, add an edge `B_i.predicate → H.predicate` tagged positive or
//!    negative depending on `B_i`'s polarity.
//! 2. Find strongly connected components (Tarjan).
//! 3. If any SCC contains an internal negation edge, reject with
//!    [`StratificationError::UnstratifiableCycle`].
//! 4. Assign strata by topologically processing SCCs and taking the max stratum
//!    of predecessor SCCs, +1 if the incoming edge is negative.

mod entry;
mod graph;
mod tarjan;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use entry::stratify;
pub use types::{Stratification, StratificationError, Stratum};
