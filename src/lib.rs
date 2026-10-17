#![doc = include_str!("../README.md")]

//! Typed Datalog rule programs and stratification analysis.

mod capability;
mod feature;
mod ir;
mod profile;
mod requirements;
mod stratify;

pub use ir::{
    AggregateBody, Aggregator, Atom, BindExpr, BuiltinFilter, ConstantSymbol, EqualityMode,
    GroundAtom, GroundTerm, Literal, MAX_TRIPLE_TERM_NESTING, PredicateSymbol, Program,
    ProgramValidationError, Rule, Term, TermShapeError, VariableSymbol,
};
pub use stratify::{Stratification, StratificationError, Stratum};

pub use capability::{DatalogCapabilities, DatalogFeature};
pub use feature::{BindingKind, FilterKind};
pub use profile::{DatalogProfile, ProfileCheckError, UnsupportedCapabilities};
