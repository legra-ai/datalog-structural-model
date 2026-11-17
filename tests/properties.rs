//! Property tests over generated programs: serialization round trips,
//! checked construction agreeing with validation, and stratification
//! invariance under rule permutation.
use datalog_structural_model::{
    Atom, BindExpr, BuiltinFilter, ConstantSymbol, Literal, PredicateSymbol, Program, Rule,
    RuleIndex, Stratification, StrictDependencyKind, Term, UnstratifiableCycle, VariableSymbol,
};
use proptest::prelude::*;

const VARIABLES: [&str; 3] = ["a", "b", "c"];
const CONSTANTS: [&str; 2] = ["k0", "k1"];
const PREDICATES: [&str; 4] = ["p0", "p1", "p2", "p3"];

fn term() -> impl Strategy<Value = Term> {
    prop_oneof![
        3 => prop::sample::select(&VARIABLES[..])
            .prop_map(|name| Term::Variable(VariableSymbol::new(name))),
        1 => prop::sample::select(&CONSTANTS[..])
            .prop_map(|name| Term::Constant(ConstantSymbol::new(name))),
    ]
}

fn atom() -> impl Strategy<Value = Atom> {
    (
        prop::sample::select(&PREDICATES[..]),
        prop::collection::vec(term(), 2),
    )
        .prop_map(|(predicate, terms)| Atom {
            predicate: PredicateSymbol::new(predicate),
            terms,
        })
}

/// A term slot that usually reuses a positive-body term (`choice`) and
/// occasionally (`wild`) takes an arbitrary, possibly unbound term.
fn slot() -> impl Strategy<Value = (u8, bool, Term)> {
    (any::<u8>(), prop::bool::weighted(0.1), term())
}

fn anchored_atom() -> impl Strategy<Value = (&'static str, Vec<(u8, bool, Term)>)> {
    (
        prop::sample::select(&PREDICATES[..]),
        prop::collection::vec(slot(), 2),
    )
}

fn anchor(predicate: &str, slots: Vec<(u8, bool, Term)>, bound: &[Term]) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms: slots
            .into_iter()
            .map(|(choice, wild, term)| {
                if wild {
                    term
                } else {
                    bound[usize::from(choice) % bound.len()].clone()
                }
            })
            .collect(),
    }
}

/// Heads and negated atoms mostly reuse positive-body terms so a useful
/// share of generated rules is safe, while some are deliberately not.
fn rule() -> impl Strategy<Value = Rule> {
    (
        prop::collection::vec(atom(), 1..3),
        prop::collection::vec(anchored_atom(), 0..2),
        anchored_atom(),
        prop::option::weighted(0.2, term()),
        prop::bool::weighted(0.2),
    )
        .prop_map(|(positives, negatives, (head, slots), filter, bind)| {
            let bound: Vec<Term> = positives
                .iter()
                .flat_map(|atom| atom.terms.iter().cloned())
                .collect();
            let head = anchor(head, slots, &bound);
            let mut body: Vec<Literal> = positives.into_iter().map(Literal::Positive).collect();
            body.extend(
                negatives
                    .into_iter()
                    .map(|(predicate, slots)| Literal::Negative(anchor(predicate, slots, &bound))),
            );
            if let Some(term) = filter {
                body.push(Literal::Filter(BuiltinFilter::Ne(term, bound[0].clone())));
            }
            if bind {
                body.push(Literal::Bind {
                    var: VariableSymbol::new("fresh"),
                    expr: BindExpr::Term(bound[0].clone()),
                });
            }
            Rule::new(head, body)
        })
}

fn program() -> impl Strategy<Value = Program> {
    prop::collection::vec(rule(), 0..6).prop_map(|rules| Program { rules })
}

fn permuted_program() -> impl Strategy<Value = (Program, Vec<usize>)> {
    program().prop_flat_map(|program| {
        let order: Vec<usize> = (0..program.rules.len()).collect();
        (Just(program), Just(order).prop_shuffle())
    })
}

/// The reported edge exists in `program`: the producer head shares the
/// consumer literal's predicate and both rules sit in the reported cycle.
fn assert_edge_is_real(program: &Program, cycle: &UnstratifiableCycle) {
    let edge = &cycle.edge;
    assert!(cycle.rules.contains(&edge.producer) && cycle.rules.contains(&edge.consumer));
    let mut literals = &program.rules[edge.consumer.get()].body;
    let steps = edge.literal.steps();
    for step in &steps[..steps.len() - 1] {
        let Literal::Aggregate(aggregate) = &literals[step.get()] else {
            panic!("path step {step:?} is not an aggregate");
        };
        literals = &aggregate.inner_body;
    }
    let literal = &literals[steps[steps.len() - 1].get()];
    let atom = literal.atom().expect("edge literal carries an atom");
    assert_eq!(
        atom.predicate,
        program.rules[edge.producer.get()].head.predicate
    );
    match edge.kind {
        StrictDependencyKind::Negation => assert!(literal.is_negative()),
        StrictDependencyKind::Aggregation => assert!(steps.len() > 1),
    }
}

fn assert_remapped(original: &Stratification, permuted: &Stratification, new_of: &[usize]) {
    assert_eq!(original.strata.len(), permuted.strata.len());
    for (old, &stratum) in original.rule_stratum.iter().enumerate() {
        assert_eq!(permuted.rule_stratum[new_of[old]], stratum, "rule {old}");
    }
    for (old_stratum, new_stratum) in original.strata.iter().zip(&permuted.strata) {
        let remapped = old_stratum
            .rules
            .iter()
            .map(|rule| RuleIndex::new(new_of[rule.get()]))
            .collect();
        assert_eq!(new_stratum.rules, remapped);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn json_and_postcard_round_trip(program in program()) {
        let json = serde_json::to_string(&program).unwrap();
        let bytes = postcard::to_allocvec(&program).unwrap();
        let from_json = serde_json::from_str::<Program>(&json);
        let from_postcard = postcard::from_bytes::<Program>(&bytes);
        if program.validate().is_ok() {
            prop_assert_eq!(from_json.unwrap(), program.clone());
            prop_assert_eq!(from_postcard.unwrap(), program);
        } else {
            prop_assert!(from_json.is_err());
            prop_assert!(from_postcard.is_err());
        }
    }

    #[test]
    fn try_new_accepts_exactly_what_validation_accepts(program in program()) {
        let checked = Program::try_new(program.rules.clone());
        prop_assert_eq!(checked.as_ref().map(|_| ()).map_err(Clone::clone), program.validate());
        if let Ok(checked) = checked {
            prop_assert_eq!(checked, program);
        }
    }

    #[test]
    fn stratification_is_invariant_under_rule_permutation(
        (program, order) in permuted_program()
    ) {
        let permuted = Program {
            rules: order.iter().map(|&old| program.rules[old].clone()).collect(),
        };
        let mut new_of = vec![0; order.len()];
        for (new, &old) in order.iter().enumerate() {
            new_of[old] = new;
        }
        match (program.stratify(), permuted.stratify()) {
            (Ok(original), Ok(reordered)) => assert_remapped(&original, &reordered, &new_of),
            (Err(original), Err(reordered)) => {
                let datalog_structural_model::ProgramValidationError::UnstratifiableCycle(a) =
                    original
                else {
                    panic!("stratify only reports cycles");
                };
                let datalog_structural_model::ProgramValidationError::UnstratifiableCycle(b) =
                    reordered
                else {
                    panic!("stratify only reports cycles");
                };
                assert_edge_is_real(&program, &a);
                assert_edge_is_real(&permuted, &b);
            }
            (original, reordered) => {
                prop_assert!(false, "{original:?} vs {reordered:?}");
            }
        }
    }
}
