//! Public rule scheduling result types and errors.

use crate::ir::PredicateSymbol;
use std::collections::BTreeSet;

/// Rule-level stratification of a finite Datalog program.
///
/// Dependencies connect a producer to a consumer only when the producer's
/// complete head pattern can unify with a body atom. EDB facts are available
/// before stratum zero. Rule indices refer to the analyzed program's rule order.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stratification {
    /// Strata in evaluation order; close each to a fixed point before advancing.
    pub strata: Vec<Stratum>,
    /// One stratum index for each rule, in program order.
    pub rule_stratum: Vec<usize>,
}

impl Stratification {
    /// Return the stratum of a rule, or `None` for an out-of-range rule index.
    #[must_use]
    pub fn stratum_of_rule(&self, rule_index: usize) -> Option<usize> {
        self.rule_stratum.get(rule_index).copied()
    }
}

/// A set of rules evaluated together to a fixed point.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stratum {
    /// Indices into the analyzed program's rule list.
    pub rules: BTreeSet<usize>,
}

/// Stratification failure.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StratificationError {
    /// A cycle of compatible rule dependencies contains negation or aggregation.
    UnstratifiableCycle {
        /// Predicates of the rules in the offending component, sorted and unique.
        predicates: Vec<PredicateSymbol>,
    },
}
