# datalog-structural-model

[![Crates.io](https://img.shields.io/crates/v/datalog-structural-model.svg?cacheSeconds=300)](https://crates.io/crates/datalog-structural-model)
[![Documentation](https://docs.rs/datalog-structural-model/badge.svg?cacheSeconds=300)](https://docs.rs/datalog-structural-model)
[![CI](https://github.com/legra-ai/datalog-structural-model/actions/workflows/ci.yml/badge.svg?branch=main&cacheSeconds=300)](https://github.com/legra-ai/datalog-structural-model/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/crates/d/datalog-structural-model.svg?cacheSeconds=300)](https://crates.io/crates/datalog-structural-model)
[![License](https://img.shields.io/crates/l/datalog-structural-model.svg?cacheSeconds=300)](LICENSE-APACHE)

Typed Datalog rule programs, validation, and stratification analysis.

`datalog-structural-model` is the finite rule-program layer between a rule authoring or
compiler front-end and an evaluator. It models predicates, variables, terms,
atoms, rules, filters, bindings, and aggregates, then validates safety and
computes rule strata over complete tuple patterns for programs with negation.

It does not read input data, execute rules, retain a fact store, or collect
solutions. Evaluation engines can consume a validated `Program` while keeping
facts and derived results in their own asynchronous, backpressured, and
spill-capable data paths.

## A small rule program

```rust
use datalog_structural_model::{
    Atom,
    ConstantSymbol,
    Literal,
    PredicateSymbol,
    Program,
    Rule,
    Term,
    VariableSymbol,
};

let person = PredicateSymbol::new("person");
let ancestor = PredicateSymbol::new("ancestor");
let x = VariableSymbol::new("x");
let y = VariableSymbol::new("y");
let z = VariableSymbol::new("z");

let parent_xy = Atom {
    predicate: PredicateSymbol::new("parent"),
    terms: vec![Term::Variable(x.clone()), Term::Variable(y.clone())],
};
let ancestor_yz = Atom {
    predicate: ancestor.clone(),
    terms: vec![Term::Variable(y), Term::Variable(z.clone())],
};
let ancestor_xz = Atom {
    predicate: ancestor,
    terms: vec![Term::Variable(x), Term::Variable(z)],
};

let program = Program {
    rules: vec![Rule::new(
        ancestor_xz,
        vec![Literal::Positive(parent_xy), Literal::Positive(ancestor_yz)],
    )],
};

program.validate().expect("the rule is safe and stratifiable");
assert_eq!(person.as_str(), "person");
assert_eq!(ConstantSymbol::new("example").as_str(), "example");
```

The rule model is deliberately separate from execution. A caller may build a
program in memory because a rule program is a finite description, but the
facts matched by that program are not part of this crate and must not be
materialized here.

## Validation and stratification

`Program::validate` checks range restriction, variable safety, supported
expression structure, and the absence of cycles through negation. A valid
program can be passed to a forward or backward evaluator. `Program::stratify`
exposes the calculated strata when an evaluator needs to schedule them.

The dependency graph uses strongly connected components and preserves the
distinction between positive and negative dependencies. A recursive positive
cycle is valid; a cycle that contains a negated dependency is rejected.


### Error locations and codes

Every validation error that concerns a location names it with typed
positions: a `RuleIndex`, a `RuleSite` (head or body), a `LiteralPath` (the
body literal, then one step per enclosing aggregate), and a `TermPath` (an
`AtomSlot` or `ExpressionTermIndex`, then one argument slot per nested triple
term or Skolem call). A cycle through negation or aggregation reports an
`UnstratifiableCycle`: the rules and predicates of the component plus the
offending `StrictDependency` — producer rule, consumer rule, consumer literal,
and whether the edge is negation or aggregation.

Each error variant has a stable kebab-case `ErrorCode` (`error.code()`, also
available as `CODE_*` associated constants), such as `unsafe-negation` or
`unstratifiable-cycle`. Codes are pinned by a census test and do not change
when a variant is renamed.

```rust
use datalog_structural_model::{
    Atom, Literal, LiteralIndex, LiteralPath, PredicateSymbol, Program,
    ProgramValidationError, Rule, RuleIndex, Term, VariableSymbol,
};

let x = Term::Variable(VariableSymbol::new("x"));
let y = Term::Variable(VariableSymbol::new("y"));
let atom = |p: &str, terms: Vec<Term>| Atom { predicate: PredicateSymbol::new(p), terms };
let error = Program::try_new(vec![Rule::new(
    atom("lonely", vec![x.clone()]),
    vec![
        Literal::Positive(atom("person", vec![x.clone()])),
        Literal::Negative(atom("knows", vec![x, y])),
    ],
)])
.unwrap_err();
assert_eq!(error.code().as_str(), "unsafe-negation");
let ProgramValidationError::UnsafeNegation { rule, literal, .. } = error else {
    unreachable!()
};
assert_eq!(rule, RuleIndex::new(0));
assert_eq!(literal, LiteralPath::top(LiteralIndex::new(1)));
```

## Serialization and checked loading

All public semantic data supports Serde directly, including symbols, nested
terms, expressions, aggregates, rules, programs, profiles and diagnostics.
JSON and Postcard are exercised; the model does not use a JSON intermediate.
Symbols serialize as exact text. Cached hashes and allocation identity are
rebuilt through the symbol constructors and never appear in serialized data.

`Program::try_new` and `Rule::try_new` validate checked construction. Their
Serde inputs use those same paths: unsafe variables, invalid Skolem placement,
excessive term nesting and cycles through negation fail on input. Standalone
`Term` and `GroundTerm` decoding checks context-free structure; rule position
and variable binding still require program validation. Raw public fields remain
available for authoring invalid source descriptions and reporting their errors;
those descriptions cannot bypass checked loading into executable programs.
Unknown semantic fields and enum variants are rejected.

```rust
use datalog_structural_model::{
    Atom, ConstantSymbol, DatalogProfile, PredicateSymbol, Program, Rule, Term,
};

let atom = Atom {
    predicate: PredicateSymbol::new("person"),
    terms: vec![Term::Constant(ConstantSymbol::new("alice"))],
};
let program = Program::try_new(vec![Rule::positive(atom, vec![])])?;
let profile = DatalogProfile::positive();
program.validate_for(&profile)?;
let bytes = postcard::to_allocvec(&program)?;
let loaded: Program = postcard::from_bytes(&bytes)?;
loaded.validate_for(&profile)?;
assert_eq!(program, loaded);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Serialized forms are pinned by exact JSON and Postcard golden files under
`tests/golden`; regenerate them deliberately with
`UPDATE_GOLDEN=1 cargo test --test golden_corpus`.

Stratification results serialize as reports. Recompute stratification from the
validated program when scheduling execution; loading a report does not certify
an executable plan. Serde encodings are neither canonical hashes (see `ProgramDigest`) nor RDF
concrete syntaxes. Callers own decoding resource budgets; this crate owns structural and
program-safety validation. Structural ceilings are typed `ValidationLimits`
(see below).

## Termination: weak acyclicity

Skolem terms, head triple terms, `BIND` and aggregates invent values, so a
recursive program can derive facts forever. `Program::check_weak_acyclicity`
builds the position graph and returns `Result<WeakAcyclicity,
TerminationViolation>`. Nodes are argument slots of atom occurrences
(`ArgumentPosition`: predicate, `AtomSlot`, `RuleIndex`, `RuleSite`). A head
variable copied from a body slot adds an ordinary edge; a value invented in the
head adds a special edge from every body slot feeding it. A head slot feeds a
body slot only when the two atoms can match the same fact, so constants that
keep patterns apart also break cycles. The analysis reads only the program,
reuses the stratifier's Tarjan traversal, and is bounded by program size, never
by data.

A program is weakly acyclic when no special edge lies on a strongly connected
component. The certificate reports `skolem_depth()`, the most special edges on
any path. A violation carries the first cyclic `SpecialEdge` in canonical
order, its sorted component, and the stable code `not-weakly-acyclic`. The
criterion is a versioned `#[non_exhaustive]` `TerminationCriterion`
(`WeaklyAcyclic`, `label()` `weakly-acyclic`, `law_version()` 1). It is
sufficient, not necessary: a violation is not a proof of divergence.

```rust
use datalog_structural_model::{
    Atom, ConstantSymbol, InventionKind, PredicateSymbol, Program, Rule, Term, VariableSymbol,
};

let x = Term::Variable(VariableSymbol::new("x"));
let next = Term::Skolem { name: ConstantSymbol::new("next"), args: vec![x.clone()] };
let node = |term| Atom { predicate: PredicateSymbol::new("node"), terms: vec![term] };
let program = Program { rules: vec![Rule::positive(node(next), vec![node(x)])] };
let violation = program.check_weak_acyclicity().unwrap_err();
assert_eq!(violation.code().as_str(), "not-weakly-acyclic");
assert_eq!(violation.edge().kind, InventionKind::Skolem);
```

## Validation limits

`ValidationLimits` bounds triple-term nesting depth (standard 4), rule count
(65,536), literals per rule including aggregate bodies (1,024) and Skolem depth
(64). `Program::validate` uses the standard limits; `Program::validate_within`
takes explicit ones. Each `DatalogProfile` owns its limits (`with_limits`,
`limits`), and `Program::validate_for` enforces them. It also requires weak
acyclicity (`ProfileCheckError::TerminationUncertified`) and a certified Skolem
depth within the limit. Limit errors (`rule-count-exceeded`,
`body-length-exceeded`, `triple-term-nesting-exceeded`, `skolem-depth-exceeded`)
report the limit and the value reached.

## Program and contract digests

`Program::digest` returns a `ProgramDigest`: a 32-byte BLAKE3 content identity
of the program as authored. It is computed from a canonical preimage, never
from a Serde encoding, so JSON, Postcard and in-memory construction of the same
program give the same digest.

The preimage starts with the versioned domain tag
`datalog-structural-model/program-digest/v1`, then writes fields in a fixed
order. Every text field and every sequence is prefixed with its length as a
`u64` little-endian value, and every enum variant writes its own tag byte. As a
result:

- rule order, body literal order, argument order, group-by order and
  aggregate inner-body order are significant;
- variables, constants, triple terms and Skolem terms are distinct, so a
  variable `x` never collides with a constant `x`;
- positive, negative, filter, bind and aggregate literals and every filter,
  expression and aggregator operator are distinct.

`ValidationContract` pairs an `EqualityMode`, `ValidationLimits` and a
`TerminationCriterion` (with its law version). `ValidationContract::digest_for`
returns a `ContractDigest` over the program digest and those parameters under
the tag `datalog-structural-model/contract-digest/v1`. The program digest
excludes them on purpose: the same program checked under different limits or
termination laws is still the same program, while a cache of validation
outcomes keys on the contract digest.

Both digests display as 64 lowercase hex characters and parse back with
`FromStr`. Digests are published identities, frozen by
`tests/digest_golden.rs`. Any change to the preimage introduces a new
domain-tag version; it never silently moves an existing digest.

```rust
use datalog_structural_model::{
    Atom, EqualityMode, PredicateSymbol, Program, ProgramDigest, Rule, Term,
    TerminationCriterion, ValidationContract, ValidationLimits, VariableSymbol,
};

let x = Term::Variable(VariableSymbol::new("x"));
let atom = |name: &str| Atom { predicate: PredicateSymbol::new(name), terms: vec![x.clone()] };
let program = Program::try_new(vec![Rule::positive(atom("q"), vec![atom("p")])])?;
let digest = program.digest();
assert_eq!(digest.to_string().parse::<ProgramDigest>()?, digest);

let contract = ValidationContract::new(
    EqualityMode::Default,
    ValidationLimits::standard(),
    TerminationCriterion::WeaklyAcyclic,
);
assert_ne!(contract.digest_for(&program).as_bytes(), digest.as_bytes());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Triple-term adapter conventions

`Term::TripleTerm` contains a generic structured `Atom`. The RDF reasoning
catalogue's convention uses a fixed `urn:rdf-reasoning:triple` relation with
three ordered operands `[subject, predicate, object]`, recursively. The RDF
predicate can be a bound variable because it is an operand, distinct from the
fixed Datalog relation symbol. A two-operand atom with its RDF predicate in the
relation symbol is a different structure; the model does not translate it.

Checked construction and nested JSON/Postcard preserve all three operands and
reject unbound variables, including a nested predicate variable. Programs need
the explicit `TripleTerms` capability. These checks do not implement term
construction or unification, validate RDF term domains/scope, or certify an
entailment regime. Adapters own those contracts. Display text is not identity.

## Explicit Datalog language profiles

`DatalogProfile::positive()` describes positive flat Datalog without extension
operators. `DatalogProfile::stratified()` adds stratified negation. Custom profiles
pair an explicit `DatalogCapabilities` set with an equality validation regime.

`Program::required_capabilities()` inspects the entire program, including nested
bindings, filters, triple terms, Skolem terms and aggregate inner bodies. Filter,
binding and aggregate requirements identify exact operations, rather than broad
"supports expressions" flags. `Program::validate_for` rejects unsupported
features with a typed missing-capability set, and retains the existing rejection
of negation/aggregation under equality rewriting.

A capability declaration describes a contract, not proof of engine support.
`DatalogCapabilities::all_model_features()` lists what the IR can represent;
an engine must independently declare and test what it implements. Forward and
backward execution remain separate from the language profile.

RDFS, OWL 2 RL/RDF, Horst and vendor-compatible entailment presets belong in
RDF reasoning rule packages. They can consume this model without adding OWL or
backend dependencies to the generic Datalog layer. This crate does not yet
provide those RDF rules or enable new inference modes in Python packages.

## Scope

This crate contains rule-program data structures and pure validation only. It
does not provide a parser or CST for a Datalog source language, and it does not
define an execution runtime. Those are separate layers so that parsers can
stream source tokens and evaluators can stream facts and results without
turning this IR crate into an in-memory data engine.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
