#![doc = include_str!("../README.md")]

//! Typed Datalog rule programs and stratification analysis.

mod ir;
mod stratify;

pub use ir::{
    AggregateBody, Aggregator, Atom, BindExpr, BuiltinFilter, ConstantSymbol, EqualityMode,
    GroundAtom, GroundTerm, Literal, MAX_TRIPLE_TERM_NESTING, PredicateSymbol, Program,
    ProgramValidationError, Rule, Term, VariableSymbol,
};
pub use stratify::{Stratification, StratificationError, Stratum};
