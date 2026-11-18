//! Stable kebab-case codes of [`ProgramValidationError`] variants.

use super::variants::ProgramValidationError;
use crate::error_code::ErrorCode;

impl ProgramValidationError {
    /// Code of [`Self::UnsafeRule`].
    pub const CODE_UNSAFE_RULE: ErrorCode = ErrorCode::new("unsafe-rule");
    /// Code of [`Self::UnsafeNegation`].
    pub const CODE_UNSAFE_NEGATION: ErrorCode = ErrorCode::new("unsafe-negation");
    /// Code of [`Self::UnsafeFilter`].
    pub const CODE_UNSAFE_FILTER: ErrorCode = ErrorCode::new("unsafe-filter");
    /// Code of [`Self::UnsafeBind`].
    pub const CODE_UNSAFE_BIND: ErrorCode = ErrorCode::new("unsafe-bind");
    /// Code of [`Self::UnsafeAggregate`].
    pub const CODE_UNSAFE_AGGREGATE: ErrorCode = ErrorCode::new("unsafe-aggregate");
    /// Code of [`Self::NestedAggregateInInnerBody`].
    pub const CODE_NESTED_AGGREGATE: ErrorCode = ErrorCode::new("nested-aggregate");
    /// Code of [`Self::UnstratifiableCycle`].
    pub const CODE_UNSTRATIFIABLE_CYCLE: ErrorCode = ErrorCode::new("unstratifiable-cycle");
    /// Code of [`Self::NegationUnderEqualityMode`].
    pub const CODE_NEGATION_UNDER_EQUALITY_MODE: ErrorCode =
        ErrorCode::new("negation-under-equality-mode");
    /// Code of [`Self::AggregationUnderEqualityMode`].
    pub const CODE_AGGREGATION_UNDER_EQUALITY_MODE: ErrorCode =
        ErrorCode::new("aggregation-under-equality-mode");
    /// Code of [`Self::SkolemInBodyAtom`].
    pub const CODE_SKOLEM_IN_BODY: ErrorCode = ErrorCode::new("skolem-in-body");
    /// Code of [`Self::TripleTermNestingExceeded`].
    pub const CODE_TRIPLE_TERM_NESTING_EXCEEDED: ErrorCode =
        ErrorCode::new("triple-term-nesting-exceeded");
    /// Code of [`Self::InvalidSkolemArguments`].
    pub const CODE_INVALID_SKOLEM_ARGUMENTS: ErrorCode = ErrorCode::new("invalid-skolem-arguments");

    /// Code of [`Self::RuleCountExceeded`].
    pub const CODE_RULE_COUNT_EXCEEDED: ErrorCode = ErrorCode::new("rule-count-exceeded");
    /// Code of [`Self::BodyLengthExceeded`].
    pub const CODE_BODY_LENGTH_EXCEEDED: ErrorCode = ErrorCode::new("body-length-exceeded");
    /// Code of [`Self::SkolemDepthExceeded`].
    pub const CODE_SKOLEM_DEPTH_EXCEEDED: ErrorCode = ErrorCode::new("skolem-depth-exceeded");

    /// Stable kebab-case code of this variant.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::UnsafeRule { .. } => Self::CODE_UNSAFE_RULE,
            Self::UnsafeNegation { .. } => Self::CODE_UNSAFE_NEGATION,
            Self::UnsafeFilter { .. } => Self::CODE_UNSAFE_FILTER,
            Self::UnsafeBind { .. } => Self::CODE_UNSAFE_BIND,
            Self::UnsafeAggregate { .. } => Self::CODE_UNSAFE_AGGREGATE,
            Self::NestedAggregateInInnerBody { .. } => Self::CODE_NESTED_AGGREGATE,
            Self::UnstratifiableCycle(_) => Self::CODE_UNSTRATIFIABLE_CYCLE,
            Self::NegationUnderEqualityMode { .. } => Self::CODE_NEGATION_UNDER_EQUALITY_MODE,
            Self::AggregationUnderEqualityMode { .. } => Self::CODE_AGGREGATION_UNDER_EQUALITY_MODE,
            Self::SkolemInBodyAtom { .. } => Self::CODE_SKOLEM_IN_BODY,
            Self::TripleTermNestingExceeded { .. } => Self::CODE_TRIPLE_TERM_NESTING_EXCEEDED,
            Self::InvalidSkolemArguments { .. } => Self::CODE_INVALID_SKOLEM_ARGUMENTS,
            Self::RuleCountExceeded { .. } => Self::CODE_RULE_COUNT_EXCEEDED,
            Self::BodyLengthExceeded { .. } => Self::CODE_BODY_LENGTH_EXCEEDED,
            Self::SkolemDepthExceeded { .. } => Self::CODE_SKOLEM_DEPTH_EXCEEDED,
        }
    }
}
