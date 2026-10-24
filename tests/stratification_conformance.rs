//! Exhaustive small graphs compared with an independent constraint solver.

use datalog_structural_model::{
    Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term,
};

const PREDICATES: [&str; 3] = ["p", "q", "r"];

struct Dependencies([[u8; 3]; 3]);

impl Dependencies {
    fn from_index(mut index: usize) -> Self {
        let mut matrix = [[0; 3]; 3];
        for row in &mut matrix {
            for edge in row {
                *edge = (index % 3) as u8;
                index /= 3;
            }
        }
        Self(matrix)
    }

    /// Boolean transitive closure detects negative cycles; repeated constraint
    /// relaxation computes the least strata without using SCCs or DFS.
    fn reference_strata(&self) -> Option<[usize; 3]> {
        let mut reachable = [[false; 3]; 3];
        for (source, row) in self.0.iter().enumerate() {
            for (target, edge) in row.iter().enumerate() {
                reachable[source][target] = *edge != 0 || source == target;
            }
        }
        for via in 0..3 {
            for source in 0..3 {
                for target in 0..3 {
                    reachable[source][target] |= reachable[source][via] && reachable[via][target];
                }
            }
        }
        for (source, row) in self.0.iter().enumerate() {
            for (target, edge) in row.iter().enumerate() {
                if *edge == 2 && reachable[target][source] {
                    return None;
                }
            }
        }
        let mut strata = [0; 3];
        // Any simple path has at most two edges; three rounds also cover
        // arbitrary edge visitation order and zero-weight positive cycles.
        for _ in 0..3 {
            let previous = strata;
            for (source, row) in self.0.iter().enumerate() {
                for (target, edge) in row.iter().enumerate() {
                    if *edge != 0 {
                        strata[target] =
                            strata[target].max(previous[source] + usize::from(*edge == 2));
                    }
                }
            }
        }
        Some(strata)
    }

    fn program(&self, with_contexts: bool) -> Program {
        // bounded: three seed rules plus at most nine dependency rules.
        let mut rules: Vec<_> = (0..3)
            .map(|predicate| Rule::positive(Self::atom(predicate, with_contexts, "seed"), vec![]))
            .collect();
        for (source, row) in self.0.iter().enumerate() {
            for (target, edge) in row.iter().enumerate() {
                let atom = Self::atom(source, with_contexts, "input");
                let body = match edge {
                    0 => continue,
                    1 => Literal::Positive(atom),
                    2 => Literal::Negative(atom),
                    _ => unreachable!("base-three edge encoding"),
                };
                rules.push(Rule::new(
                    Self::atom(target, with_contexts, "output"),
                    vec![body],
                ));
            }
        }
        Program { rules }
    }

    fn atom(predicate: usize, with_contexts: bool, context: &str) -> Atom {
        Atom {
            predicate: PredicateSymbol::new(PREDICATES[predicate]),
            terms: if with_contexts {
                vec![Term::Constant(ConstantSymbol::new(context))]
            } else {
                vec![]
            },
        }
    }
}

#[test]
fn every_three_predicate_signed_graph_matches_the_reference() {
    for index in 0..3_usize.pow(9) {
        let dependencies = Dependencies::from_index(index);
        let expected = dependencies.reference_strata();
        for with_contexts in [false, true] {
            let result = dependencies.program(with_contexts).stratify();
            match expected {
                None if !with_contexts => assert!(result.is_err(), "graph {index}"),
                None => assert!(result.is_ok(), "disjoint tuples, graph {index}"),
                Some(expected) => {
                    let actual = result.unwrap_or_else(|error| {
                        panic!("graph {index}, contexts={with_contexts}: {error:?}")
                    });
                    // Seed rules have no dependencies. Each later rule has
                    // one signed edge. Disjoint input/output tuple constants
                    // eliminate every producer edge in the tuple variant.
                    assert_eq!(&actual.rule_stratum[..3], &[0, 0, 0]);
                    let mut rule_index = 3;
                    for (source, row) in dependencies.0.iter().enumerate() {
                        for edge in row {
                            if *edge == 0 {
                                continue;
                            }
                            let level = if with_contexts {
                                0
                            } else {
                                expected[source] + usize::from(*edge == 2)
                            };
                            assert_eq!(
                                actual.stratum_of_rule(rule_index),
                                Some(level),
                                "graph {index}, tuples={with_contexts}, rule={rule_index}"
                            );
                            rule_index += 1;
                        }
                    }
                }
            }
        }
    }
}
