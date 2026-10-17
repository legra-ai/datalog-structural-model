//! Equality-mode flag for Datalog program validation.
//!
//! Some Datalog features interact non-monotonically with
//! equality rewriting. Specifically, when the
//! reasoner rewrites terms across equality classes (the
//! `eq-rep-s` / `eq-rep-p` / `eq-rep-o` rules; see
//! [`crate::ir::Literal::Negative`] for the corresponding
//! body-literal shape), the set a negated atom is evaluated
//! against can change shape mid-evaluation, and the multiset an
//! aggregate folds over likewise shifts. Both lead to
//! semantically unstable results.
//!
//! Programs that combine negation or aggregation with equality mode are
//! rejected because those operators depend on the complete contents of the
//! equality classes. This restriction keeps validation explicit and prevents
//! an evaluator from silently changing results while equality classes merge.

/// Equality-reasoning mode under which a Datalog program is
/// validated.
///
/// Programs are checked under [`Self::Default`] by
/// [`Program::validate`](super::program::Program::validate); callers that have
/// an active equality-rewriting regime pass [`Self::SameAs`] through
/// [`Program::validate_with_equality_mode`](super::program::Program::validate_with_equality_mode)
/// to additionally reject negation and aggregation. The two
/// methods share their other checks (safety, stratification,
/// triple-term nesting).
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub enum EqualityMode {
    /// No special equality-reasoning regime. All Datalog
    /// features — including negation and aggregation — are
    /// allowed.
    #[default]
    Default,
    /// The evaluator rewrites terms across equality classes. Negation
    /// and aggregation are rejected at validation time because
    /// they interact non-monotonically with the rewrite — see
    /// the module-level docs.
    SameAs,
}
