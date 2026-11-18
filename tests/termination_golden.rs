//! Exact JSON, Postcard and `Display` bytes of a weak-acyclicity violation.
//! Regenerate with `UPDATE_GOLDEN=1 cargo test --test termination_golden`.
mod support;

use std::path::PathBuf;

use datalog_structural_model::{Program, TerminationViolation};
use support::build::{atom, rule, skolem, var};

/// `b(SKOLEM(b, x)) :- a(x)` and `a(y) :- b(y)`: a Skolem cycle through an
/// ordinary rule.
fn violation() -> TerminationViolation {
    Program {
        rules: vec![
            rule(
                atom("b", vec![skolem("b", vec![var("x")])]),
                vec![atom("a", vec![var("x")])],
            ),
            rule(atom("a", vec![var("y")]), vec![atom("b", vec![var("y")])]),
        ],
    }
    .check_weak_acyclicity()
    .unwrap_err()
}

fn check_golden(extension: &str, actual: &[u8]) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("termination_violation.{extension}"));
    if std::env::var_os("UPDATE_GOLDEN").is_some_and(|value| value == "1") {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("{}: {error}; run with UPDATE_GOLDEN=1", path.display()));
    assert_eq!(actual, expected.as_slice(), "{} drifted", path.display());
}

#[test]
fn violation_json_matches_golden() {
    let mut json = serde_json::to_string_pretty(&violation()).unwrap();
    json.push('\n');
    check_golden("json", json.as_bytes());
    assert_eq!(
        serde_json::from_str::<TerminationViolation>(&json).unwrap(),
        violation()
    );
}

#[test]
fn violation_postcard_matches_golden() {
    let bytes = postcard::to_allocvec(&violation()).unwrap();
    check_golden("postcard", &bytes);
    assert_eq!(
        postcard::from_bytes::<TerminationViolation>(&bytes).unwrap(),
        violation()
    );
}

#[test]
fn violation_display_matches_golden() {
    let mut text = violation().to_string();
    text.push('\n');
    check_golden("txt", text.as_bytes());
}
