//! Internal exhaustive traversal shared by validation and capability inspection.
use super::{Atom, BindExpr, BuiltinFilter, Literal, Rule, Term};

pub(crate) trait Visitor {
    fn term(&mut self, _term: &Term) {}
    fn binding(&mut self, _expr: &BindExpr) {}
    fn filter(&mut self, _filter: &BuiltinFilter) {}
    fn literal(&mut self, _literal: &Literal) {}
}
impl Rule {
    pub(crate) fn walk(&self, visitor: &mut impl Visitor) {
        self.head.walk(visitor);
        for literal in &self.body {
            literal.walk(visitor);
        }
    }
}
impl Atom {
    pub(crate) fn walk(&self, visitor: &mut impl Visitor) {
        for term in &self.terms {
            term.walk(visitor);
        }
    }
}
impl Term {
    pub(crate) fn walk(&self, visitor: &mut impl Visitor) {
        visitor.term(self);
        match self {
            Self::Variable(_) | Self::Constant(_) => {}
            Self::TripleTerm(atom) => atom.walk(visitor),
            Self::Skolem { args, .. } => {
                for term in args {
                    term.walk(visitor);
                }
            }
        }
    }
}
impl Literal {
    pub(crate) fn walk(&self, visitor: &mut impl Visitor) {
        visitor.literal(self);
        match self {
            Self::Positive(atom) | Self::Negative(atom) => atom.walk(visitor),
            Self::Filter(filter) => filter.walk(visitor),
            Self::Bind { expr, .. } => expr.walk(visitor),
            Self::Aggregate(aggregate) => {
                for literal in &aggregate.inner_body {
                    literal.walk(visitor);
                }
            }
        }
    }
}
impl BindExpr {
    pub(crate) fn walk(&self, visitor: &mut impl Visitor) {
        visitor.binding(self);
        match self {
            Self::Term(a0) => {
                a0.walk(visitor);
            }
            Self::Add(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Sub(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Mul(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Div(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Neg(a0) => {
                a0.walk(visitor);
            }
            Self::Abs(a0) => {
                a0.walk(visitor);
            }
            Self::Ceil(a0) => {
                a0.walk(visitor);
            }
            Self::Floor(a0) => {
                a0.walk(visitor);
            }
            Self::Round(a0) => {
                a0.walk(visitor);
            }
            Self::Str(a0) => {
                a0.walk(visitor);
            }
            Self::StrLen(a0) => {
                a0.walk(visitor);
            }
            Self::UCase(a0) => {
                a0.walk(visitor);
            }
            Self::LCase(a0) => {
                a0.walk(visitor);
            }
            Self::Concat(a0) => {
                for item in a0 {
                    item.walk(visitor);
                }
            }
            Self::SubStr { value, start, len } => {
                value.walk(visitor);
                start.walk(visitor);
                if let Some(item) = len {
                    item.walk(visitor);
                }
            }
            Self::StrBefore(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::StrAfter(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Replace {
                value,
                pattern,
                replacement,
                flags,
            } => {
                value.walk(visitor);
                pattern.walk(visitor);
                replacement.walk(visitor);
                if let Some(item) = flags {
                    item.walk(visitor);
                }
            }
            Self::EncodeForUri(a0) => {
                a0.walk(visitor);
            }
            Self::Md5(a0) => {
                a0.walk(visitor);
            }
            Self::Sha1(a0) => {
                a0.walk(visitor);
            }
            Self::Sha256(a0) => {
                a0.walk(visitor);
            }
            Self::Sha384(a0) => {
                a0.walk(visitor);
            }
            Self::Sha512(a0) => {
                a0.walk(visitor);
            }
            Self::CastInteger(a0) => {
                a0.walk(visitor);
            }
            Self::CastDecimal(a0) => {
                a0.walk(visitor);
            }
            Self::CastDouble(a0) => {
                a0.walk(visitor);
            }
            Self::CastFloat(a0) => {
                a0.walk(visitor);
            }
            Self::CastString(a0) => {
                a0.walk(visitor);
            }
            Self::Year(a0) => {
                a0.walk(visitor);
            }
            Self::Month(a0) => {
                a0.walk(visitor);
            }
            Self::Day(a0) => {
                a0.walk(visitor);
            }
            Self::Hours(a0) => {
                a0.walk(visitor);
            }
            Self::Minutes(a0) => {
                a0.walk(visitor);
            }
            Self::Seconds(a0) => {
                a0.walk(visitor);
            }
            Self::Timezone(a0) => {
                a0.walk(visitor);
            }
            Self::Tz(a0) => {
                a0.walk(visitor);
            }
            Self::Now => {}
            Self::Lang(a0) => {
                a0.walk(visitor);
            }
            Self::Iri(a0) => {
                a0.walk(visitor);
            }
            Self::BNode(a0) => {
                a0.walk(visitor);
            }
        }
    }
}
impl BuiltinFilter {
    pub(crate) fn walk(&self, visitor: &mut impl Visitor) {
        visitor.filter(self);
        match self {
            Self::Eq(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Ne(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Lt(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Le(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Gt(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Ge(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::StrStarts(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::StrEnds(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Contains(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::RegexMatch {
                value,
                pattern,
                flags,
            } => {
                value.walk(visitor);
                pattern.walk(visitor);
                if let Some(item) = flags {
                    item.walk(visitor);
                }
            }
            Self::IsIri(a0) => {
                a0.walk(visitor);
            }
            Self::IsLiteral(a0) => {
                a0.walk(visitor);
            }
            Self::IsBlank(a0) => {
                a0.walk(visitor);
            }
            Self::IsNumeric(a0) => {
                a0.walk(visitor);
            }
            Self::LangMatches(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::And(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Or(a0, a1) => {
                a0.walk(visitor);
                a1.walk(visitor);
            }
            Self::Not(a0) => {
                a0.walk(visitor);
            }
        }
    }
}
