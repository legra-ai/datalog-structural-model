//! Typed locations inside a rule program, attached to validation errors.

mod head_path;
mod index;
mod literal_path;
mod rule_site;
mod term_path;

pub use head_path::HeadAtomPath;
pub use index::{
    AtomSlot, ExpressionTermIndex, HeadAtomIndex, HeadDisjunctIndex, LiteralIndex, RuleIndex,
};
pub use literal_path::LiteralPath;
pub use rule_site::RuleSite;
pub use term_path::{TermPath, TermRoot};
