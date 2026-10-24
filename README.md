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

Stratification results serialize as reports. Recompute stratification from the
validated program when scheduling execution; loading a report does not certify
an executable plan. Serde encodings are neither canonical hashes nor RDF concrete
syntaxes. Callers own decoding resource budgets; this crate owns structural and
program-safety validation. The supported triple-term nesting depth remains
`MAX_TRIPLE_TERM_NESTING`.

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
