//! Finite first-order tuple unification with independently scoped variables.

use crate::ir::{Atom, ConstantSymbol, PredicateSymbol, Term, VariableSymbol};
use std::collections::BTreeMap;

#[derive(Clone)]
enum Pattern {
    Variable(usize),
    Constant(ConstantSymbol),
    Triple(PredicateSymbol, Vec<Self>),
}

#[derive(Default)]
struct Unifier {
    variables: BTreeMap<(bool, VariableSymbol), usize>,
    bindings: Vec<Option<Pattern>>,
}

/// Whether the two atoms can denote the same ground fact. SKOLEM outputs are
/// conservatively fresh variables: their runtime value encoding is backend-owned.
pub(super) fn overlaps(head: &Atom, body: &Atom) -> bool {
    if head.predicate != body.predicate || head.arity() != body.arity() {
        return false;
    }
    let mut unifier = Unifier::default();
    // bounded: one pair per argument in two finite rule atoms.
    let mut pending: Vec<_> = head
        .terms
        .iter()
        .zip(&body.terms)
        .map(|(left, right)| (unifier.pattern(left, false), unifier.pattern(right, true)))
        .collect();
    while let Some((left, right)) = pending.pop() {
        let left = unifier.resolve(left);
        let right = unifier.resolve(right);
        match (left, right) {
            (Pattern::Variable(a), Pattern::Variable(b)) if a == b => {}
            (Pattern::Variable(variable), value) | (value, Pattern::Variable(variable)) => {
                if unifier.occurs(variable, &value) {
                    return false;
                }
                unifier.bindings[variable] = Some(value);
            }
            (Pattern::Constant(a), Pattern::Constant(b)) if a == b => {}
            (Pattern::Triple(a, left), Pattern::Triple(b, right))
                if a == b && left.len() == right.len() =>
            {
                pending.extend(left.into_iter().zip(right));
            }
            _ => return false,
        }
    }
    true
}

impl Unifier {
    fn fresh(&mut self) -> usize {
        let index = self.bindings.len();
        self.bindings.push(None);
        index
    }

    fn pattern(&mut self, term: &Term, side: bool) -> Pattern {
        match term {
            Term::Variable(name) => {
                let key = (side, name.clone());
                let index = if let Some(&index) = self.variables.get(&key) {
                    index
                } else {
                    let index = self.fresh();
                    self.variables.insert(key, index);
                    index
                };
                Pattern::Variable(index)
            }
            Term::Constant(value) => Pattern::Constant(value.clone()),
            Term::TripleTerm(atom) => Pattern::Triple(
                atom.predicate.clone(),
                atom.terms
                    .iter()
                    .map(|term| self.pattern(term, side))
                    .collect(),
            ),
            Term::Skolem { .. } => Pattern::Variable(self.fresh()),
        }
    }

    fn resolve(&self, mut pattern: Pattern) -> Pattern {
        while let Pattern::Variable(variable) = pattern {
            let Some(value) = &self.bindings[variable] else {
                break;
            };
            pattern = value.clone();
        }
        pattern
    }

    fn occurs(&self, variable: usize, value: &Pattern) -> bool {
        // bounded: terms and bindings of the two finite rule atoms.
        let mut pending = vec![value];
        while let Some(value) = pending.pop() {
            match value {
                Pattern::Variable(index) if *index == variable => return true,
                Pattern::Variable(index) => {
                    if let Some(bound) = &self.bindings[*index] {
                        pending.push(bound);
                    }
                }
                Pattern::Triple(_, terms) => pending.extend(terms),
                Pattern::Constant(_) => {}
            }
        }
        false
    }
}
