//! Property tests of weak acyclicity: programs without value invention are
//! always weakly acyclic, and the verdict ignores rule order.
use datalog_structural_model::{
    Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term, VariableSymbol,
};
use proptest::prelude::*;

const VARIABLES: [&str; 3] = ["a", "b", "c"];
const CONSTANTS: [&str; 2] = ["k0", "k1"];
const PREDICATES: [&str; 3] = ["p0", "p1", "p2"];

fn flat_term() -> impl Strategy<Value = Term> {
    prop_oneof![
        3 => prop::sample::select(&VARIABLES[..])
            .prop_map(|name| Term::Variable(VariableSymbol::new(name))),
        1 => prop::sample::select(&CONSTANTS[..])
            .prop_map(|name| Term::Constant(ConstantSymbol::new(name))),
    ]
}

/// A head term that sometimes invents a value with a Skolem call.
fn head_term() -> impl Strategy<Value = Term> {
    prop_oneof![
        4 => flat_term(),
        1 => prop::collection::vec(flat_term(), 0..2).prop_map(|args| Term::Skolem {
            name: ConstantSymbol::new("s"),
            args,
        }),
    ]
}

fn atom(term: impl Strategy<Value = Term>) -> impl Strategy<Value = Atom> {
    (
        prop::sample::select(&PREDICATES[..]),
        prop::collection::vec(term, 2),
    )
        .prop_map(|(predicate, terms)| Atom {
            predicate: PredicateSymbol::new(predicate),
            terms,
        })
}

fn rule(head: impl Strategy<Value = Term>) -> impl Strategy<Value = Rule> {
    (
        atom(head),
        prop::collection::vec(atom(flat_term()), 1..3),
        prop::option::weighted(0.2, atom(flat_term())),
    )
        .prop_map(|(head, positives, negative)| {
            let mut body: Vec<Literal> = positives.into_iter().map(Literal::Positive).collect();
            body.extend(negative.map(Literal::Negative));
            Rule::new(head, body)
        })
}

fn permuted(head: impl Strategy<Value = Term>) -> impl Strategy<Value = (Program, Vec<usize>)> {
    prop::collection::vec(rule(head), 0..6).prop_flat_map(|rules| {
        let order: Vec<usize> = (0..rules.len()).collect();
        (Just(Program { rules }), Just(order).prop_shuffle())
    })
}

fn reorder(program: &Program, order: &[usize]) -> Program {
    Program {
        rules: order
            .iter()
            .map(|&index| program.rules[index].clone())
            .collect(),
    }
}

proptest! {
    #[test]
    fn programs_without_invention_are_weakly_acyclic(
        rules in prop::collection::vec(rule(flat_term()), 0..8),
    ) {
        let certificate = Program { rules }.check_weak_acyclicity();
        prop_assert!(certificate.is_ok(), "{certificate:?}");
        let certificate = certificate.unwrap();
        prop_assert_eq!(certificate.special_edges(), 0);
        prop_assert_eq!(certificate.skolem_depth(), 0);
    }

    #[test]
    fn rule_order_does_not_change_the_verdict((program, order) in permuted(head_term())) {
        let original = program.check_weak_acyclicity();
        let shuffled = reorder(&program, &order).check_weak_acyclicity();
        match (&original, &shuffled) {
            (Ok(left), Ok(right)) => {
                prop_assert_eq!(left.skolem_depth(), right.skolem_depth());
                prop_assert_eq!(left.special_edges(), right.special_edges());
                prop_assert_eq!(left.positions(), right.positions());
            }
            (Err(left), Err(right)) => {
                prop_assert_eq!(left.code(), right.code());
                prop_assert_eq!(left.component().len(), right.component().len());
            }
            _ => prop_assert!(false, "{original:?} vs {shuffled:?}"),
        }
    }

    #[test]
    fn a_reported_edge_lies_in_its_component((program, _) in permuted(head_term())) {
        if let Err(violation) = program.check_weak_acyclicity() {
            let edge = violation.edge();
            prop_assert!(violation.component().contains(&edge.from));
            prop_assert!(violation.component().contains(&edge.to));
            prop_assert_eq!(edge.from.rule, edge.to.rule);
        }
    }
}
