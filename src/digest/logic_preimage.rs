//! Canonical preimages of builtin filters and binding expressions.

use super::{domain::CanonicalPreimage, sink::PreimageSink};
use crate::{BindExpr, BuiltinFilter, Term};

/// Writes an optional child: tag 0 for absent, tag 1 then the value.
fn write_optional<T: CanonicalPreimage>(value: Option<&T>, sink: &mut PreimageSink) {
    match value {
        None => sink.tag(0),
        Some(value) => {
            sink.tag(1);
            value.write_preimage(sink);
        }
    }
}

impl<T: CanonicalPreimage> CanonicalPreimage for Box<T> {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        T::write_preimage(self, sink);
    }
}

impl BuiltinFilter {
    fn write_pair(tag: u8, left: &Term, right: &Term, sink: &mut PreimageSink) {
        sink.tag(tag);
        left.write_preimage(sink);
        right.write_preimage(sink);
    }

    fn write_unary(tag: u8, term: &Term, sink: &mut PreimageSink) {
        sink.tag(tag);
        term.write_preimage(sink);
    }
}

impl CanonicalPreimage for BuiltinFilter {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        match self {
            Self::Eq(a, b) => Self::write_pair(0, a, b, sink),
            Self::Ne(a, b) => Self::write_pair(1, a, b, sink),
            Self::Lt(a, b) => Self::write_pair(2, a, b, sink),
            Self::Le(a, b) => Self::write_pair(3, a, b, sink),
            Self::Gt(a, b) => Self::write_pair(4, a, b, sink),
            Self::Ge(a, b) => Self::write_pair(5, a, b, sink),
            Self::StrStarts(a, b) => Self::write_pair(6, a, b, sink),
            Self::StrEnds(a, b) => Self::write_pair(7, a, b, sink),
            Self::Contains(a, b) => Self::write_pair(8, a, b, sink),
            Self::RegexMatch {
                value,
                pattern,
                flags,
            } => {
                Self::write_pair(9, value, pattern, sink);
                write_optional(flags.as_ref(), sink);
            }
            Self::IsIri(a) => Self::write_unary(10, a, sink),
            Self::IsLiteral(a) => Self::write_unary(11, a, sink),
            Self::IsBlank(a) => Self::write_unary(12, a, sink),
            Self::IsNumeric(a) => Self::write_unary(13, a, sink),
            Self::LangMatches(a, b) => Self::write_pair(14, a, b, sink),
            Self::And(a, b) => {
                sink.tag(15);
                a.write_preimage(sink);
                b.write_preimage(sink);
            }
            Self::Or(a, b) => {
                sink.tag(16);
                a.write_preimage(sink);
                b.write_preimage(sink);
            }
            Self::Not(a) => {
                sink.tag(17);
                a.write_preimage(sink);
            }
        }
    }
}

impl BindExpr {
    fn write_unary(tag: u8, operand: &Self, sink: &mut PreimageSink) {
        sink.tag(tag);
        operand.write_preimage(sink);
    }

    fn write_binary(tag: u8, left: &Self, right: &Self, sink: &mut PreimageSink) {
        sink.tag(tag);
        left.write_preimage(sink);
        right.write_preimage(sink);
    }
}

impl CanonicalPreimage for BindExpr {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        match self {
            Self::Term(term) => {
                sink.tag(0);
                term.write_preimage(sink);
            }
            Self::Add(a, b) => Self::write_binary(1, a, b, sink),
            Self::Sub(a, b) => Self::write_binary(2, a, b, sink),
            Self::Mul(a, b) => Self::write_binary(3, a, b, sink),
            Self::Div(a, b) => Self::write_binary(4, a, b, sink),
            Self::StrBefore(a, b) => Self::write_binary(5, a, b, sink),
            Self::StrAfter(a, b) => Self::write_binary(6, a, b, sink),
            Self::Concat(parts) => {
                sink.tag(7);
                Self::write_sequence(parts, sink);
            }
            Self::SubStr { value, start, len } => {
                Self::write_binary(8, value, start, sink);
                write_optional(len.as_ref(), sink);
            }
            Self::Replace {
                value,
                pattern,
                replacement,
                flags,
            } => {
                Self::write_binary(9, value, pattern, sink);
                replacement.write_preimage(sink);
                write_optional(flags.as_ref(), sink);
            }
            Self::Neg(a) => Self::write_unary(10, a, sink),
            Self::Abs(a) => Self::write_unary(11, a, sink),
            Self::Ceil(a) => Self::write_unary(12, a, sink),
            Self::Floor(a) => Self::write_unary(13, a, sink),
            Self::Round(a) => Self::write_unary(14, a, sink),
            Self::Str(a) => Self::write_unary(15, a, sink),
            Self::StrLen(a) => Self::write_unary(16, a, sink),
            Self::UCase(a) => Self::write_unary(17, a, sink),
            Self::LCase(a) => Self::write_unary(18, a, sink),
            Self::EncodeForUri(a) => Self::write_unary(19, a, sink),
            Self::Md5(a) => Self::write_unary(20, a, sink),
            Self::Sha1(a) => Self::write_unary(21, a, sink),
            Self::Sha256(a) => Self::write_unary(22, a, sink),
            Self::Sha384(a) => Self::write_unary(23, a, sink),
            Self::Sha512(a) => Self::write_unary(24, a, sink),
            Self::CastInteger(a) => Self::write_unary(25, a, sink),
            Self::CastDecimal(a) => Self::write_unary(26, a, sink),
            Self::CastDouble(a) => Self::write_unary(27, a, sink),
            Self::CastFloat(a) => Self::write_unary(28, a, sink),
            Self::CastString(a) => Self::write_unary(29, a, sink),
            Self::Year(a) => Self::write_unary(30, a, sink),
            Self::Month(a) => Self::write_unary(31, a, sink),
            Self::Day(a) => Self::write_unary(32, a, sink),
            Self::Hours(a) => Self::write_unary(33, a, sink),
            Self::Minutes(a) => Self::write_unary(34, a, sink),
            Self::Seconds(a) => Self::write_unary(35, a, sink),
            Self::Timezone(a) => Self::write_unary(36, a, sink),
            Self::Tz(a) => Self::write_unary(37, a, sink),
            Self::Lang(a) => Self::write_unary(38, a, sink),
            Self::Iri(a) => Self::write_unary(39, a, sink),
            Self::BNode(a) => Self::write_unary(40, a, sink),
            Self::Now => sink.tag(41),
        }
    }
}
