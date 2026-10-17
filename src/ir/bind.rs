//! `BIND(expr AS ?var)` expression tree.

use std::collections::BTreeSet;

use super::symbols::VariableSymbol;
use super::term::Term;

/// Expression carried by a
/// [`Literal::Bind`](super::literal::Literal::Bind) body literal.
///
/// `BindExpr` covers identity assignment plus integer arithmetic
/// over [`ConstantSymbol`](super::symbols::ConstantSymbol)s whose
/// lexical form parses as an `i64`. Operands may be
/// [`Term::Variable`]s (resolved against the current candidate
/// binding) or [`Term::Constant`]s.
///
/// **Numeric semantics**: arithmetic operands must parse as
/// base-10 `i64`; otherwise the expression is undefined for the
/// current binding and the evaluator drops the candidate. SPARQL
/// numeric promotion (integer → decimal → float → double),
/// `xsd:decimal` arithmetic, and the remaining SPARQL string /
/// date / IRI builtins inside `BIND` are tracked as extension
/// follow-ups under
/// the rule validation specification.
///
/// **String semantics**: `Str`, `StrLen`, `UCase`, `LCase`, and
/// `Concat` operate on the lexical form of literals (when the
/// operand classifies as a literal under the evaluator’s RDF term classification)
/// or on the constant string itself otherwise. The result of
/// `Str` / `UCase` / `LCase` / `Concat` is encoded as a plain
/// literal (empty datatype IRI); the result of `StrLen` is an
/// `xsd:integer` literal so subsequent arithmetic can consume it
/// without re-parsing.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BindExpr {
    /// Identity — bind the variable to the term itself (typically
    /// useful in the form `BIND(?x AS ?y)` for renaming).
    Term(Term),
    /// SPARQL `+`. Type-polymorphic in the forward engine: numeric
    /// addition (over the integer → decimal → float → double
    /// promotion lattice) or
    /// `xsd:dateTime + xsd:dayTimeDuration → xsd:dateTime`.
    /// Operand pairs that don't match either shape drop the
    /// candidate.
    Add(Box<BindExpr>, Box<BindExpr>),
    /// SPARQL `-`. Type-polymorphic in the forward engine: numeric
    /// subtraction, `xsd:dateTime - xsd:dayTimeDuration →
    /// xsd:dateTime` (backward shift), or
    /// `xsd:dateTime - xsd:dateTime → xsd:dayTimeDuration` (the
    /// duration between two instants).
    Sub(Box<BindExpr>, Box<BindExpr>),
    /// Integer multiplication over `i64`.
    Mul(Box<BindExpr>, Box<BindExpr>),
    /// Integer division (truncated) over `i64`. Division by zero
    /// is undefined and drops the candidate binding.
    Div(Box<BindExpr>, Box<BindExpr>),
    /// Integer negation over `i64`.
    Neg(Box<BindExpr>),
    /// `ABS(numeric)` — absolute value, preserving the input's SPARQL
    /// numeric datatype (integer → integer, decimal → decimal,
    /// float → float, double → double). A non-numeric operand drops
    /// the candidate binding.
    Abs(Box<BindExpr>),
    /// `CEIL(numeric)` — smallest integer-valued result ≥ the input,
    /// preserving the input's SPARQL numeric datatype. For an
    /// `xsd:integer` input the value passes through unchanged. For
    /// an `xsd:decimal` input the lexical form is the integer
    /// ceiling (no fractional component) but the datatype stays
    /// `xsd:decimal` per SPARQL 1.1 §17.4.4.4.
    Ceil(Box<BindExpr>),
    /// `FLOOR(numeric)` — largest integer-valued result ≤ the input,
    /// preserving the input's SPARQL numeric datatype. Symmetric to
    /// [`Self::Ceil`].
    Floor(Box<BindExpr>),
    /// `ROUND(numeric)` — round half toward positive infinity
    /// (`XPath` `fn:round` semantics, matching SPARQL 1.1
    /// §17.4.4.6). Preserves the input's SPARQL numeric datatype.
    Round(Box<BindExpr>),
    /// `STR(?x)` — lexical form. For literals returns a plain
    /// literal carrying the lex; for IRIs / blank nodes returns
    /// the constant string as a plain literal.
    Str(Box<BindExpr>),
    /// `STRLEN(?x)` — number of Unicode scalar values in the
    /// lexical form. Result is an `xsd:integer` literal.
    StrLen(Box<BindExpr>),
    /// `UCASE(?x)` — uppercase the lexical form (Unicode-aware).
    UCase(Box<BindExpr>),
    /// `LCASE(?x)` — lowercase the lexical form (Unicode-aware).
    LCase(Box<BindExpr>),
    /// `CONCAT(?a, ?b, ...)` — concatenate the lexical forms of
    /// each argument and return a plain literal.
    Concat(Vec<BindExpr>),
    /// `SUBSTR(value, start)` or `SUBSTR(value, start, len)` —
    /// 1-based substring extraction over Unicode scalar values.
    /// Negative or zero `start` clamps to position 1; start past
    /// the end of the value yields the empty plain literal; an
    /// over-long `len` is clamped to "until end". Mirrors the
    /// SPARQL 1.1 semantics for `SUBSTR`.
    SubStr {
        /// The string-valued source expression.
        value: Box<BindExpr>,
        /// 1-based start position, in Unicode scalars.
        start: Box<BindExpr>,
        /// Optional length, in Unicode scalars. `None` means
        /// "until end".
        len: Option<Box<BindExpr>>,
    },
    /// `STRBEFORE(value, needle)` — substring of `value` before
    /// the first occurrence of `needle`. Per SPARQL 1.1: empty
    /// plain literal when `needle` is not found, and an empty
    /// plain literal when `needle` itself is empty.
    StrBefore(Box<BindExpr>, Box<BindExpr>),
    /// `STRAFTER(value, needle)` — substring of `value` after
    /// the first occurrence of `needle`. Per SPARQL 1.1: empty
    /// plain literal when `needle` is not found, and the full
    /// `value` when `needle` is empty.
    StrAfter(Box<BindExpr>, Box<BindExpr>),
    /// `REPLACE(value, pattern, replacement, flags?)` — replace
    /// every match of the regex `pattern` (compiled with the
    /// same SPARQL flag subset as
    /// [`BuiltinFilter::RegexMatch`](crate::BuiltinFilter::RegexMatch)
    /// — `i`, `m`, `s`, `x`) with `replacement`. A malformed
    /// pattern, an unsupported flag, or any unbound variable
    /// drops the candidate binding.
    Replace {
        /// The string-valued source expression.
        value: Box<BindExpr>,
        /// The regex pattern.
        pattern: Box<BindExpr>,
        /// The replacement template (back-references such as
        /// `$1` follow the [`regex` crate]'s syntax).
        ///
        /// [`regex` crate]: https://docs.rs/regex
        replacement: Box<BindExpr>,
        /// Optional flags string (e.g. `"i"`, `"im"`).
        flags: Option<Box<BindExpr>>,
    },
    /// `ENCODE_FOR_URI(value)` — percent-encode every code point
    /// outside the RFC 3986 unreserved set (`ALPHA` / `DIGIT` /
    /// `-` / `.` / `_` / `~`) into its UTF-8-byte `%HH` form.
    EncodeForUri(Box<BindExpr>),
    /// `MD5(value)` — RFC 1321 message digest of the lexical
    /// form's UTF-8 bytes. Result is a plain literal carrying
    /// the lowercase hex digest.
    Md5(Box<BindExpr>),
    /// `SHA1(value)` — FIPS 180-4 SHA-1 digest. Result is a
    /// plain literal carrying the lowercase hex digest.
    Sha1(Box<BindExpr>),
    /// `SHA256(value)` — FIPS 180-4 SHA-256 digest.
    Sha256(Box<BindExpr>),
    /// `SHA384(value)` — FIPS 180-4 SHA-384 digest.
    Sha384(Box<BindExpr>),
    /// `SHA512(value)` — FIPS 180-4 SHA-512 digest.
    Sha512(Box<BindExpr>),
    /// `xsd:integer(value)` — parse the lexical form as base-10
    /// `i64`. Emits an `xsd:integer` literal. A parse failure
    /// drops the candidate binding.
    CastInteger(Box<BindExpr>),
    /// `xsd:decimal(value)` — parse the lexical form as a SPARQL
    /// decimal (delegates to `bigdecimal::BigDecimal::from_str`).
    /// Emits an `xsd:decimal` literal carrying the parsed
    /// canonical form. A parse failure drops the candidate.
    CastDecimal(Box<BindExpr>),
    /// `xsd:double(value)` — parse the lexical form as a base-10
    /// `f64`. Emits an `xsd:double` literal. A parse failure or
    /// non-finite value drops the candidate.
    CastDouble(Box<BindExpr>),
    /// `xsd:float(value)` — parse the lexical form as a base-10
    /// `f32`. Emits an `xsd:float` literal. A parse failure or
    /// non-finite value drops the candidate.
    CastFloat(Box<BindExpr>),
    /// `xsd:string(value)` — emit the lexical form as an
    /// `xsd:string`-typed literal. Always succeeds.
    CastString(Box<BindExpr>),
    /// `YEAR(dateTime)` — extract the year component as an
    /// `xsd:integer` literal. The argument's lexical form must
    /// parse as `xsd:dateTime`; otherwise the candidate drops.
    Year(Box<BindExpr>),
    /// `MONTH(dateTime)` — month component (1–12) as an
    /// `xsd:integer` literal.
    Month(Box<BindExpr>),
    /// `DAY(dateTime)` — day-of-month (1–31) as an
    /// `xsd:integer` literal.
    Day(Box<BindExpr>),
    /// `HOURS(dateTime)` — hour-of-day (0–23) as an
    /// `xsd:integer` literal.
    Hours(Box<BindExpr>),
    /// `MINUTES(dateTime)` — minute-of-hour (0–59) as an
    /// `xsd:integer` literal.
    Minutes(Box<BindExpr>),
    /// `SECONDS(dateTime)` — second-of-minute (including the
    /// fractional part) as an `xsd:decimal` literal.
    Seconds(Box<BindExpr>),
    /// `TIMEZONE(dateTime)` — UTC offset as an
    /// `xsd:dayTimeDuration` literal (e.g. `"PT5H"`, `"-PT5H"`,
    /// `"PT0S"`). The candidate drops when the input has no
    /// timezone designator (per SPARQL spec — `TIMEZONE` is an
    /// error in that case).
    Timezone(Box<BindExpr>),
    /// `TZ(dateTime)` — UTC offset as an `xsd:string` literal:
    /// `"Z"` for UTC, `"+05:00"` / `"-08:00"` etc. for offset
    /// designators, and the empty string when the input has no
    /// timezone.
    Tz(Box<BindExpr>),
    /// `NOW()` — current `xsd:dateTime` (UTC). The literal
    /// returned is fixed at evaluation time; SPARQL guarantees
    /// it stays constant across one query, but this Datalog
    /// `BIND` evaluates the expression once per candidate
    /// binding, which is consistent with SPARQL's stability
    /// requirement (every reference under one rule firing sees
    /// the same instant).
    Now,
    /// `LANG(?lit)` — extract the language tag from a
    /// language-tagged literal. Returns a plain `xsd:string`
    /// literal carrying the tag (lower-cased per SPARQL 1.1
    /// canonicalization), or the empty string for non-language-
    /// tagged literals. Drops the candidate when the operand is
    /// not a literal (IRIs / blank nodes — SPARQL "type error
    /// → no result").
    Lang(Box<BindExpr>),
    /// `IRI(?str)` — construct an IRI term from a string-valued
    /// expression. If the operand is already an IRI it passes
    /// through unchanged (SPARQL 1.1 §17.4.2.8). Drops the
    /// candidate for blank-node operands.
    Iri(Box<BindExpr>),
    /// `BNODE(?seed)` — construct a blank node deterministically
    /// from a string seed. Multiple calls with the same seed
    /// yield the same blank-node label, which matches SPARQL 1.1
    /// §17.4.2.4 "two calls with the same string return the same
    /// blank node within one solution sequence" and gives rules
    /// reproducible blank-node identity across firings. The
    /// 0-arg `BNODE()` / `UUID()` / `STRUUID()` forms remain
    /// non-deterministic and stay out of `BindExpr` (they require
    /// the extension skolemization track).
    BNode(Box<BindExpr>),
}

impl BindExpr {
    /// All variables referenced anywhere inside the expression.
    #[must_use]
    pub fn variables(&self) -> BTreeSet<VariableSymbol> {
        let mut out = BTreeSet::new();
        self.collect_variables(&mut out);
        out
    }

    fn collect_variables(&self, out: &mut BTreeSet<VariableSymbol>) {
        match self {
            Self::Term(term) => {
                term.collect_variables(out);
            }
            Self::Add(lhs, rhs)
            | Self::Sub(lhs, rhs)
            | Self::Mul(lhs, rhs)
            | Self::Div(lhs, rhs) => {
                lhs.collect_variables(out);
                rhs.collect_variables(out);
            }
            Self::Neg(inner)
            | Self::Abs(inner)
            | Self::Ceil(inner)
            | Self::Floor(inner)
            | Self::Round(inner)
            | Self::Str(inner)
            | Self::StrLen(inner)
            | Self::UCase(inner)
            | Self::LCase(inner)
            | Self::EncodeForUri(inner)
            | Self::Md5(inner)
            | Self::Sha1(inner)
            | Self::Sha256(inner)
            | Self::Sha384(inner)
            | Self::Sha512(inner)
            | Self::CastInteger(inner)
            | Self::CastDecimal(inner)
            | Self::CastDouble(inner)
            | Self::CastFloat(inner)
            | Self::CastString(inner)
            | Self::Year(inner)
            | Self::Month(inner)
            | Self::Day(inner)
            | Self::Hours(inner)
            | Self::Minutes(inner)
            | Self::Seconds(inner)
            | Self::Timezone(inner)
            | Self::Tz(inner)
            | Self::Lang(inner)
            | Self::Iri(inner)
            | Self::BNode(inner) => inner.collect_variables(out),
            Self::Now => {}
            Self::Concat(parts) => {
                for part in parts {
                    part.collect_variables(out);
                }
            }
            Self::SubStr { value, start, len } => {
                value.collect_variables(out);
                start.collect_variables(out);
                if let Some(len) = len {
                    len.collect_variables(out);
                }
            }
            Self::StrBefore(value, needle) | Self::StrAfter(value, needle) => {
                value.collect_variables(out);
                needle.collect_variables(out);
            }
            Self::Replace {
                value,
                pattern,
                replacement,
                flags,
            } => {
                value.collect_variables(out);
                pattern.collect_variables(out);
                replacement.collect_variables(out);
                if let Some(flags) = flags {
                    flags.collect_variables(out);
                }
            }
        }
    }
}
