//! Deserialize through the same checked construction used by callers.
use super::{Atom, Literal, Program, ProgramValidationError, Rule};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProgramInput {
    rules: Vec<Rule>,
}
impl TryFrom<ProgramInput> for Program {
    type Error = ProgramValidationError;
    fn try_from(input: ProgramInput) -> Result<Self, Self::Error> {
        Self::try_new(input.rules)
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuleInput {
    head: Atom,
    body: Vec<Literal>,
}
impl TryFrom<RuleInput> for Rule {
    type Error = ProgramValidationError;
    fn try_from(input: RuleInput) -> Result<Self, Self::Error> {
        Self::try_new(input.head, input.body)
    }
}
