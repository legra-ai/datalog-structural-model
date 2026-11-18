//! Where each variable of a rule gets its value: the body positions that
//! bind it, and whether a construct invents a new value on the way.

use std::collections::{BTreeMap, BTreeSet};

use super::edge::{ArgumentPosition, InventionKind};
use crate::ir::{AggregateBody, Atom, Literal, VariableSymbol};
use crate::position::{AtomSlot, LiteralIndex, LiteralPath, RuleIndex, RuleSite};

/// One origin of a variable's value.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Source {
    pub(super) position: ArgumentPosition,
    pub(super) invention: Option<InventionKind>,
}

impl Source {
    fn invented(&self, kind: InventionKind) -> Self {
        Self {
            position: self.position.clone(),
            invention: Some(kind),
        }
    }
}

/// A positive body atom with its location, whose slots are graph nodes.
pub(super) struct BodyAtom<'a> {
    pub(super) literal: LiteralPath,
    pub(super) atom: &'a Atom,
}

impl BodyAtom<'_> {
    pub(super) fn position(&self, rule: RuleIndex, slot: usize) -> ArgumentPosition {
        ArgumentPosition::new(
            self.atom.predicate.clone(),
            AtomSlot::new(slot),
            rule,
            RuleSite::Body(self.literal.clone()),
        )
    }
}

/// Variable origins and positive atoms of one body scope.
// bounded: variables, atoms and positions of one finite authored rule.
pub(super) struct VariableSources<'a> {
    rule: RuleIndex,
    origins: BTreeMap<&'a VariableSymbol, BTreeSet<Source>>,
    pub(super) atoms: Vec<BodyAtom<'a>>,
}

impl<'a> VariableSources<'a> {
    /// Origins of every variable bound by the body of `rule`, in body order.
    pub(super) fn of_body(rule: RuleIndex, body: &'a [Literal]) -> Self {
        let mut sources = Self {
            rule,
            origins: BTreeMap::new(),
            atoms: Vec::new(),
        };
        sources.add_body(body, None);
        sources
    }

    /// Origins of `variable`; empty when nothing binds it.
    pub(super) fn of(&self, variable: &VariableSymbol) -> impl Iterator<Item = &Source> {
        self.origins.get(variable).into_iter().flatten()
    }

    fn add_body(&mut self, body: &'a [Literal], parent: Option<&LiteralPath>) {
        for (index, literal) in body.iter().enumerate() {
            let index = LiteralIndex::new(index);
            let path = parent.map_or_else(|| LiteralPath::top(index), |path| path.inner(index));
            self.add_literal(literal, path);
        }
    }

    fn add_literal(&mut self, literal: &'a Literal, path: LiteralPath) {
        match literal {
            Literal::Positive(atom) => self.add_atom(atom, path),
            Literal::Negative(_) | Literal::Filter(_) => {}
            Literal::Bind { var, expr } => {
                let invented = self.invented_from(&expr.variables(), InventionKind::Bind);
                self.origins.entry(var).or_default().extend(invented);
            }
            Literal::Aggregate(aggregate) => self.add_aggregate(aggregate, &path),
        }
    }

    fn add_atom(&mut self, atom: &'a Atom, literal: LiteralPath) {
        let body_atom = BodyAtom { literal, atom };
        for (slot, term) in atom.terms.iter().enumerate() {
            let position = body_atom.position(self.rule, slot);
            term.visit_variables(&mut |variable| {
                self.origins.entry(variable).or_default().insert(Source {
                    position: position.clone(),
                    invention: None,
                });
            });
        }
        self.atoms.push(body_atom);
    }

    /// The inner body is its own variable scope; only group keys and the
    /// folded result leave it.
    fn add_aggregate(&mut self, aggregate: &'a AggregateBody, path: &LiteralPath) {
        let mut inner = Self {
            rule: self.rule,
            origins: BTreeMap::new(),
            atoms: Vec::new(),
        };
        inner.add_body(&aggregate.inner_body, Some(path));
        for key in &aggregate.group_by {
            let copied: Vec<Source> = inner.of(key).cloned().collect();
            self.origins.entry(key).or_default().extend(copied);
        }
        let folded: Vec<Source> = inner
            .of(&aggregate.input_var)
            .map(|source| source.invented(InventionKind::Aggregate))
            .collect();
        self.origins
            .entry(&aggregate.result_var)
            .or_default()
            .extend(folded);
        self.atoms.append(&mut inner.atoms);
    }

    fn invented_from(
        &self,
        variables: &BTreeSet<VariableSymbol>,
        kind: InventionKind,
    ) -> Vec<Source> {
        variables
            .iter()
            .flat_map(|variable| self.of(variable))
            .map(|source| source.invented(kind))
            .collect()
    }
}
