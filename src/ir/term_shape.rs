//! Validate term shapes in the head, every body literal, and every
//! aggregate scope, reporting the exact rule, literal, and term path.
use super::walk::Visitor;
use super::{Atom, Literal, ProgramValidationError, Rule, Term, TermShapeError};
use crate::limits::ValidationLimits;
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
    fn locate_fault(
        &self,
        path: TermPath,
        in_body: bool,
        limits: &ValidationLimits,
    ) -> Option<(TermPath, TermFault)> {
        if let Err(error) = self.validate_structure_within(limits) {
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
        args.iter().enumerate().find_map(|(slot, arg)| {
            arg.locate_fault(path.child(AtomSlot::new(slot)), in_body, limits)
        })
    }
}

impl Atom {
    fn locate_fault(
        &self,
        in_body: bool,
        limits: &ValidationLimits,
    ) -> Option<(TermPath, TermFault)> {
        self.terms.iter().enumerate().find_map(|(slot, term)| {
            let path = TermPath::new(TermRoot::Slot(AtomSlot::new(slot)));
            term.locate_fault(path, in_body, limits)
        })
    }
}

/// Visits the top-level terms of one filter or BIND expression in order.
struct ExpressionScanner<'l> {
    limits: &'l ValidationLimits,
    depth: usize,
    ordinal: usize,
    fault: Option<(TermPath, TermFault)>,
}

impl<'l> ExpressionScanner<'l> {
    fn new(limits: &'l ValidationLimits) -> Self {
        Self {
            limits,
            depth: 0,
            ordinal: 0,
            fault: None,
        }
    }
}

impl Visitor for ExpressionScanner<'_> {
    fn term(&mut self, term: &Term) {
        if self.depth == 0 && self.fault.is_none() {
            let root = TermRoot::ExpressionTerm(ExpressionTermIndex::new(self.ordinal));
            self.fault = term.locate_fault(TermPath::new(root), true, self.limits);
            self.ordinal += 1;
        }
        self.depth += 1;
    }

    fn leave_term(&mut self, _term: &Term) {
        self.depth -= 1;
    }
}

impl Literal {
    fn locate_fault(
        &self,
        path: &LiteralPath,
        limits: &ValidationLimits,
    ) -> Option<(LiteralPath, TermPath, TermFault)> {
        let found = match self {
            Self::Positive(atom) | Self::Negative(atom) => atom.locate_fault(true, limits),
            Self::Filter(filter) => {
                let mut scanner = ExpressionScanner::new(limits);
                filter.walk(&mut scanner);
                scanner.fault
            }
            Self::Bind { expr, .. } => {
                let mut scanner = ExpressionScanner::new(limits);
                expr.walk(&mut scanner);
                scanner.fault
            }
            Self::Aggregate(aggregate) => {
                return locate_in_body(&aggregate.inner_body, Some(path), limits);
            }
        };
        found.map(|(term, fault)| (path.clone(), term, fault))
    }
}

fn locate_in_body(
    body: &[Literal],
    parent: Option<&LiteralPath>,
    limits: &ValidationLimits,
) -> Option<(LiteralPath, TermPath, TermFault)> {
    body.iter().enumerate().find_map(|(index, literal)| {
        let index = LiteralIndex::new(index);
        let path = parent.map_or_else(|| LiteralPath::top(index), |path| path.inner(index));
        literal.locate_fault(&path, limits)
    })
}

impl Rule {
    pub(super) fn validate_term_shapes(
        &self,
        rule: RuleIndex,
        limits: &ValidationLimits,
    ) -> Result<(), ProgramValidationError> {
        if let Some((term, fault)) = self.head.locate_fault(false, limits) {
            return Err(fault.into_error(rule, RuleSite::Head, term));
        }
        match locate_in_body(&self.body, None, limits) {
            Some((literal, term, fault)) => {
                Err(fault.into_error(rule, RuleSite::Body(literal), term))
            }
            None => Ok(()),
        }
    }
}
