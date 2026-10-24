//! Dependency depth must not consume the native call stack.

use std::{env, process::Command, thread};

use datalog_structural_model::{Atom, Literal, PredicateSymbol, Program, Rule};

#[test]
fn deep_dependency_chain_uses_heap_traversal() {
    const CHILD: &str = "DATALOG_DEEP_DEPENDENCIES_CHILD";
    if env::var_os(CHILD).is_some() {
        thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                // bounded: fixed 4096-rule stress fixture, no data relation.
                let rules = (1..=4096)
                    .map(|index| {
                        let head = Atom {
                            predicate: PredicateSymbol::new(format!("p{index:05}")),
                            terms: vec![],
                        };
                        let input = Atom {
                            predicate: PredicateSymbol::new(format!("p{:05}", index - 1)),
                            terms: vec![],
                        };
                        Rule::new(head, vec![Literal::Negative(input)])
                    })
                    .collect();
                let program = Program { rules };
                let strata = program.stratify().unwrap();
                assert_eq!(strata.strata.len(), 4097);
                assert_eq!(
                    strata.stratum_of(&PredicateSymbol::new("p04096")),
                    Some(4096)
                );
            })
            .unwrap()
            .join()
            .unwrap();
        return;
    }
    let output = Command::new(env::current_exe().unwrap())
        .args([
            "--exact",
            "deep_dependency_chain_uses_heap_traversal",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "dependency traversal child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
