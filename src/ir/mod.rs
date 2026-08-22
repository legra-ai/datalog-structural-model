//! Datalog intermediate representation.

mod aggregate;
mod atom;
mod bind;
mod equality_mode;
mod error;
mod filter;
mod ground;
mod literal;
mod program;
mod rule;
mod symbols;
mod term;

#[cfg(test)]
mod tests;

pub use aggregate::{AggregateBody, Aggregator};
pub use atom::Atom;
pub use bind::BindExpr;
pub use equality_mode::EqualityMode;
pub use error::ProgramValidationError;
pub use filter::BuiltinFilter;
pub use ground::{GroundAtom, GroundTerm};
pub use literal::Literal;
pub use program::Program;
pub use rule::Rule;
pub use symbols::{ConstantSymbol, PredicateSymbol, VariableSymbol};
pub use term::{MAX_TRIPLE_TERM_NESTING, Term};
