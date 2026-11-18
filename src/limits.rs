//! [`ValidationLimits`] — structural ceilings a program must stay within.

/// Structural ceilings checked during validation. Every ceiling is
/// inclusive: a value equal to its limit is accepted.
///
/// [`Self::default`] holds the standard limits used by [`Program::validate`]
/// and the built-in profiles. A [`DatalogProfile`] owns its limits, and
/// [`Program::validate_for`] enforces them.
///
/// [`Program::validate`]: crate::Program::validate
/// [`Program::validate_for`]: crate::Program::validate_for
/// [`DatalogProfile`]: crate::DatalogProfile
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationLimits {
    triple_term_nesting: usize,
    rules: usize,
    body_literals: usize,
    skolem_depth: usize,
}

impl ValidationLimits {
    /// Standard triple-term nesting depth.
    pub const STANDARD_TRIPLE_TERM_NESTING: usize = 4;
    /// Standard number of rules in one program.
    pub const STANDARD_RULES: usize = 65_536;
    /// Standard number of literals in one rule body, aggregate bodies included.
    pub const STANDARD_BODY_LITERALS: usize = 1_024;
    /// Standard depth of invented values, as measured by
    /// [`WeakAcyclicity::skolem_depth`](crate::WeakAcyclicity::skolem_depth).
    pub const STANDARD_SKOLEM_DEPTH: usize = 64;

    /// The standard limits.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            triple_term_nesting: Self::STANDARD_TRIPLE_TERM_NESTING,
            rules: Self::STANDARD_RULES,
            body_literals: Self::STANDARD_BODY_LITERALS,
            skolem_depth: Self::STANDARD_SKOLEM_DEPTH,
        }
    }

    /// Replace the triple-term nesting depth limit.
    #[must_use]
    pub const fn with_triple_term_nesting(mut self, limit: usize) -> Self {
        self.triple_term_nesting = limit;
        self
    }

    /// Replace the rule count limit.
    #[must_use]
    pub const fn with_rules(mut self, limit: usize) -> Self {
        self.rules = limit;
        self
    }

    /// Replace the per-rule body length limit.
    #[must_use]
    pub const fn with_body_literals(mut self, limit: usize) -> Self {
        self.body_literals = limit;
        self
    }

    /// Replace the invented-value depth limit.
    #[must_use]
    pub const fn with_skolem_depth(mut self, limit: usize) -> Self {
        self.skolem_depth = limit;
        self
    }

    /// Deepest accepted [`Term::TripleTerm`](crate::Term::TripleTerm) nesting.
    #[must_use]
    pub const fn triple_term_nesting(&self) -> usize {
        self.triple_term_nesting
    }

    /// Most accepted rules in one program.
    #[must_use]
    pub const fn rules(&self) -> usize {
        self.rules
    }

    /// Most accepted literals in one rule, counting aggregate inner bodies.
    #[must_use]
    pub const fn body_literals(&self) -> usize {
        self.body_literals
    }

    /// Deepest accepted nesting of invented values.
    #[must_use]
    pub const fn skolem_depth(&self) -> usize {
        self.skolem_depth
    }
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self::standard()
    }
}
