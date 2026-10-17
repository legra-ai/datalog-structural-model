//! Context-free term invariants shared by constructors and checked decoding.
use super::{Atom, ConstantSymbol, GroundAtom, GroundTerm, MAX_TRIPLE_TERM_NESTING, Term};
use std::{error::Error, fmt};

/// Invalid term structure, independent of variable binding or rule position.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TermShapeError {
    /// A structured term exceeds the supported nesting depth.
    TripleTermNestingExceeded {
        /// Observed nesting depth.
        depth: usize,
        /// Supported maximum depth.
        limit: usize,
    },
    /// A Skolem argument is neither a variable nor a constant.
    InvalidSkolemArguments,
}
impl fmt::Display for TermShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TripleTermNestingExceeded { depth, limit } => {
                write!(f, "triple term depth {depth} exceeds limit {limit}")
            }
            Self::InvalidSkolemArguments => {
                f.write_str("Skolem arguments must be variables or constants")
            }
        }
    }
}
impl Error for TermShapeError {}
impl Term {
    /// Construct a structured term with checked nesting and Skolem arguments.
    /// # Errors
    /// Rejects excessive nesting or invalid nested Skolem arguments.
    pub fn try_triple_term(atom: Atom) -> Result<Self, TermShapeError> {
        let term = Self::TripleTerm(Box::new(atom));
        term.validate_structure()?;
        Ok(term)
    }
    /// Construct a deterministic head term with flat arguments.
    /// # Errors
    /// Rejects arguments other than variables and constants.
    pub fn try_skolem(name: ConstantSymbol, args: Vec<Self>) -> Result<Self, TermShapeError> {
        let term = Self::Skolem { name, args };
        term.validate_structure()?;
        Ok(term)
    }
    /// Check context-free structure. Rule-position and binding checks require
    /// [`super::Program::validate`].
    /// # Errors
    /// Rejects excessive triple depth or non-flat Skolem arguments.
    pub fn validate_structure(&self) -> Result<(), TermShapeError> {
        let depth = self.nesting_depth();
        if depth > MAX_TRIPLE_TERM_NESTING {
            return Err(TermShapeError::TripleTermNestingExceeded {
                depth,
                limit: MAX_TRIPLE_TERM_NESTING,
            });
        }
        match self {
            Self::Variable(_) | Self::Constant(_) => {}
            Self::TripleTerm(atom) => {
                for term in &atom.terms {
                    term.validate_structure()?;
                }
            }
            Self::Skolem { args, .. } => {
                if args
                    .iter()
                    .any(|term| !matches!(term, Self::Variable(_) | Self::Constant(_)))
                {
                    return Err(TermShapeError::InvalidSkolemArguments);
                }
            }
        }
        Ok(())
    }
}
impl GroundTerm {
    fn nesting_depth(&self) -> usize {
        match self {
            Self::Constant(_) => 0,
            Self::TripleTerm(atom) => {
                1 + atom.args.iter().map(Self::nesting_depth).max().unwrap_or(0)
            }
        }
    }
    /// Construct a grounded structured term with checked nesting.
    /// # Errors
    /// Rejects excessive triple-term nesting.
    pub fn try_triple_term(atom: GroundAtom) -> Result<Self, TermShapeError> {
        let term = Self::TripleTerm(Box::new(atom));
        term.validate_structure()?;
        Ok(term)
    }
    /// Check the same nesting invariant used before grounding a program.
    /// # Errors
    /// Rejects excessive triple-term nesting.
    pub fn validate_structure(&self) -> Result<(), TermShapeError> {
        let depth = self.nesting_depth();
        if depth > MAX_TRIPLE_TERM_NESTING {
            return Err(TermShapeError::TripleTermNestingExceeded {
                depth,
                limit: MAX_TRIPLE_TERM_NESTING,
            });
        }
        Ok(())
    }
}
