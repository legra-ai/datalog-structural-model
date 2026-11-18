//! [`HeadForm`] — the logical shape of a rule head.

use std::fmt;

use crate::DatalogFeature;

/// Which logical form a [`RuleHead`](super::RuleHead) has.
///
/// The form is a pure function of the head with fixed precedence: falsum,
/// then existential, then disjunctive (more than one disjunct), then
/// conjunctive (more than one atom), then atomic. A one-disjunct disjunction
/// classifies by its conjunction and a one-atom conjunction is atomic, so the
/// classification depends on content, not on the constructor used.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum HeadForm {
    /// Exactly one atom and no existential variable: plain Datalog.
    Atomic,
    /// One disjunct of several atoms, no existential variable.
    Conjunctive,
    /// At least one existentially quantified head variable.
    Existential,
    /// Several disjuncts, no existential variable.
    Disjunctive,
    /// The head is `false`: a body match is an inconsistency witness.
    Inconsistency,
}

impl HeadForm {
    /// Every form, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::Atomic,
        Self::Conjunctive,
        Self::Existential,
        Self::Disjunctive,
        Self::Inconsistency,
    ];

    /// The capability a profile must declare to accept this form; `None` for
    /// the atomic form every Datalog profile accepts.
    #[must_use]
    pub const fn required_feature(self) -> Option<DatalogFeature> {
        match self {
            Self::Atomic => None,
            Self::Conjunctive => Some(DatalogFeature::ConjunctiveHeads),
            Self::Existential => Some(DatalogFeature::ExistentialHeads),
            Self::Disjunctive => Some(DatalogFeature::DisjunctiveHeads),
            Self::Inconsistency => Some(DatalogFeature::InconsistencyConstraints),
        }
    }

    /// Stable kebab-case name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Atomic => "atomic",
            Self::Conjunctive => "conjunctive",
            Self::Existential => "existential",
            Self::Disjunctive => "disjunctive",
            Self::Inconsistency => "inconsistency",
        }
    }
}

impl fmt::Display for HeadForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
