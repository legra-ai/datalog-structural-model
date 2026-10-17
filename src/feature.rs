//! Exact built-in operation identities used by capability declarations.
use crate::{BindExpr, BuiltinFilter};
/// An operation represented by [`BindExpr`], independent of evaluator support.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum BindingKind {
    /// The `Term` operation.
    Term,
    /// The `Add` operation.
    Add,
    /// The `Sub` operation.
    Sub,
    /// The `Mul` operation.
    Mul,
    /// The `Div` operation.
    Div,
    /// The `Neg` operation.
    Neg,
    /// The `Abs` operation.
    Abs,
    /// The `Ceil` operation.
    Ceil,
    /// The `Floor` operation.
    Floor,
    /// The `Round` operation.
    Round,
    /// The `Str` operation.
    Str,
    /// The `StrLen` operation.
    StrLen,
    /// The `UCase` operation.
    UCase,
    /// The `LCase` operation.
    LCase,
    /// The `Concat` operation.
    Concat,
    /// The `SubStr` operation.
    SubStr,
    /// The `StrBefore` operation.
    StrBefore,
    /// The `StrAfter` operation.
    StrAfter,
    /// The `Replace` operation.
    Replace,
    /// The `EncodeForUri` operation.
    EncodeForUri,
    /// The `Md5` operation.
    Md5,
    /// The `Sha1` operation.
    Sha1,
    /// The `Sha256` operation.
    Sha256,
    /// The `Sha384` operation.
    Sha384,
    /// The `Sha512` operation.
    Sha512,
    /// The `CastInteger` operation.
    CastInteger,
    /// The `CastDecimal` operation.
    CastDecimal,
    /// The `CastDouble` operation.
    CastDouble,
    /// The `CastFloat` operation.
    CastFloat,
    /// The `CastString` operation.
    CastString,
    /// The `Year` operation.
    Year,
    /// The `Month` operation.
    Month,
    /// The `Day` operation.
    Day,
    /// The `Hours` operation.
    Hours,
    /// The `Minutes` operation.
    Minutes,
    /// The `Seconds` operation.
    Seconds,
    /// The `Timezone` operation.
    Timezone,
    /// The `Tz` operation.
    Tz,
    /// The `Now` operation.
    Now,
    /// The `Lang` operation.
    Lang,
    /// The `Iri` operation.
    Iri,
    /// The `BNode` operation.
    BNode,
}
impl BindingKind {
    /// Every operation in this model revision.
    pub const ALL: &'static [Self] = &[
        Self::Term,
        Self::Add,
        Self::Sub,
        Self::Mul,
        Self::Div,
        Self::Neg,
        Self::Abs,
        Self::Ceil,
        Self::Floor,
        Self::Round,
        Self::Str,
        Self::StrLen,
        Self::UCase,
        Self::LCase,
        Self::Concat,
        Self::SubStr,
        Self::StrBefore,
        Self::StrAfter,
        Self::Replace,
        Self::EncodeForUri,
        Self::Md5,
        Self::Sha1,
        Self::Sha256,
        Self::Sha384,
        Self::Sha512,
        Self::CastInteger,
        Self::CastDecimal,
        Self::CastDouble,
        Self::CastFloat,
        Self::CastString,
        Self::Year,
        Self::Month,
        Self::Day,
        Self::Hours,
        Self::Minutes,
        Self::Seconds,
        Self::Timezone,
        Self::Tz,
        Self::Now,
        Self::Lang,
        Self::Iri,
        Self::BNode,
    ];
}
impl BindExpr {
    /// Return this expression’s operation identity.
    #[must_use]
    pub fn kind(&self) -> BindingKind {
        match self {
            Self::Term(..) => BindingKind::Term,
            Self::Add(..) => BindingKind::Add,
            Self::Sub(..) => BindingKind::Sub,
            Self::Mul(..) => BindingKind::Mul,
            Self::Div(..) => BindingKind::Div,
            Self::Neg(..) => BindingKind::Neg,
            Self::Abs(..) => BindingKind::Abs,
            Self::Ceil(..) => BindingKind::Ceil,
            Self::Floor(..) => BindingKind::Floor,
            Self::Round(..) => BindingKind::Round,
            Self::Str(..) => BindingKind::Str,
            Self::StrLen(..) => BindingKind::StrLen,
            Self::UCase(..) => BindingKind::UCase,
            Self::LCase(..) => BindingKind::LCase,
            Self::Concat(..) => BindingKind::Concat,
            Self::SubStr { .. } => BindingKind::SubStr,
            Self::StrBefore(..) => BindingKind::StrBefore,
            Self::StrAfter(..) => BindingKind::StrAfter,
            Self::Replace { .. } => BindingKind::Replace,
            Self::EncodeForUri(..) => BindingKind::EncodeForUri,
            Self::Md5(..) => BindingKind::Md5,
            Self::Sha1(..) => BindingKind::Sha1,
            Self::Sha256(..) => BindingKind::Sha256,
            Self::Sha384(..) => BindingKind::Sha384,
            Self::Sha512(..) => BindingKind::Sha512,
            Self::CastInteger(..) => BindingKind::CastInteger,
            Self::CastDecimal(..) => BindingKind::CastDecimal,
            Self::CastDouble(..) => BindingKind::CastDouble,
            Self::CastFloat(..) => BindingKind::CastFloat,
            Self::CastString(..) => BindingKind::CastString,
            Self::Year(..) => BindingKind::Year,
            Self::Month(..) => BindingKind::Month,
            Self::Day(..) => BindingKind::Day,
            Self::Hours(..) => BindingKind::Hours,
            Self::Minutes(..) => BindingKind::Minutes,
            Self::Seconds(..) => BindingKind::Seconds,
            Self::Timezone(..) => BindingKind::Timezone,
            Self::Tz(..) => BindingKind::Tz,
            Self::Now => BindingKind::Now,
            Self::Lang(..) => BindingKind::Lang,
            Self::Iri(..) => BindingKind::Iri,
            Self::BNode(..) => BindingKind::BNode,
        }
    }
}
/// An operation represented by [`BuiltinFilter`], independent of evaluator support.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum FilterKind {
    /// The `Eq` operation.
    Eq,
    /// The `Ne` operation.
    Ne,
    /// The `Lt` operation.
    Lt,
    /// The `Le` operation.
    Le,
    /// The `Gt` operation.
    Gt,
    /// The `Ge` operation.
    Ge,
    /// The `StrStarts` operation.
    StrStarts,
    /// The `StrEnds` operation.
    StrEnds,
    /// The `Contains` operation.
    Contains,
    /// The `RegexMatch` operation.
    RegexMatch,
    /// The `IsIri` operation.
    IsIri,
    /// The `IsLiteral` operation.
    IsLiteral,
    /// The `IsBlank` operation.
    IsBlank,
    /// The `IsNumeric` operation.
    IsNumeric,
    /// The `LangMatches` operation.
    LangMatches,
    /// The `And` operation.
    And,
    /// The `Or` operation.
    Or,
    /// The `Not` operation.
    Not,
}
impl FilterKind {
    /// Every operation in this model revision.
    pub const ALL: &'static [Self] = &[
        Self::Eq,
        Self::Ne,
        Self::Lt,
        Self::Le,
        Self::Gt,
        Self::Ge,
        Self::StrStarts,
        Self::StrEnds,
        Self::Contains,
        Self::RegexMatch,
        Self::IsIri,
        Self::IsLiteral,
        Self::IsBlank,
        Self::IsNumeric,
        Self::LangMatches,
        Self::And,
        Self::Or,
        Self::Not,
    ];
}
impl BuiltinFilter {
    /// Return this expression’s operation identity.
    #[must_use]
    pub fn kind(&self) -> FilterKind {
        match self {
            Self::Eq(..) => FilterKind::Eq,
            Self::Ne(..) => FilterKind::Ne,
            Self::Lt(..) => FilterKind::Lt,
            Self::Le(..) => FilterKind::Le,
            Self::Gt(..) => FilterKind::Gt,
            Self::Ge(..) => FilterKind::Ge,
            Self::StrStarts(..) => FilterKind::StrStarts,
            Self::StrEnds(..) => FilterKind::StrEnds,
            Self::Contains(..) => FilterKind::Contains,
            Self::RegexMatch { .. } => FilterKind::RegexMatch,
            Self::IsIri(..) => FilterKind::IsIri,
            Self::IsLiteral(..) => FilterKind::IsLiteral,
            Self::IsBlank(..) => FilterKind::IsBlank,
            Self::IsNumeric(..) => FilterKind::IsNumeric,
            Self::LangMatches(..) => FilterKind::LangMatches,
            Self::And(..) => FilterKind::And,
            Self::Or(..) => FilterKind::Or,
            Self::Not(..) => FilterKind::Not,
        }
    }
}
