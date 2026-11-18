#![doc = include_str!("../README.md")]

//! Typed Datalog rule programs and stratification analysis.

mod capability;
mod digest;
mod error_code;
mod feature;
mod ir;
mod limits;
mod position;
mod profile;
mod requirements;
mod scc;
mod stratify;
mod termination;

pub use digest::{ContractDigest, DigestParseError, ProgramDigest, ValidationContract};
pub use error_code::ErrorCode;
pub use ir::{
    AggregateBody, Aggregator, Atom, BindExpr, BuiltinFilter, ConstantSymbol, EmptyHeadCollection,
    EqualityMode, ExistentialVariables, GroundAtom, GroundTerm, HeadCollectionKind,
    HeadConjunction, HeadDisjunction, HeadForm, Literal, PredicateSymbol, Program,
    ProgramValidationError, Rule, RuleHead, Term, TermShapeError, VariableSymbol,
};
pub use limits::ValidationLimits;
pub use position::{
    AtomSlot, ExpressionTermIndex, HeadAtomIndex, HeadAtomPath, HeadDisjunctIndex, LiteralIndex,
    LiteralPath, RuleIndex, RuleSite, TermPath, TermRoot,
};
pub use stratify::{
    Stratification, StratificationError, Stratum, StrictDependency, StrictDependencyKind,
    UnstratifiableCycle,
};

pub use termination::{
    ArgumentPosition, InventionKind, LawVersion, SpecialEdge, TerminationCriterion,
    TerminationViolation, WeakAcyclicity,
};

pub use capability::{DatalogCapabilities, DatalogFeature};
pub use feature::{BindingKind, FilterKind};
pub use profile::{DatalogProfile, ProfileCheckError, UnsupportedCapabilities};
