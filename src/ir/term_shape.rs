//! Validate terms in every body expression and aggregate scope.
use super::walk::Visitor;
use super::{Literal, ProgramValidationError, Rule, Term, TermShapeError};
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
        if let Err(error) = term.validate_structure() {
            self.error = Some(match error {
                TermShapeError::TripleTermNestingExceeded { depth, limit } => {
                    ProgramValidationError::TripleTermNestingExceeded {
                        rule_index: self.rule_index,
                        depth,
                        limit,
                    }
                }
                TermShapeError::InvalidSkolemArguments => {
                    ProgramValidationError::InvalidSkolemArguments {
                        rule_index: self.rule_index,
                    }
                }
            });
            return;
        }
        if self.in_body && matches!(term, Term::Skolem { .. }) {
            self.error = Some(ProgramValidationError::SkolemInBodyAtom {
                rule_index: self.rule_index,
            });
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
