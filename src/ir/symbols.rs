//! Predicate, variable, and constant symbols used by the
//! Datalog IR.
//!
//! All three carry their textual identity as `Arc<str>` so the
//! evaluator hot-path binding clones and trail rollback
//! becomes a refcount bump instead of a `String::clone` →
//! memmove. The wire / debug surface is unchanged; constructors
//! still accept `impl Into<String>` so callers don't see the
//! switch.
//!
//! [`VariableSymbol`] additionally caches a precomputed `FxHash`
//! of its name so `HashMap` lookups on the forward engine's
//! `Binding` skip rehashing the name string every time. The
//! cached value is `state.write_u64`-fed into whatever hasher
//! the map uses (Fx today, deterministic per-name regardless).
//! Trail/profile justification: samply showed
//! `Binding::get` (`FxHash` + memcmp) costing ~15 % of total
//! samples on a rule-materialization benchmark after replacing
//! `SipHash` with `FxHashMap`.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use rustc_hash::FxHasher;

/// Predicate symbol in a Datalog atom.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PredicateSymbol(Arc<str>);

impl PredicateSymbol {
    /// Construct a predicate symbol.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Arc::from(value.into()))
    }

    /// Access the raw predicate identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Variable symbol used in Datalog terms.
///
/// Carries a precomputed `FxHash` of its name alongside the
/// `Arc<str>` so [`HashMap`]-keyed lookups (notably the forward
/// engine bindings) hash a single `u64` instead of walking
/// the name bytes on every probe.
#[derive(Clone, Debug)]
pub struct VariableSymbol {
    name: Arc<str>,
    cached_hash: u64,
}

impl VariableSymbol {
    /// Construct a variable symbol. Computes the cached hash of
    /// `name` once; subsequent `Hash` calls feed that `u64` to
    /// whatever hasher the map uses.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let name: Arc<str> = Arc::from(value.into());
        let cached_hash = {
            let mut h = FxHasher::default();
            h.write(name.as_bytes());
            h.finish()
        };
        Self { name, cached_hash }
    }

    /// Access the raw variable identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl PartialEq for VariableSymbol {
    fn eq(&self, other: &Self) -> bool {
        // Hash inequality is a fast 8-byte prefilter that lets
        // hash hits short-circuit before walking the full name.
        // True equality still requires identical names because
        // the cached hash is derived from them; the inequality
        // case is the common one on HashMap probes.
        self.cached_hash == other.cached_hash && self.name == other.name
    }
}

impl Eq for VariableSymbol {}

impl Hash for VariableSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.cached_hash);
    }
}

impl PartialOrd for VariableSymbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VariableSymbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Stable, content-addressed ordering — compare names, not
        // hashes, so `BTreeSet` / `BTreeMap` order is independent
        // of the hasher seed.
        self.name.cmp(&other.name)
    }
}

/// Constant symbol used in Datalog terms.
///
/// Like [`VariableSymbol`], carries a precomputed `FxHash` of its
/// value alongside the `Arc<str>`. Constants often dominate
/// evaluator hot paths — a hash-join frontier index
/// (`FxHashMap<GroundTerm, _>`) hashes a constant on every probe,
/// and `unify_one` compares constants on every fact cell — so a
/// cached hash (fed as a single `u64`) plus a hash-prefiltered
/// equality cut the per-fact term cost. Profile justification:
/// profiling of a bulk rule-materialization benchmark showed
/// `GroundTerm` `PartialEq` plus hash-join
/// index's `GroundTerm` hashing at ~20 % of total samples, all
/// content-based on the constant `Arc<str>`.
///
/// Ordering stays content-based so `BTreeSet` / `BTreeMap` order
/// (a relation prefix index may rely on it for range queries) is
/// independent of the hasher seed — the same split
/// [`VariableSymbol`] makes.
#[derive(Clone, Debug)]
pub struct ConstantSymbol {
    value: Arc<str>,
    cached_hash: u64,
}

impl ConstantSymbol {
    /// Construct a constant symbol. Computes the cached hash of
    /// `value` once; subsequent `Hash` calls feed that `u64` to
    /// whatever hasher the map uses.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let value: Arc<str> = Arc::from(value.into());
        let cached_hash = {
            let mut h = FxHasher::default();
            h.write(value.as_bytes());
            h.finish()
        };
        Self { value, cached_hash }
    }

    /// Access the raw constant identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl PartialEq for ConstantSymbol {
    fn eq(&self, other: &Self) -> bool {
        // Hash inequality is a fast 8-byte prefilter; true equality
        // still requires identical values (the cached hash is
        // derived from them). Inequality is the common case on the
        // hash-join probe and on `unify_one`'s constant comparisons.
        self.cached_hash == other.cached_hash && self.value == other.value
    }
}

impl Eq for ConstantSymbol {}

impl Hash for ConstantSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.cached_hash);
    }
}

impl PartialOrd for ConstantSymbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConstantSymbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Content-addressed ordering — compare values, not hashes,
        // so `BTreeSet` / `BTreeMap` order is deterministic and
        // hasher-independent.
        self.value.cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};

    use super::*;

    /// The hand-written `Eq`/`Hash` must keep the std invariant
    /// `a == b ⟹ hash(a) == hash(b)`: equal values resolve to the
    /// same `HashMap` slot regardless of which `Arc` allocation they
    /// came from.
    #[test]
    fn equal_constants_hash_and_compare_equal() {
        let a = ConstantSymbol::new("http://ex.org/x");
        let b = ConstantSymbol::new(String::from("http://ex.org/x"));
        assert_eq!(a, b);

        let mut map: HashMap<ConstantSymbol, u32> = HashMap::new();
        map.insert(a, 7);
        assert_eq!(map.get(&b), Some(&7), "equal values must hash to one slot");
    }

    /// Distinct values stay distinct (the cached-hash prefilter is an
    /// optimisation, not a substitute for value equality).
    #[test]
    fn distinct_constants_are_unequal() {
        let a = ConstantSymbol::new("http://ex.org/x");
        let b = ConstantSymbol::new("http://ex.org/y");
        assert_ne!(a, b);
    }

    /// Ordering stays content-based (lexicographic on the value), so
    /// `BTreeSet`/`BTreeMap` order is independent of the cached hash —
    /// the property the `FactSet` prefix-range index relies on.
    #[test]
    fn ordering_is_content_based() {
        let set: BTreeSet<ConstantSymbol> = ["c", "a", "b"]
            .into_iter()
            .map(ConstantSymbol::new)
            .collect();
        let ordered: Vec<&str> = set.iter().map(ConstantSymbol::as_str).collect();
        assert_eq!(ordered, ["a", "b", "c"]);
    }
}
