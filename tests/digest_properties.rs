//! Property tests for `ProgramDigest`: equality, order significance, single
//! term sensitivity and stability across JSON and Postcard round trips.
use datalog_structural_model::{
    Atom, ConstantSymbol, ExistentialVariables, HeadConjunction, HeadDisjunction, Literal,
    PredicateSymbol, Program, ProgramDigest, Rule, RuleHead, Term, VariableSymbol,
};
use proptest::prelude::*;

const NAMES: [&str; 4] = ["a", "b", "c", "d"];
const PREDICATES: [&str; 3] = ["p0", "p1", "p2"];

fn leaf() -> impl Strategy<Value = Term> {
    prop_oneof![
        prop::sample::select(&NAMES[..]).prop_map(|n| Term::Variable(VariableSymbol::new(n))),
        prop::sample::select(&NAMES[..]).prop_map(|n| Term::Constant(ConstantSymbol::new(n))),
    ]
}

fn term() -> impl Strategy<Value = Term> {
    leaf().prop_recursive(2, 8, 2, |inner| {
        (prop::sample::select(&PREDICATES[..]), inner.clone(), inner).prop_map(|(p, s, o)| {
            Term::TripleTerm(Box::new(Atom {
                predicate: PredicateSymbol::new(p),
                terms: vec![s, o],
            }))
        })
    })
}

fn atom() -> impl Strategy<Value = Atom> {
    (
        prop::sample::select(&PREDICATES[..]),
        prop::collection::vec(term(), 1..4),
    )
        .prop_map(|(p, terms)| Atom {
            predicate: PredicateSymbol::new(p),
            terms,
        })
}

fn conjunction(atoms: impl Strategy<Value = Atom>) -> impl Strategy<Value = HeadConjunction> {
    prop::collection::vec(atoms, 1..3).prop_map(|atoms| HeadConjunction::try_new(atoms).unwrap())
}

/// Every head form over atoms drawn from `atoms`. Existential heads quantify
/// `w`, which no generated atom mentions, and append it to the first atom.
fn head_over(atoms: BoxedStrategy<Atom>) -> impl Strategy<Value = RuleHead> {
    prop_oneof![
        4 => atoms.clone().prop_map(RuleHead::Atom),
        1 => conjunction(atoms.clone()).prop_map(RuleHead::Conjunction),
        1 => conjunction(atoms.clone()).prop_map(|conjunction| {
            let mut atoms: Vec<Atom> = conjunction.into();
            atoms[0].terms.push(Term::Variable(VariableSymbol::new("w")));
            RuleHead::Existential {
                variables: ExistentialVariables::try_new([VariableSymbol::new("w")]).unwrap(),
                conjunction: HeadConjunction::try_new(atoms).unwrap(),
            }
        }),
        1 => prop::collection::vec(conjunction(atoms), 1..3)
            .prop_map(|disjuncts| RuleHead::Disjunction(HeadDisjunction::try_new(disjuncts).unwrap())),
        1 => Just(RuleHead::Falsum),
    ]
}

fn literal() -> impl Strategy<Value = Literal> {
    prop_oneof![
        3 => atom().prop_map(Literal::Positive),
        1 => atom().prop_map(Literal::Negative),
    ]
}

fn rule() -> impl Strategy<Value = Rule> {
    (
        head_over(atom().boxed()),
        prop::collection::vec(literal(), 0..4),
    )
        .prop_map(|(head, body)| Rule::new(head, body))
}

fn program() -> impl Strategy<Value = Program> {
    prop::collection::vec(rule(), 1..5).prop_map(|rules| Program { rules })
}

/// Safe positive rules: every head argument is a positive-body argument.
fn safe_rule() -> impl Strategy<Value = Rule> {
    prop::collection::vec(atom(), 1..4).prop_flat_map(|body| {
        let bound: Vec<Term> = body.iter().flat_map(|a| a.terms.iter().cloned()).collect();
        let head_atom = (
            prop::sample::select(&PREDICATES[..]),
            prop::collection::vec(prop::sample::select(bound), 1..4),
        )
            .prop_map(|(p, terms)| Atom {
                predicate: PredicateSymbol::new(p),
                terms,
            });
        head_over(head_atom.boxed()).prop_map(move |head| Rule::positive(head, body.clone()))
    })
}

fn safe_program() -> impl Strategy<Value = Program> {
    prop::collection::vec(safe_rule(), 0..5).prop_map(|rules| Program { rules })
}

/// Every top-level argument slot of every atom, in authored order.
fn argument_slots(program: &mut Program) -> Vec<&mut Term> {
    program
        .rules
        .iter_mut()
        .flat_map(|rule| {
            let body = rule.body.iter_mut().filter_map(|literal| match literal {
                Literal::Positive(atom) | Literal::Negative(atom) => Some(atom),
                _ => None,
            });
            rule.head.atoms_mut().chain(body)
        })
        .flat_map(|atom| atom.terms.iter_mut())
        .collect()
}

/// A term guaranteed to differ from `term`.
fn replacement(term: &Term) -> Term {
    match term {
        Term::Variable(var) => Term::Constant(ConstantSymbol::new(var.as_str())),
        _ => Term::Variable(VariableSymbol::new("mutated")),
    }
}

fn digest(program: &Program) -> ProgramDigest {
    program.digest()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn equal_programs_have_equal_digests(program in program()) {
        let copy = program.clone();
        prop_assert_eq!(digest(&program), digest(&copy));
        let text = digest(&program).to_string();
        prop_assert_eq!(text.parse::<ProgramDigest>().unwrap(), digest(&program));
    }

    #[test]
    fn rule_order_is_significant(program in program(), i in any::<usize>(), j in any::<usize>()) {
        let n = program.rules.len();
        let (i, j) = (i % n, j % n);
        prop_assume!(program.rules[i] != program.rules[j]);
        let mut swapped = program.clone();
        swapped.rules.swap(i, j);
        prop_assert_ne!(digest(&program), digest(&swapped));
    }

    #[test]
    fn body_literal_order_is_significant(
        program in program(), r in any::<usize>(), i in any::<usize>(), j in any::<usize>(),
    ) {
        let r = r % program.rules.len();
        let n = program.rules[r].body.len();
        prop_assume!(n >= 2);
        let i = i % n;
        let j = (i + 1 + j % (n - 1)) % n;
        prop_assume!(program.rules[r].body[i] != program.rules[r].body[j]);
        let mut swapped = program.clone();
        swapped.rules[r].body.swap(i, j);
        prop_assert_ne!(digest(&program), digest(&swapped));
    }

    #[test]
    fn argument_order_is_significant(
        program in program(), r in any::<usize>(), i in any::<usize>(), j in any::<usize>(),
    ) {
        let r = r % program.rules.len();
        let mut swapped = program.clone();
        let head = swapped.rules[r].head.atoms_mut().next();
        prop_assume!(head.is_some());
        let terms = &mut head.unwrap().terms;
        let n = terms.len();
        let (i, j) = (i % n, j % n);
        prop_assume!(terms[i] != terms[j]);
        terms.swap(i, j);
        prop_assert_ne!(digest(&program), digest(&swapped));
    }

    #[test]
    fn changing_any_single_term_changes_the_digest(program in program(), slot in any::<usize>()) {
        let original = digest(&program);
        let mut changed = program.clone();
        let mut slots = argument_slots(&mut changed);
        prop_assume!(!slots.is_empty());
        let index = slot % slots.len();
        let term = &mut *slots[index];
        *term = replacement(term);
        prop_assert_ne!(original, digest(&changed));
    }

    #[test]
    fn json_and_postcard_round_trips_keep_the_digest(program in safe_program()) {
        prop_assume!(program.validate().is_ok());
        let json = serde_json::to_string(&program).unwrap();
        let bytes = postcard::to_allocvec(&program).unwrap();
        let from_json: Program = serde_json::from_str(&json).unwrap();
        let from_postcard: Program = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(digest(&from_json), digest(&program));
        prop_assert_eq!(digest(&from_postcard), digest(&program));
    }
}
