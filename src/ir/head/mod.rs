//! Rule heads: atomic, conjunctive, existential, disjunctive, or falsum.
//!
//! # No-output heads are not modelled
//!
//! Some rule languages distinguish an *operational* no-output rule, whose body
//! runs but emits nothing, from a logical empty head (`false`), which asserts
//! the body unsatisfiable. The distinction rests on evaluating the body for
//! its effects. This crate never executes rules and its bodies have no
//! effects, so a no-output rule would denote nothing: it derives no fact and
//! constrains no model. Its only meaning would be operational, which belongs
//! to an evaluator. [`RuleHead::Falsum`] is therefore the only empty head, and
//! it always means an inconsistency constraint.

mod collections;
mod form;
mod rule_head;
mod safety;

pub use collections::{
    EmptyHeadCollection, ExistentialVariables, HeadCollectionKind, HeadConjunction, HeadDisjunction,
};
pub use form::HeadForm;
pub use rule_head::RuleHead;
pub(super) use safety::{ExistentialFault, HeadFault};
