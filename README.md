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
computes predicate strata for programs with negation.

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
