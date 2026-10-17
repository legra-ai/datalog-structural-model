//! Deserialize through the same checked construction used by callers.
use super::{Atom, Literal, Program, ProgramValidationError, Rule};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProgramInput {
    rules: Vec<RuleInput>,
}
impl TryFrom<ProgramInput> for Program {
    type Error = ProgramValidationError;
    fn try_from(input: ProgramInput) -> Result<Self, Self::Error> {
        Self::try_new(
            input
                .rules
                .into_iter()
                .map(RuleInput::into_description)
                .collect(),
        )
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuleInput {
    head: Atom,
    body: Vec<Literal>,
}
impl RuleInput {
    // Preserve the surrounding program's original rule indices. Only the
    // private decoding description bypasses per-rule checks; Program::try_new
    // immediately validates the complete program with the correct context.
    fn into_description(self) -> Rule {
        Rule::new(self.head, self.body)
    }
}
impl TryFrom<RuleInput> for Rule {
    type Error = ProgramValidationError;
    fn try_from(input: RuleInput) -> Result<Self, Self::Error> {
        Self::try_new(input.head, input.body)
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum TermInput {
    Variable(super::VariableSymbol),
    Constant(super::ConstantSymbol),
    TripleTerm(Box<Atom>),
    Skolem {
        name: super::ConstantSymbol,
        args: Vec<super::Term>,
    },
}
impl TryFrom<TermInput> for super::Term {
    type Error = super::TermShapeError;
    fn try_from(input: TermInput) -> Result<Self, Self::Error> {
        let term = match input {
            TermInput::Variable(value) => Self::Variable(value),
            TermInput::Constant(value) => Self::Constant(value),
            TermInput::TripleTerm(value) => Self::TripleTerm(value),
            TermInput::Skolem { name, args } => Self::Skolem { name, args },
        };
        term.validate_structure()?;
        Ok(term)
    }
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum GroundTermInput {
    Constant(super::ConstantSymbol),
    TripleTerm(Box<super::GroundAtom>),
}
impl TryFrom<GroundTermInput> for super::GroundTerm {
    type Error = super::TermShapeError;
    fn try_from(input: GroundTermInput) -> Result<Self, Self::Error> {
        let term = match input {
            GroundTermInput::Constant(value) => Self::Constant(value),
            GroundTermInput::TripleTerm(value) => Self::TripleTerm(value),
        };
        term.validate_structure()?;
        Ok(term)
    }
}
