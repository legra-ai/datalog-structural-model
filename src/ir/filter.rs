//! Built-in filter expressions guarding rule-body candidate
//! bindings.

use std::collections::BTreeSet;

use super::symbols::VariableSymbol;
use super::term::Term;

/// Built-in filter expression.
///
/// Carries a tree of comparisons / string tests over Datalog
/// [`Term`]s plus boolean compositors. The evaluator substitutes
/// each variable with its bound constant from the current
/// candidate binding and then evaluates the test; bindings that
/// fail to substitute (variable not bound), encounter a malformed
/// regex pattern, or evaluate to anything other than `true` are
/// dropped from the frontier.
///
/// **Comparison semantics**: `Lt`, `Le`, `Gt`, `Ge` compare two
/// [`ConstantSymbol`](super::symbols::ConstantSymbol)s
/// lexicographically over the underlying string representation.
/// SPARQL value-typed numeric or `xsd:dateTime` ordering is
/// *not yet* implemented at this layer.
///
/// **String tests**: `StrStarts`, `StrEnds`, `Contains`, and
/// `RegexMatch` operate on the raw constant string. SPARQL
/// language-tag and datatype-aware semantics — `LANGMATCHES`,
/// language-aware `REGEX` evaluation against `xsd:string`-only
/// arguments, etc. — track as further extension follow-ups; see
/// the rule validation specification.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BuiltinFilter {
    /// `lhs = rhs` — equality of constants.
    Eq(Term, Term),
    /// `lhs != rhs` — inequality of constants.
    Ne(Term, Term),
    /// `lhs < rhs` — lexicographic less-than over constants.
    Lt(Term, Term),
    /// `lhs <= rhs` — lexicographic less-than-or-equal.
    Le(Term, Term),
    /// `lhs > rhs` — lexicographic greater-than.
    Gt(Term, Term),
    /// `lhs >= rhs` — lexicographic greater-than-or-equal.
    Ge(Term, Term),
    /// `STRSTARTS(haystack, needle)` — `haystack` begins with
    /// `needle` over the **lexical form** of each argument. Plain
    /// literals (`"Alice"`), `xsd:string`-typed literals
    /// (`"Alice"^^xsd:string`), and bare IRI constants all
    /// compare against their bare-string representation, so the
    /// match is insensitive to datatype framing.
    StrStarts(Term, Term),
    /// `STRENDS(haystack, needle)` — `haystack` ends with
    /// `needle` over the lexical form of each argument. Same
    /// datatype-insensitivity as [`Self::StrStarts`].
    StrEnds(Term, Term),
    /// `CONTAINS(haystack, needle)` — `haystack` contains
    /// `needle` somewhere in the lexical form. Same
    /// datatype-insensitivity as [`Self::StrStarts`].
    Contains(Term, Term),
    /// `REGEX(value, pattern, flags?)` — `value` matches the
    /// `regex::Regex` compiled from `pattern` (and the optional
    /// `flags`), comparing against `value`'s lexical form (same
    /// datatype-insensitivity as [`Self::StrStarts`]). Supported
    /// flags follow the SPARQL subset that maps to the
    /// [`regex` crate]'s `(?…)` inline modifiers: `i`
    /// (case-insensitive), `m` (multi-line), `s`
    /// (dot-matches-newline), `x` (ignore whitespace inside the
    /// pattern). A compile failure or unsupported flag is treated
    /// as "undefined for the current binding" — the candidate is
    /// dropped, matching the SPARQL "type error → not true"
    /// framing.
    ///
    /// [`regex` crate]: https://docs.rs/regex
    RegexMatch {
        /// The string value to match against.
        value: Term,
        /// The regular expression pattern.
        pattern: Term,
        /// Optional flags string (e.g. `"i"`, `"im"`).
        flags: Option<Term>,
    },
    /// `isIRI(?x)` — true when `?x` resolves to an IRI under the
    /// term-kind classification in the evaluator’s RDF term classification.
    IsIri(Term),
    /// `isLiteral(?x)` — true when `?x` resolves to a literal.
    IsLiteral(Term),
    /// `isBlank(?x)` — true when `?x` resolves to a blank node.
    IsBlank(Term),
    /// `isNumeric(?x)` — true when `?x` resolves to a literal
    /// whose datatype IRI is recognized as numeric by
    /// the evaluator’s numeric datatype classification.
    IsNumeric(Term),
    /// `LANGMATCHES(language-tag, language-range)` — RFC 4647
    /// basic-range matching over the lexical form of both
    /// arguments. The wildcard range `"*"` matches any non-empty
    /// tag; otherwise the range matches the tag when the tag's
    /// initial subtags (split on `-`) match the range's subtags
    /// case-insensitively.
    ///
    /// Both arguments are resolved through the `lexical_form`
    /// helper, so callers may supply either a plain string
    /// constant or an encoded literal (typically the result of a
    /// future `BindExpr::Lang(?lit)` extraction). The full
    /// pipeline integration that wires `LANG(?lit)` through to
    /// language-tagged literal storage is tracked under R12-21
    /// follow-up — this slice lands the rule and the basic-range
    /// matcher; existing tests construct the language tag as a
    /// plain literal manually.
    LangMatches(Term, Term),
    /// `lhs && rhs` — logical conjunction of two filters.
    And(Box<BuiltinFilter>, Box<BuiltinFilter>),
    /// `lhs || rhs` — logical disjunction of two filters.
    Or(Box<BuiltinFilter>, Box<BuiltinFilter>),
    /// `!inner` — logical negation.
    Not(Box<BuiltinFilter>),
}

impl BuiltinFilter {
    /// All variables referenced anywhere inside the filter
    /// expression. Used by safety analysis to require that every
    /// such variable is bound by some positive body atom.
    #[must_use]
    pub fn variables(&self) -> BTreeSet<VariableSymbol> {
        let mut out = BTreeSet::new();
        self.collect_variables(&mut out);
        out
    }

    fn collect_variables(&self, out: &mut BTreeSet<VariableSymbol>) {
        match self {
            Self::Eq(a, b)
            | Self::Ne(a, b)
            | Self::Lt(a, b)
            | Self::Le(a, b)
            | Self::Gt(a, b)
            | Self::Ge(a, b)
            | Self::StrStarts(a, b)
            | Self::StrEnds(a, b)
            | Self::Contains(a, b) => {
                insert_variable(a, out);
                insert_variable(b, out);
            }
            Self::RegexMatch {
                value,
                pattern,
                flags,
            } => {
                insert_variable(value, out);
                insert_variable(pattern, out);
                if let Some(flags) = flags {
                    insert_variable(flags, out);
                }
            }
            Self::IsIri(t) | Self::IsLiteral(t) | Self::IsBlank(t) | Self::IsNumeric(t) => {
                insert_variable(t, out);
            }
            Self::LangMatches(tag, range) => {
                insert_variable(tag, out);
                insert_variable(range, out);
            }
            Self::And(lhs, rhs) | Self::Or(lhs, rhs) => {
                lhs.collect_variables(out);
                rhs.collect_variables(out);
            }
            Self::Not(inner) => inner.collect_variables(out),
        }
    }
}

fn insert_variable(term: &Term, out: &mut BTreeSet<VariableSymbol>) {
    term.collect_variables(out);
}
