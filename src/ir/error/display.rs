//! `Display`, `Error`, and `From<StratificationError>` for
//! [`ProgramValidationError`].

use std::error::Error;
use std::fmt;

use super::variants::ProgramValidationError;
use crate::ir::symbols::VariableSymbol;
use crate::stratify::StratificationError;

impl From<StratificationError> for ProgramValidationError {
    fn from(err: StratificationError) -> Self {
        let StratificationError::UnstratifiableCycle(cycle) = err;
        Self::UnstratifiableCycle(cycle)
    }
}

struct VariableList<'a>(&'a [VariableSymbol]);

impl fmt::Display for VariableList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, var) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            f.write_str(var.as_str())?;
        }
        Ok(())
    }
}

impl fmt::Display for ProgramValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.code())?;
        match self {
            Self::UnsafeRule { rule, variables } => write!(
                f,
                "{rule} is unsafe; head variables not bound by the body: {}",
                VariableList(variables)
            ),
            Self::UnsafeNegation {
                rule,
                literal,
                variables,
            } => write!(
                f,
                "{rule}, {literal}: negated literal uses variables not bound by the positive \
                 body: {}",
                VariableList(variables)
            ),
            Self::UnsafeFilter {
                rule,
                literal,
                variables,
            } => write!(
                f,
                "{rule}, {literal}: filter uses variables not bound by the positive body: {}",
                VariableList(variables)
            ),
            Self::UnsafeBind {
                rule,
                literal,
                variables,
            } => write!(
                f,
                "{rule}, {literal}: unsafe BIND (unbound expression reference or colliding \
                 output name): {}",
                VariableList(variables)
            ),
            Self::UnsafeAggregate {
                rule,
                literal,
                variables,
            } => write!(
                f,
                "{rule}, {literal}: unsafe aggregate (unbound input or group key, colliding \
                 result variable, or unsafe inner literal): {}",
                VariableList(variables)
            ),
            Self::NestedAggregateInInnerBody { rule, literal } => write!(
                f,
                "{rule}, {literal}: aggregate nested inside an aggregate's inner body; lift it \
                 into its own rule"
            ),
            Self::UnstratifiableCycle(cycle) => write!(f, "program contains a {cycle}"),
            Self::NegationUnderEqualityMode { rule, literal } => write!(
                f,
                "{rule}, {literal}: negative literal while equality mode is `SameAs`"
            ),
            Self::AggregationUnderEqualityMode { rule, literal } => write!(
                f,
                "{rule}, {literal}: aggregate literal while equality mode is `SameAs`"
            ),
            Self::SkolemInBodyAtom {
                rule,
                literal,
                term,
            } => write!(
                f,
                "{rule}, {literal}, {term}: SKOLEM term in a body literal; SKOLEM is only \
                 allowed in the head"
            ),
            Self::TripleTermNestingExceeded {
                rule,
                site,
                term,
                depth,
                limit,
            } => write!(
                f,
                "{rule}, {site}, {term}: triple term nested {depth} deep; limit is {limit}"
            ),
            Self::InvalidSkolemArguments { rule, site, term } => write!(
                f,
                "{rule}, {site}, {term}: Skolem arguments must be variables or constants"
            ),
        }
    }
}

impl Error for ProgramValidationError {}
