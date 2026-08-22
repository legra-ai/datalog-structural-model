//! Aggregation IR for extension.
//!
//! Datalog with aggregation extends pure Datalog by allowing a
//! body literal that folds a set of inner bindings into a single
//! aggregate value, optionally grouped by a chosen set of
//! variables. The aggregate result is bound to a fresh variable
//! that downstream body literals (and the rule head) can consume.
//!
//! This module defines the IR data types only:
//! [`Aggregator`] (the SQL/SPARQL aggregator vocabulary) and
//! [`AggregateBody`] (an aggregation sub-query). The
//! corresponding body-literal variant is
//! [`Literal::Aggregate`](super::literal::Literal::Aggregate),
//! and rule-level safety analysis is in
//! [`Rule::unsafe_aggregate_variables`](super::rule::Rule::unsafe_aggregate_variables).
//!
//! **Stratification**: an aggregation depends on the *complete*
//! extension of its inner predicates, so each predicate referenced
//! from the aggregate's inner body forms a polarity-bearing
//! dependency edge analogous to a negative literal — those
//! predicates must sit in a strictly lower stratum than the
//! rule's head predicate.

use std::collections::BTreeSet;

use super::literal::Literal;
use super::symbols::VariableSymbol;

/// SQL/SPARQL aggregator vocabulary supported in rule bodies.
///
/// Mirrors the subset called out in the extension deliverable
/// (the rule validation specification):
/// MIN, MAX, SUM, COUNT, AVG. SPARQL's `SAMPLE` and
/// `GROUP_CONCAT` are not in scope for extension — they map onto
/// SPARQL's `Group` operator rather than a Datalog rule body.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Aggregator {
    /// `COUNT(?expr)` — number of distinct `?expr` values that
    /// appear in the inner result set.
    Count,
    /// `SUM(?expr)` — numeric sum of `?expr` values.
    Sum,
    /// `MIN(?expr)` — minimum `?expr` value under SPARQL's
    /// ordering (numeric / lexicographic per term kind).
    Min,
    /// `MAX(?expr)` — maximum `?expr` value under the same
    /// ordering as [`Self::Min`].
    Max,
    /// `AVG(?expr)` — numeric mean of `?expr` values. Undefined
    /// for an empty input.
    Avg,
}

impl Aggregator {
    /// Render the aggregator as its canonical uppercase name.
    /// Used by diagnostics and rule-name maps.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Count => "COUNT",
            Self::Sum => "SUM",
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::Avg => "AVG",
        }
    }
}

/// An aggregation sub-query inside a rule body.
///
/// Semantically equivalent to a SPARQL `Group { ... } over { ... }`
/// projection: evaluate `inner_body` as a sub-query, group the
/// results by `group_by`, fold each group's `input_var` values
/// with `aggregator`, and bind the per-group result to
/// `result_var`. The aggregate then participates in the outer
/// rule body as if `result_var` were produced by a positive
/// atom that ranges over the per-group results.
///
/// # Variable scoping
///
/// `inner_body` is its own scope: positive atoms and `BIND`
/// outputs inside the inner body bind variables that are visible
/// only within the aggregation. The three sets of variables that
/// leak out are `group_by`, `input_var`, and (after the
/// aggregation fires) `result_var`. Concretely:
///
/// - `group_by` and `input_var` MUST be bound by some positive atom or `BIND`
///   inside `inner_body`.
/// - `result_var` MUST be fresh — it must not appear anywhere else in the outer
///   rule (head or body).
///
/// # Inner-body literal kinds
///
/// `inner_body` accepts the same four literal kinds the outer
/// rule body accepts — [`Literal::Positive`], [`Literal::Negative`],
/// [`Literal::Filter`], and [`Literal::Bind`] — under the usual
/// safety rules (negated atoms and filter expressions reference
/// only inner-body-bound variables; `BIND` expressions reference
/// only earlier inner positives / binds and bind a fresh variable).
/// Nested aggregates inside an aggregate's inner body are rejected
/// at validation time as
/// [`super::error::ProgramValidationError::NestedAggregateInInnerBody`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AggregateBody {
    /// The variable that receives the per-group aggregate
    /// result.
    pub result_var: VariableSymbol,
    /// Which aggregator to apply.
    pub aggregator: Aggregator,
    /// The variable whose values are folded into the aggregate
    /// (the "input expression" in SPARQL terms).
    pub input_var: VariableSymbol,
    /// Group-by keys. The aggregator runs once per distinct
    /// tuple of `group_by` values; an empty `group_by`
    /// collapses every inner-body binding into a single group.
    pub group_by: Vec<VariableSymbol>,
    /// Literals forming the aggregation's inner sub-query.
    /// Accepts [`Literal::Positive`], [`Literal::Negative`],
    /// [`Literal::Filter`], and [`Literal::Bind`] under the
    /// per-kind safety rules. Nested
    /// [`Literal::Aggregate`] is rejected at validation time.
    pub inner_body: Vec<Literal>,
}

impl AggregateBody {
    /// Variables bound by the inner body — every variable a
    /// positive inner atom binds, plus every `BIND`'s output
    /// variable. Used by safety analysis to verify that
    /// `input_var` and `group_by` are sourced from the inner
    /// body and that inner negation / filter / bind references
    /// stay within the inner scope.
    #[must_use]
    pub fn inner_bound_variables(&self) -> BTreeSet<VariableSymbol> {
        let mut out = BTreeSet::new();
        for lit in &self.inner_body {
            match lit {
                Literal::Positive(atom) => out.extend(atom.variables()),
                Literal::Bind { var, .. } => {
                    out.insert(var.clone());
                }
                Literal::Negative(_) | Literal::Filter(_) | Literal::Aggregate(_) => {}
            }
        }
        out
    }

    /// Variables this aggregate exposes to the outer rule body:
    /// the freshly-bound `result_var` plus every key the
    /// outer rule groups by. The outer-scope view of an
    /// aggregate body literal.
    #[must_use]
    pub fn exported_variables(&self) -> BTreeSet<VariableSymbol> {
        let mut out: BTreeSet<VariableSymbol> = self.group_by.iter().cloned().collect();
        out.insert(self.result_var.clone());
        out
    }

    /// Returns `true` when the inner body contains a nested
    /// `Literal::Aggregate`. Nested aggregates are rejected as
    /// [`super::error::ProgramValidationError::NestedAggregateInInnerBody`].
    #[must_use]
    pub fn contains_nested_aggregate(&self) -> bool {
        self.inner_body
            .iter()
            .any(|lit| matches!(lit, Literal::Aggregate(_)))
    }
}
