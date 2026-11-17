//! Validate term shapes in the head, every body literal, and every
//! aggregate scope, reporting the exact rule, literal, and term path.
use super::walk::Visitor;
use super::{Atom, Literal, ProgramValidationError, Rule, Term, TermShapeError};
use crate::position::{
    AtomSlot, ExpressionTermIndex, LiteralIndex, LiteralPath, RuleIndex, RuleSite, TermPath,
    TermRoot,
};

/// A term-level defect found at a [`TermPath`].
enum TermFault {
    Shape(TermShapeError),
    SkolemInBody,
}

impl TermFault {
    fn into_error(self, rule: RuleIndex, site: RuleSite, term: TermPath) -> ProgramValidationError {
        match (self, site) {
            (Self::Shape(TermShapeError::TripleTermNestingExceeded { depth, limit }), site) => {
                ProgramValidationError::TripleTermNestingExceeded {
                    rule,
                    site,
                    term,
                    depth,
                    limit,
                }
            }
            (Self::Shape(TermShapeError::InvalidSkolemArguments), site) => {
                ProgramValidationError::InvalidSkolemArguments { rule, site, term }
            }
            (Self::SkolemInBody, RuleSite::Body(literal)) => {
                ProgramValidationError::SkolemInBodyAtom {
                    rule,
                    literal,
                    term,
                }
            }
            (Self::SkolemInBody, RuleSite::Head) => {
                unreachable!("Skolem terms are only faults in body literals")
            }
        }
    }
}

impl Term {
    /// First fault in pre-order: a shape error at this term, a Skolem in a
    /// body, or a fault in a nested argument.
    fn locate_fault(&self, path: TermPath, in_body: bool) -> Option<(TermPath, TermFault)> {
        if let Err(error) = self.validate_structure() {
            return Some((path, TermFault::Shape(error)));
        }
        let args: &[Term] = match self {
            Self::Variable(_) | Self::Constant(_) => &[],
            Self::TripleTerm(atom) => &atom.terms,
            Self::Skolem { args, .. } => {
                if in_body {
                    return Some((path, TermFault::SkolemInBody));
                }
                args
            }
        };
        args.iter()
            .enumerate()
            .find_map(|(slot, arg)| arg.locate_fault(path.child(AtomSlot::new(slot)), in_body))
    }
}

impl Atom {
    fn locate_fault(&self, in_body: bool) -> Option<(TermPath, TermFault)> {
        self.terms.iter().enumerate().find_map(|(slot, term)| {
            term.locate_fault(TermPath::new(TermRoot::Slot(AtomSlot::new(slot))), in_body)
        })
    }
}

/// Visits the top-level terms of one filter or BIND expression in order.
#[derive(Default)]
struct ExpressionScanner {
    depth: usize,
    ordinal: usize,
    fault: Option<(TermPath, TermFault)>,
}

impl Visitor for ExpressionScanner {
    fn term(&mut self, term: &Term) {
        if self.depth == 0 && self.fault.is_none() {
            let root = TermRoot::ExpressionTerm(ExpressionTermIndex::new(self.ordinal));
            self.fault = term.locate_fault(TermPath::new(root), true);
            self.ordinal += 1;
        }
        self.depth += 1;
    }

    fn leave_term(&mut self, _term: &Term) {
        self.depth -= 1;
    }
}

impl Literal {
    fn locate_fault(&self, path: &LiteralPath) -> Option<(LiteralPath, TermPath, TermFault)> {
        let found = match self {
            Self::Positive(atom) | Self::Negative(atom) => atom.locate_fault(true),
            Self::Filter(filter) => {
                let mut scanner = ExpressionScanner::default();
                filter.walk(&mut scanner);
                scanner.fault
            }
            Self::Bind { expr, .. } => {
                let mut scanner = ExpressionScanner::default();
                expr.walk(&mut scanner);
                scanner.fault
            }
            Self::Aggregate(aggregate) => {
                return locate_in_body(&aggregate.inner_body, Some(path));
            }
        };
        found.map(|(term, fault)| (path.clone(), term, fault))
    }
}

fn locate_in_body(
    body: &[Literal],
    parent: Option<&LiteralPath>,
) -> Option<(LiteralPath, TermPath, TermFault)> {
    body.iter().enumerate().find_map(|(index, literal)| {
        let index = LiteralIndex::new(index);
        let path = parent.map_or_else(|| LiteralPath::top(index), |path| path.inner(index));
        literal.locate_fault(&path)
    })
}

impl Rule {
    pub(super) fn validate_term_shapes(
        &self,
        rule: RuleIndex,
    ) -> Result<(), ProgramValidationError> {
        if let Some((term, fault)) = self.head.locate_fault(false) {
            return Err(fault.into_error(rule, RuleSite::Head, term));
        }
        match locate_in_body(&self.body, None) {
            Some((literal, term, fault)) => {
                Err(fault.into_error(rule, RuleSite::Body(literal), term))
            }
            None => Ok(()),
        }
    }
}
