//! Datalog [`Rule`] — head atom plus a conjunction of body
//! literals.

use std::collections::BTreeSet;

use super::aggregate::AggregateBody;
use super::atom::Atom;
use super::literal::Literal;
use super::symbols::VariableSymbol;

/// Datalog rule: `head :- body_1, ..., body_n`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "super::serde_input::RuleInput")]
pub struct Rule {
    /// Head atom.
    pub head: Atom,
    /// Body literals. An empty body represents a fact.
    pub body: Vec<Literal>,
}

impl Rule {
    /// Construct a standalone rule only when it is safe and stratifiable.
    /// # Errors
    /// Rejects invalid terms, unsafe variables and recursion through negation.
    pub fn try_new(head: Atom, body: Vec<Literal>) -> Result<Self, super::ProgramValidationError> {
        let rule = Self { head, body };
        // bounded: one authored rule, not a fact relation or graph.
        super::Program {
            rules: vec![rule.clone()],
        }
        .validate()?;
        Ok(rule)
    }

    /// Construct a rule whose body contains only positive literals.
    ///
    /// Convenience for the common case — callers that do not need
    /// negation never have to name [`Literal`].
    #[must_use]
    pub fn positive(head: Atom, body: Vec<Atom>) -> Self {
        Self {
            head,
            body: body.into_iter().map(Literal::Positive).collect(),
        }
    }

    /// Construct a rule with an explicit literal body.
    #[must_use]
    pub fn new(head: Atom, body: Vec<Literal>) -> Self {
        Self { head, body }
    }

    /// Returns true when the rule satisfies Datalog¬ safety.
    ///
    /// A rule is safe when:
    ///
    /// 1. Every variable appearing in the head is bound by some positive body
    ///    atom or `Bind`.
    /// 2. Every variable appearing in a **negative** body literal is bound by
    ///    some positive body atom or `Bind` (range restriction —
    ///    negation-as-failure requires the negated atom to be ground by the
    ///    time it is evaluated).
    /// 3. Every variable appearing inside a [`Literal::Filter`] is bound by
    ///    some positive body atom or `Bind` — filters guard candidate bindings,
    ///    they do not produce them.
    /// 4. Every variable inside a [`Literal::Bind`] expression is bound by an
    ///    **earlier** positive body atom or `Bind`, and the `Bind`'s output
    ///    variable is fresh (not bound by any earlier positive atom or `Bind`).
    #[must_use]
    pub fn is_safe(&self) -> bool {
        let unsafe_head = self.unsafe_head_variables();
        let unsafe_neg = self.unsafe_negative_variables();
        let unsafe_filter = self.unsafe_filter_variables();
        let unsafe_bind = self.unsafe_bind_variables();
        let unsafe_aggregate = self.unsafe_aggregate_variables();
        unsafe_head.is_empty()
            && unsafe_neg.is_empty()
            && unsafe_filter.is_empty()
            && unsafe_bind.is_empty()
            && unsafe_aggregate.is_empty()
    }

    #[must_use]
    pub(super) fn unsafe_head_variables(&self) -> Vec<VariableSymbol> {
        let head_vars = self.head.variables();
        let bound = self.bound_body_variables();
        head_vars.difference(&bound).cloned().collect()
    }

    #[must_use]
    pub(super) fn unsafe_negative_variables(&self) -> Vec<VariableSymbol> {
        let bound = self.bound_body_variables();
        self.body
            .iter()
            .filter_map(|lit| match lit {
                Literal::Negative(atom) => Some(atom),
                Literal::Positive(_)
                | Literal::Filter(_)
                | Literal::Bind { .. }
                | Literal::Aggregate(_) => None,
            })
            .flat_map(Atom::variables)
            .filter(|v| !bound.contains(v))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[must_use]
    pub(super) fn unsafe_filter_variables(&self) -> Vec<VariableSymbol> {
        let bound = self.bound_body_variables();
        self.body
            .iter()
            .filter_map(|lit| match lit {
                Literal::Filter(filter) => Some(filter.variables()),
                Literal::Positive(_)
                | Literal::Negative(_)
                | Literal::Bind { .. }
                | Literal::Aggregate(_) => None,
            })
            .flatten()
            .filter(|v| !bound.contains(v))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Position-aware safety check for `BIND(expr AS ?var)`.
    ///
    /// Walks the body in order, building up the set of variables
    /// bound by every positive atom and prior `Bind`. Each
    /// `Bind` is inspected when first encountered:
    ///
    /// * every variable inside `expr` must already be in the bound-set (i.e.
    ///   produced by an earlier positive atom or `Bind`);
    /// * the `Bind`'s output variable must NOT already be in the bound-set —
    ///   `BIND` introduces *fresh* variables.
    ///
    /// Both kinds of failure surface the offending variable for
    /// diagnostics. Positive atoms anywhere in the body
    /// contribute their variables to the bound-set up front (the
    /// evaluator joins atoms in order, but positive-atom variables
    /// are present at the binding-set level regardless of position
    /// — so this only constrains the `Bind`-related ordering).
    #[must_use]
    pub(super) fn unsafe_bind_variables(&self) -> Vec<VariableSymbol> {
        let mut bound = self.positive_atom_variables();
        let mut errors: BTreeSet<VariableSymbol> = BTreeSet::new();
        for lit in &self.body {
            let Literal::Bind { var, expr } = lit else {
                continue;
            };
            errors.extend(expr.variables().into_iter().filter(|v| !bound.contains(v)));
            if bound.contains(var) {
                errors.insert(var.clone());
            }
            bound.insert(var.clone());
        }
        errors.into_iter().collect()
    }

    /// Safety check for [`Literal::Aggregate`].
    ///
    /// Each aggregate body literal must satisfy:
    ///
    /// 1. `input_var` and every `group_by` key are bound by some inner-body
    ///    positive atom or `BIND` output (the aggregator can only fold values
    ///    that the inner sub-query actually produces).
    /// 2. `result_var` is fresh — it does not appear in any other body
    ///    literal's variable set or in the rule's head before this aggregate
    ///    fires. (A later aggregate's `result_var` may shadow an earlier
    ///    aggregate's inner-only variable, since inner scopes don't leak.)
    /// 3. Inner-body literals satisfy the same per-kind safety rules the outer
    ///    rule body enforces — negative literals and filters reference only
    ///    inner-positive / inner-`BIND` variables; inner `BIND` expressions
    ///    reference only earlier inner positives / binds and the `BIND` output
    ///    variable is fresh inside the inner scope.
    ///
    /// Nested aggregates are reported separately by
    /// [`Self::has_nested_aggregate`] and surface as
    /// [`super::error::ProgramValidationError::NestedAggregateInInnerBody`].
    #[must_use]
    pub(super) fn unsafe_aggregate_variables(&self) -> Vec<VariableSymbol> {
        let mut already_bound = self.positive_atom_variables();
        for lit in &self.body {
            if let Literal::Bind { var, .. } = lit {
                already_bound.insert(var.clone());
            }
        }
        let mut errors: BTreeSet<VariableSymbol> = BTreeSet::new();
        for lit in &self.body {
            let Literal::Aggregate(agg) = lit else {
                continue;
            };
            let inner_bound = agg.inner_bound_variables();
            if !inner_bound.contains(&agg.input_var) {
                errors.insert(agg.input_var.clone());
            }
            for key in &agg.group_by {
                if !inner_bound.contains(key) {
                    errors.insert(key.clone());
                }
            }
            if already_bound.contains(&agg.result_var) {
                errors.insert(agg.result_var.clone());
            }
            errors.extend(unsafe_inner_body_variables(agg, &inner_bound));
            // Once this aggregate fires, its result_var becomes
            // visible to subsequent body literals.
            already_bound.insert(agg.result_var.clone());
        }
        errors.into_iter().collect()
    }

    /// Returns `true` when any aggregate body literal's `inner_body`
    /// contains a nested [`Literal::Aggregate`]. Reported by
    /// [`super::program::Program::validate`] as
    /// [`super::error::ProgramValidationError::NestedAggregateInInnerBody`].
    #[must_use]
    pub(super) fn has_nested_aggregate_in_inner_body(&self) -> bool {
        self.body.iter().any(|lit| match lit {
            Literal::Aggregate(agg) => agg.contains_nested_aggregate(),
            _ => false,
        })
    }

    /// Variables bound by the time the rule head fires: positive
    /// body atoms, every `Bind` output variable, and every
    /// `Aggregate` body literal's exported variables
    /// (`result_var` and `group_by` — the inner-only variables
    /// stay scoped inside the aggregate). Used by head,
    /// negation, and filter safety checks.
    #[must_use]
    fn bound_body_variables(&self) -> BTreeSet<VariableSymbol> {
        let mut out = self.positive_atom_variables();
        for lit in &self.body {
            match lit {
                Literal::Bind { var, .. } => {
                    out.insert(var.clone());
                }
                Literal::Aggregate(agg) => {
                    out.extend(agg.exported_variables());
                }
                Literal::Positive(_) | Literal::Negative(_) | Literal::Filter(_) => {}
            }
        }
        out
    }

    /// Variables produced by positive body atoms only (no
    /// `Bind` output variables, no aggregate exports). Used by
    /// `Bind` safety as the initial bound-set before walking
    /// the body.
    #[must_use]
    fn positive_atom_variables(&self) -> BTreeSet<VariableSymbol> {
        self.body
            .iter()
            .filter_map(|lit| match lit {
                Literal::Positive(atom) => Some(atom.variables()),
                Literal::Negative(_)
                | Literal::Filter(_)
                | Literal::Bind { .. }
                | Literal::Aggregate(_) => None,
            })
            .flatten()
            .collect()
    }
}

/// Validate variable safety inside an aggregate's inner body.
fn unsafe_inner_body_variables(
    agg: &AggregateBody,
    inner_bound: &BTreeSet<VariableSymbol>,
) -> BTreeSet<VariableSymbol> {
    let mut errors: BTreeSet<VariableSymbol> = BTreeSet::new();
    // Inner-Bind position-aware safety. Start the bound set
    // from inner-positive variables; walk inner `BIND`s in
    // order, checking expression references and freshness, and
    // extending the bound set with each output.
    let mut inner_bind_bound: BTreeSet<VariableSymbol> = agg
        .inner_body
        .iter()
        .filter_map(|lit| match lit {
            Literal::Positive(atom) => Some(atom.variables()),
            _ => None,
        })
        .flatten()
        .collect();
    for lit in &agg.inner_body {
        match lit {
            Literal::Negative(atom) => {
                for v in atom.variables() {
                    if !inner_bound.contains(&v) {
                        errors.insert(v);
                    }
                }
            }
            Literal::Filter(filter) => {
                for v in filter.variables() {
                    if !inner_bound.contains(&v) {
                        errors.insert(v);
                    }
                }
            }
            Literal::Bind { var, expr } => {
                for v in expr.variables() {
                    if !inner_bind_bound.contains(&v) {
                        errors.insert(v);
                    }
                }
                if inner_bind_bound.contains(var) {
                    errors.insert(var.clone());
                }
                inner_bind_bound.insert(var.clone());
            }
            Literal::Positive(_) | Literal::Aggregate(_) => {}
        }
    }
    errors
}
