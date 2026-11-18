//! Validation contracts and their digests.

use super::{
    domain::{CanonicalPreimage, DomainDigest},
    output::{ContractDigest, ProgramDigest},
    sink::PreimageSink,
};
use crate::{EqualityMode, Program, TerminationCriterion, ValidationLimits};

/// The parameters a program is checked under, separate from the program.
///
/// A [`ContractDigest`] binds a [`ProgramDigest`] to one contract, so a cache
/// of validation outcomes never reuses a result across limits, equality modes
/// or termination laws, while the program's own identity stays independent of
/// how it was checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationContract {
    equality_mode: EqualityMode,
    limits: ValidationLimits,
    termination: TerminationCriterion,
}

impl ValidationContract {
    /// Creates a contract.
    #[must_use]
    pub const fn new(
        equality_mode: EqualityMode,
        limits: ValidationLimits,
        termination: TerminationCriterion,
    ) -> Self {
        Self {
            equality_mode,
            limits,
            termination,
        }
    }

    /// The equality mode.
    #[must_use]
    pub const fn equality_mode(&self) -> EqualityMode {
        self.equality_mode
    }

    /// The structural limits.
    #[must_use]
    pub const fn limits(&self) -> &ValidationLimits {
        &self.limits
    }

    /// The termination criterion.
    #[must_use]
    pub const fn termination(&self) -> TerminationCriterion {
        self.termination
    }

    /// Identity of `program` checked under this contract.
    #[must_use]
    pub fn digest_for(&self, program: &Program) -> ContractDigest {
        BoundContract {
            program: program.digest(),
            contract: self,
        }
        .digest()
    }
}

/// A program identity paired with the contract it is checked under.
struct BoundContract<'a> {
    program: ProgramDigest,
    contract: &'a ValidationContract,
}

impl DomainDigest for BoundContract<'_> {
    const DOMAIN: &'static str = "datalog-structural-model/contract-digest/v1";
    type Output = ContractDigest;
}

impl CanonicalPreimage for BoundContract<'_> {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        sink.frame(self.program.as_bytes());
        self.contract.write_preimage(sink);
    }
}

impl CanonicalPreimage for ValidationContract {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        sink.tag(match self.equality_mode {
            EqualityMode::Default => 0,
            EqualityMode::SameAs => 1,
        });
        sink.count(self.limits.triple_term_nesting());
        sink.count(self.limits.rules());
        sink.count(self.limits.body_literals());
        sink.count(self.limits.skolem_depth());
        sink.text(self.termination.label());
        sink.word(self.termination.law_version().get());
    }
}
