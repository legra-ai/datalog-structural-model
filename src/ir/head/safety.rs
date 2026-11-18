//! Range restriction of rule heads, with existential variables exempt.

use crate::ir::rule::Rule;
use crate::ir::symbols::VariableSymbol;
use crate::position::HeadAtomPath;

/// A head atom and the variables that make it unsafe.
pub(in crate::ir) struct HeadFault {
    pub(in crate::ir) atom: HeadAtomPath,
    pub(in crate::ir) variables: Vec<VariableSymbol>,
}

/// An existential variable that is not a genuine witness.
pub(in crate::ir) enum ExistentialFault {
    /// No head atom mentions the variable.
    Unused(VariableSymbol),
    /// The body already binds the variable.
    BoundInBody(VariableSymbol),
}

impl Rule {
    /// First head atom, in path order, with variables that are neither bound
    /// by the body nor existentially quantified.
    #[must_use]
    pub(in crate::ir) fn unsafe_head_variables(&self) -> Option<HeadFault> {
        let bound = self.bound_body_variables();
        self.head.atoms().find_map(|(atom, value)| {
            let variables: Vec<VariableSymbol> = value
                .variables()
                .into_iter()
                .filter(|v| !bound.contains(v) && !self.head.is_existential(v))
                .collect();
            (!variables.is_empty()).then_some(HeadFault { atom, variables })
        })
    }

    /// First existential variable, in symbol order, that the body binds or
    /// that no head atom uses.
    #[must_use]
    pub(in crate::ir) fn existential_fault(&self) -> Option<ExistentialFault> {
        let existentials = self.head.existential_variables();
        if existentials.is_empty() {
            return None;
        }
        let bound = self.bound_body_variables();
        let used = self.head.variables();
        existentials.iter().find_map(|variable| {
            if bound.contains(variable) {
                Some(ExistentialFault::BoundInBody(variable.clone()))
            } else if used.contains(variable) {
                None
            } else {
                Some(ExistentialFault::Unused(variable.clone()))
            }
        })
    }
}
