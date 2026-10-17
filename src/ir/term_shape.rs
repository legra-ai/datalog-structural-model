//! Validate terms in every body expression and aggregate scope.
use super::walk::Visitor;
use super::{Literal, MAX_TRIPLE_TERM_NESTING, ProgramValidationError, Rule, Term};
struct TermShapeValidator {
    rule_index: usize,
    in_body: bool,
    error: Option<ProgramValidationError>,
}
impl Visitor for TermShapeValidator {
    fn literal(&mut self, _literal: &Literal) {
        self.in_body = true;
    }
    fn term(&mut self, term: &Term) {
        if self.error.is_some() {
            return;
        }
        let depth = term.nesting_depth();
        if depth > MAX_TRIPLE_TERM_NESTING {
            self.error = Some(ProgramValidationError::TripleTermNestingExceeded {
                rule_index: self.rule_index,
                depth,
                limit: MAX_TRIPLE_TERM_NESTING,
            });
            return;
        }
        if let Term::Skolem { args, .. } = term {
            if self.in_body {
                self.error = Some(ProgramValidationError::SkolemInBodyAtom {
                    rule_index: self.rule_index,
                });
            } else if args
                .iter()
                .any(|arg| !matches!(arg, Term::Variable(_) | Term::Constant(_)))
            {
                self.error = Some(ProgramValidationError::InvalidSkolemArguments {
                    rule_index: self.rule_index,
                });
            }
        }
    }
}
impl Rule {
    pub(super) fn validate_term_shapes(
        &self,
        rule_index: usize,
    ) -> Result<(), ProgramValidationError> {
        let mut visitor = TermShapeValidator {
            rule_index,
            in_body: false,
            error: None,
        };
        self.walk(&mut visitor);
        match visitor.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
