//! Shared helpers for IR unit tests.

pub(super) use std::collections::BTreeSet;

pub(super) use crate::ir::aggregate::{AggregateBody, Aggregator};
pub(super) use crate::ir::atom::Atom;
pub(super) use crate::ir::bind::BindExpr;
pub(super) use crate::ir::equality_mode::EqualityMode;
pub(super) use crate::ir::error::ProgramValidationError;
pub(super) use crate::ir::filter::BuiltinFilter;
pub(super) use crate::ir::literal::Literal;
pub(super) use crate::ir::program::Program;
pub(super) use crate::ir::rule::Rule;
pub(super) use crate::ir::symbols::{ConstantSymbol, PredicateSymbol, VariableSymbol};
pub(super) use crate::ir::term::Term;
pub(super) use crate::limits::ValidationLimits;
pub(super) use crate::position::{
    AtomSlot, LiteralIndex, LiteralPath, RuleIndex, RuleSite, TermPath, TermRoot,
};

pub(super) fn rule0() -> RuleIndex {
    RuleIndex::new(0)
}

pub(super) fn lit(index: usize) -> LiteralPath {
    LiteralPath::top(LiteralIndex::new(index))
}

pub(super) fn slot(index: usize) -> TermPath {
    TermPath::new(TermRoot::Slot(AtomSlot::new(index)))
}

pub(super) fn var(name: &str) -> Term {
    Term::Variable(VariableSymbol::new(name))
}

pub(super) fn constant(name: &str) -> Term {
    Term::Constant(ConstantSymbol::new(name))
}

pub(super) fn atom(predicate: &str, terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms,
    }
}

// ── extension aggregation IR ────────────────────────────────────────

pub(super) fn agg_body(
    result: &str,
    aggregator: Aggregator,
    input: &str,
    group_by: Vec<&str>,
    inner_atoms: Vec<Atom>,
) -> AggregateBody {
    agg_body_with_inner(
        result,
        aggregator,
        input,
        group_by,
        inner_atoms.into_iter().map(Literal::Positive).collect(),
    )
}

pub(super) fn agg_body_with_inner(
    result: &str,
    aggregator: Aggregator,
    input: &str,
    group_by: Vec<&str>,
    inner_body: Vec<Literal>,
) -> AggregateBody {
    AggregateBody {
        result_var: VariableSymbol::new(result),
        aggregator,
        input_var: VariableSymbol::new(input),
        group_by: group_by.into_iter().map(VariableSymbol::new).collect(),
        inner_body,
    }
}
