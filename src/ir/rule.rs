//! Datalog [`Rule`] — a [`RuleHead`] plus a conjunction of body
//! literals.

use std::collections::BTreeSet;

use super::aggregate::AggregateBody;
use super::atom::Atom;
use super::head::{HeadForm, RuleHead};
use super::literal::Literal;
use super::symbols::VariableSymbol;
use crate::position::{LiteralIndex, LiteralPath};

/// A body literal and the variables that make it unsafe.
pub(super) struct LiteralFault {
    pub(super) literal: LiteralPath,
    pub(super) variables: Vec<VariableSymbol>,
}

impl LiteralFault {
    fn new(literal: LiteralPath, variables: Vec<VariableSymbol>) -> Self {
        Self { literal, variables }
    }
}

/// Sorted variables of `vars` missing from `bound`.
fn unbound(
    vars: BTreeSet<VariableSymbol>,
    bound: &BTreeSet<VariableSymbol>,
) -> Vec<VariableSymbol> {
    vars.into_iter().filter(|v| !bound.contains(v)).collect()
}

/// Datalog rule: `head :- body_1, ..., body_n`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "super::serde_input::RuleInput")]
pub struct Rule {
    /// What the rule concludes.
    pub head: RuleHead,
    /// Body literals. An empty body represents a fact.
    pub body: Vec<Literal>,
}

impl Rule {
    /// Construct a standalone rule only when it is safe and stratifiable.
    /// # Errors
    /// Rejects invalid terms, unsafe variables and recursion through negation.
    pub fn try_new(
        head: impl Into<RuleHead>,
        body: Vec<Literal>,
    ) -> Result<Self, super::ProgramValidationError> {
        let rule = Self::new(head, body);
        // bounded: one authored rule, not a fact relation or graph.
        super::Program {
            rules: vec![rule.clone()],
        }
        .validate()?;
        Ok(rule)
    }

    /// Number of body literals, counting each aggregate's inner body.
    #[must_use]
    pub fn literal_count(&self) -> usize {
        Literal::count(&self.body)
    }

    /// Construct a rule whose body contains only positive literals.
    ///
    /// Convenience for the common case — callers that do not need
    /// negation never have to name [`Literal`].
    #[must_use]
    pub fn positive(head: impl Into<RuleHead>, body: Vec<Atom>) -> Self {
        Self {
            head: head.into(),
            body: body.into_iter().map(Literal::Positive).collect(),
        }
    }

    /// Construct a rule with an explicit literal body.
    #[must_use]
    pub fn new(head: impl Into<RuleHead>, body: Vec<Literal>) -> Self {
        Self {
            head: head.into(),
            body,
        }
    }

    /// The logical form of the head; see [`HeadForm`].
    #[must_use]
    pub fn head_form(&self) -> HeadForm {
        self.head.form()
    }

    /// Returns true when the rule satisfies Datalog¬ safety.
    ///
    /// A rule is safe when:
    ///
    /// 1. Every variable appearing in a head atom is bound by some positive
    ///    body atom, `Bind`, or aggregate export, unless the head quantifies it
    ///    existentially. Existential variables are unused nowhere and bound
    ///    nowhere in the body.
    /// 2. Every variable appearing in a **negative** body literal is bound by
    ///    some positive body atom, `Bind`, or aggregate export (range restriction —
    ///    negation-as-failure requires the negated atom to be ground by the
    ///    time it is evaluated).
    /// 3. Every variable appearing inside a [`Literal::Filter`] is bound by
    ///    some positive body atom, `Bind`, or aggregate export — filters guard bindings,
    ///    they do not produce them.
    /// 4. Every variable inside a [`Literal::Bind`] expression is bound by an
    ///    positive body atom, an earlier `Bind`, or an earlier aggregate export,
    ///    and the `Bind`'s output variable is fresh in that bound set.
    #[must_use]
    pub fn is_safe(&self) -> bool {
        self.unsafe_head_variables().is_none()
            && self.existential_fault().is_none()
            && self.unsafe_negative_variables().is_none()
            && self.unsafe_filter_variables().is_none()
            && self.unsafe_bind_variables().is_none()
            && self.unsafe_aggregate_variables().is_none()
    }

    /// First negative body literal with variables no positive literal binds.
    #[must_use]
    pub(super) fn unsafe_negative_variables(&self) -> Option<LiteralFault> {
        let bound = self.bound_body_variables();
        self.first_literal_fault(|lit| match lit {
            Literal::Negative(atom) => unbound(atom.variables(), &bound),
            Literal::Positive(_)
            | Literal::Filter(_)
            | Literal::Bind { .. }
            | Literal::Aggregate(_) => Vec::new(),
        })
    }

    /// First filter literal with variables no positive literal binds.
    #[must_use]
    pub(super) fn unsafe_filter_variables(&self) -> Option<LiteralFault> {
        let bound = self.bound_body_variables();
        self.first_literal_fault(|lit| match lit {
            Literal::Filter(filter) => unbound(filter.variables(), &bound),
            Literal::Positive(_)
            | Literal::Negative(_)
            | Literal::Bind { .. }
            | Literal::Aggregate(_) => Vec::new(),
        })
    }

    /// Position-aware safety check for `BIND(expr AS ?var)`.
    ///
    /// Walks the body in order, building up the set of variables bound by
    /// every positive atom, prior `Bind`, and prior aggregate export. Every
    /// variable inside `expr` must already be bound and the output variable
    /// must be fresh. Reports the first offending `Bind`.
    #[must_use]
    pub(super) fn unsafe_bind_variables(&self) -> Option<LiteralFault> {
        let mut bound = self.positive_atom_variables();
        self.first_literal_fault(|lit| match lit {
            Literal::Aggregate(aggregate) => {
                bound.extend(aggregate.exported_variables());
                Vec::new()
            }
            Literal::Bind { var, expr } => {
                let mut errors: BTreeSet<VariableSymbol> = expr
                    .variables()
                    .into_iter()
                    .filter(|v| !bound.contains(v))
                    .collect();
                if !bound.insert(var.clone()) {
                    errors.insert(var.clone());
                }
                errors.into_iter().collect()
            }
            Literal::Positive(_) | Literal::Negative(_) | Literal::Filter(_) => Vec::new(),
        })
    }

    /// Safety check for [`Literal::Aggregate`]; reports the first offending
    /// aggregate.
    ///
    /// 1. `input_var` and every `group_by` key are bound by the inner body.
    /// 2. `result_var` is fresh relative to positive body atoms, `BIND`
    ///    outputs, and previous aggregate exports.
    /// 3. Inner-body literals satisfy the outer per-kind safety rules.
    ///
    /// Nested aggregates are reported separately by
    /// [`Self::nested_aggregate_in_inner_body`].
    #[must_use]
    pub(super) fn unsafe_aggregate_variables(&self) -> Option<LiteralFault> {
        let mut already_bound = self.positive_atom_variables();
        for lit in &self.body {
            if let Literal::Bind { var, .. } = lit {
                already_bound.insert(var.clone());
            }
        }
        self.first_literal_fault(|lit| {
            let Literal::Aggregate(agg) = lit else {
                return Vec::new();
            };
            let inner_bound = agg.inner_bound_variables();
            let mut errors: BTreeSet<VariableSymbol> = BTreeSet::new();
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
            // Both the result and grouping keys are visible to subsequent
            // body literals; inner-only variables remain scoped inside.
            already_bound.extend(agg.exported_variables());
            errors.into_iter().collect()
        })
    }

    /// Path to the first aggregate nested inside an aggregate's inner body.
    #[must_use]
    pub(super) fn nested_aggregate_in_inner_body(&self) -> Option<LiteralPath> {
        self.body.iter().enumerate().find_map(|(outer, lit)| {
            let Literal::Aggregate(agg) = lit else {
                return None;
            };
            agg.inner_body
                .iter()
                .position(Literal::is_aggregate)
                .map(|inner| {
                    LiteralPath::top(LiteralIndex::new(outer)).inner(LiteralIndex::new(inner))
                })
        })
    }

    /// First top-level body literal for which `check` reports variables.
    /// `check` sees every literal in order, so it may carry scope state.
    fn first_literal_fault(
        &self,
        mut check: impl FnMut(&Literal) -> Vec<VariableSymbol>,
    ) -> Option<LiteralFault> {
        self.body.iter().enumerate().find_map(|(index, lit)| {
            let variables = check(lit);
            (!variables.is_empty())
                .then(|| LiteralFault::new(LiteralPath::top(LiteralIndex::new(index)), variables))
        })
    }

    /// Variables bound by the time the rule head fires: positive
    /// body atoms, every `Bind` output variable, and every
    /// `Aggregate` body literal's exported variables
    /// (`result_var` and `group_by` — the inner-only variables
    /// stay scoped inside the aggregate). Used by head,
    /// negation, and filter safety checks.
    #[must_use]
    pub(super) fn bound_body_variables(&self) -> BTreeSet<VariableSymbol> {
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
