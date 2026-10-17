//! Body literal — positive/negated atom, filter, `BIND`, or
//! aggregation.

use super::aggregate::AggregateBody;
use super::atom::Atom;
use super::bind::BindExpr;
use super::filter::BuiltinFilter;
use super::symbols::VariableSymbol;

/// Datalog body literal — a positive or negated atom, a built-in
/// filter expression, or a built-in `BIND(expr AS ?var)` form.
///
/// Rule bodies in Datalog¬ are conjunctions of literals. Positive
/// literals must match facts in the database; negative literals
/// must **not** match. Negation semantics follow the stratified
/// negation-as-failure convention: every negated atom references
/// only predicates in strictly lower strata, so the set of facts
/// the negation is evaluated against is already closed by the time
/// the rule fires.
///
/// `Filter` literals carry a [`BuiltinFilter`] expression that
/// guards the current candidate binding — the evaluator drops a
/// binding whenever the filter does not evaluate to `true`.
///
/// `Bind` literals carry a [`BindExpr`] expression and a fresh
/// variable name; the evaluator extends the current candidate
/// binding with `var → eval(expr, binding)` and the new variable
/// is then visible to all subsequent body literals (including
/// further `Bind`s, `Filter`s, the rule head, and SPARQL-style
/// constraints over computed values). Filter and bind literals
/// participate in safety analysis but contribute no edges to the
/// predicate dependency graph used by stratification.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Literal {
    /// Positive literal — the body atom must match a fact.
    Positive(Atom),
    /// Negative literal — the body atom must **not** match any
    /// fact.
    Negative(Atom),
    /// Built-in filter expression evaluated against the current
    /// candidate binding.
    Filter(BuiltinFilter),
    /// `BIND(expr AS ?var)` — extend the current candidate
    /// binding with a computed value. `var` must be fresh
    /// (untouched by any earlier positive atom or `Bind`); every
    /// variable inside `expr` must be bound by an earlier
    /// positive atom or `Bind`.
    Bind {
        /// The fresh variable produced by this bind.
        var: VariableSymbol,
        /// The expression to compute. Evaluated against the
        /// current candidate binding.
        expr: BindExpr,
    },
    /// Aggregation sub-query — fold the inner body into a
    /// per-group aggregate result that the outer body sees as a
    /// freshly-bound variable. See [`AggregateBody`] for the
    /// scoping and inner-body restrictions; the evaluator wiring
    /// is a deferred extension follow-up.
    Aggregate(AggregateBody),
}

impl Literal {
    /// Access the underlying atom, when this literal carries one.
    ///
    /// Returns `None` for [`Literal::Filter`], [`Literal::Bind`],
    /// and [`Literal::Aggregate`] — none of which carries a
    /// single fact-matching atom in its top-level position.
    /// (An aggregate's inner literals live inside
    /// [`AggregateBody::inner_body`].)
    #[must_use]
    pub fn atom(&self) -> Option<&Atom> {
        match self {
            Self::Positive(atom) | Self::Negative(atom) => Some(atom),
            Self::Filter(_) | Self::Bind { .. } | Self::Aggregate(_) => None,
        }
    }

    /// True when the literal is positive.
    #[must_use]
    pub fn is_positive(&self) -> bool {
        matches!(self, Self::Positive(_))
    }

    /// True when the literal is negated.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        matches!(self, Self::Negative(_))
    }

    /// True when the literal is a [`BuiltinFilter`] guard.
    #[must_use]
    pub fn is_filter(&self) -> bool {
        matches!(self, Self::Filter(_))
    }

    /// True when the literal is a `BIND(expr AS ?var)` form.
    #[must_use]
    pub fn is_bind(&self) -> bool {
        matches!(self, Self::Bind { .. })
    }

    /// True when the literal is an aggregation sub-query.
    #[must_use]
    pub fn is_aggregate(&self) -> bool {
        matches!(self, Self::Aggregate(_))
    }
}
