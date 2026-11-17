//! Exact JSON and Postcard bytes for a corpus of programs. Regenerate with
//! `UPDATE_GOLDEN=1 cargo test --test golden_corpus`.
use std::path::PathBuf;

use datalog_structural_model::{
    AggregateBody, Aggregator, Atom, BindExpr, BuiltinFilter, ConstantSymbol, Literal,
    PredicateSymbol, Program, Rule, Term, VariableSymbol,
};

fn var(name: &str) -> Term {
    Term::Variable(VariableSymbol::new(name))
}

fn constant(name: &str) -> Term {
    Term::Constant(ConstantSymbol::new(name))
}

fn atom(predicate: &str, terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms,
    }
}

fn ancestor() -> Program {
    Program::try_new(vec![
        Rule::positive(
            atom("ancestor", vec![var("x"), var("y")]),
            vec![atom("parent", vec![var("x"), var("y")])],
        ),
        Rule::positive(
            atom("ancestor", vec![var("x"), var("z")]),
            vec![
                atom("parent", vec![var("x"), var("y")]),
                atom("ancestor", vec![var("y"), var("z")]),
            ],
        ),
    ])
    .unwrap()
}

fn stratified_negation() -> Program {
    Program::try_new(vec![Rule::new(
        atom("bachelor", vec![var("x")]),
        vec![
            Literal::Positive(atom("person", vec![var("x")])),
            Literal::Negative(atom("married", vec![var("x")])),
        ],
    )])
    .unwrap()
}

fn filter_and_bind() -> Program {
    Program::try_new(vec![Rule::new(
        atom("label", vec![var("p"), var("upper")]),
        vec![
            Literal::Positive(atom("name", vec![var("p"), var("n")])),
            Literal::Filter(BuiltinFilter::Ne(var("n"), constant(""))),
            Literal::Bind {
                var: VariableSymbol::new("upper"),
                expr: BindExpr::UCase(Box::new(BindExpr::Term(var("n")))),
            },
        ],
    )])
    .unwrap()
}

fn aggregate() -> Program {
    Program::try_new(vec![Rule::new(
        atom("payroll", vec![var("dept"), var("total")]),
        vec![Literal::Aggregate(AggregateBody {
            result_var: VariableSymbol::new("total"),
            aggregator: Aggregator::Sum,
            input_var: VariableSymbol::new("sal"),
            group_by: vec![VariableSymbol::new("dept")],
            inner_body: vec![Literal::Positive(atom(
                "salary",
                vec![var("e"), var("dept"), var("sal")],
            ))],
        })],
    )])
    .unwrap()
}

fn triple_terms_and_skolem() -> Program {
    let quoted = Term::TripleTerm(Box::new(atom("knows", vec![var("a"), var("b")])));
    Program::try_new(vec![Rule::positive(
        atom(
            "claim",
            vec![
                Term::Skolem {
                    name: ConstantSymbol::new("claim-id"),
                    args: vec![var("s"), constant("v1")],
                },
                quoted.clone(),
            ],
        ),
        vec![atom("asserted", vec![var("s"), quoted])],
    )])
    .unwrap()
}

fn corpus() -> Vec<(&'static str, Program)> {
    vec![
        ("ancestor", ancestor()),
        ("stratified_negation", stratified_negation()),
        ("filter_and_bind", filter_and_bind()),
        ("aggregate", aggregate()),
        ("triple_terms_and_skolem", triple_terms_and_skolem()),
        ("empty", Program::try_new(vec![]).unwrap()),
    ]
}

fn golden_path(name: &str, extension: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("{name}.{extension}"))
}

fn check_golden(name: &str, extension: &str, actual: &[u8]) {
    let path = golden_path(name, extension);
    if std::env::var_os("UPDATE_GOLDEN").is_some_and(|value| value == "1") {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("{}: {error}; run with UPDATE_GOLDEN=1", path.display()));
    assert_eq!(actual, expected.as_slice(), "{} drifted", path.display());
}

#[test]
fn json_encoding_matches_goldens() {
    for (name, program) in corpus() {
        let mut json = serde_json::to_string_pretty(&program).unwrap();
        json.push('\n');
        check_golden(name, "json", json.as_bytes());
        assert_eq!(
            serde_json::from_str::<Program>(&json).unwrap(),
            program,
            "{name}"
        );
    }
}

#[test]
fn postcard_encoding_matches_goldens() {
    for (name, program) in corpus() {
        let bytes = postcard::to_allocvec(&program).unwrap();
        check_golden(name, "postcard", &bytes);
        assert_eq!(
            postcard::from_bytes::<Program>(&bytes).unwrap(),
            program,
            "{name}"
        );
    }
}

#[test]
fn goldens_decode_to_the_corpus() {
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        return;
    }
    for (name, program) in corpus() {
        let json = std::fs::read(golden_path(name, "json")).unwrap();
        assert_eq!(
            serde_json::from_slice::<Program>(&json).unwrap(),
            program,
            "{name}"
        );
        let bytes = std::fs::read(golden_path(name, "postcard")).unwrap();
        assert_eq!(
            postcard::from_bytes::<Program>(&bytes).unwrap(),
            program,
            "{name}"
        );
    }
}
