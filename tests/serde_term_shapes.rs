//! Standalone term decoding cannot bypass structural limits.
use datalog_structural_model::{
    Atom, ConstantSymbol, GroundAtom, GroundTerm, MAX_TRIPLE_TERM_NESTING, PredicateSymbol, Term,
};
#[test]
fn standalone_term_depth_is_checked_in_both_formats() {
    let mut value = Term::Constant(ConstantSymbol::new("value"));
    for _ in 0..=MAX_TRIPLE_TERM_NESTING {
        value = Term::TripleTerm(Box::new(Atom {
            predicate: PredicateSymbol::new("p"),
            terms: vec![value],
        }));
    }
    assert!(serde_json::from_str::<Term>(&serde_json::to_string(&value).unwrap()).is_err());
    assert!(postcard::from_bytes::<Term>(&postcard::to_allocvec(&value).unwrap()).is_err());
}
#[test]
fn standalone_ground_term_depth_is_checked_in_both_formats() {
    let mut value = GroundTerm::Constant(ConstantSymbol::new("value"));
    for _ in 0..=MAX_TRIPLE_TERM_NESTING {
        value = GroundTerm::TripleTerm(Box::new(GroundAtom {
            predicate: PredicateSymbol::new("p"),
            args: vec![value],
        }));
    }
    assert!(serde_json::from_str::<GroundTerm>(&serde_json::to_string(&value).unwrap()).is_err());
    assert!(postcard::from_bytes::<GroundTerm>(&postcard::to_allocvec(&value).unwrap()).is_err());
}
#[test]
fn standalone_skolem_arguments_are_checked_in_both_formats() {
    let inner = Term::Skolem {
        name: ConstantSymbol::new("inner"),
        args: vec![],
    };
    let value = Term::Skolem {
        name: ConstantSymbol::new("outer"),
        args: vec![inner],
    };
    assert!(serde_json::from_str::<Term>(&serde_json::to_string(&value).unwrap()).is_err());
    assert!(postcard::from_bytes::<Term>(&postcard::to_allocvec(&value).unwrap()).is_err());
}

#[test]
fn supported_depths_and_flat_skolem_arguments_round_trip() {
    let mut term = Term::Constant(ConstantSymbol::new("value"));
    let mut ground = GroundTerm::Constant(ConstantSymbol::new("value"));
    for depth in 0..=MAX_TRIPLE_TERM_NESTING {
        assert_eq!(
            serde_json::from_str::<Term>(&serde_json::to_string(&term).unwrap()).unwrap(),
            term
        );
        assert_eq!(
            postcard::from_bytes::<Term>(&postcard::to_allocvec(&term).unwrap()).unwrap(),
            term
        );
        assert_eq!(
            serde_json::from_str::<GroundTerm>(&serde_json::to_string(&ground).unwrap()).unwrap(),
            ground
        );
        assert_eq!(
            postcard::from_bytes::<GroundTerm>(&postcard::to_allocvec(&ground).unwrap()).unwrap(),
            ground
        );
        if depth < MAX_TRIPLE_TERM_NESTING {
            term = Term::try_triple_term(Atom {
                predicate: PredicateSymbol::new("p"),
                terms: vec![term],
            })
            .unwrap();
            ground = GroundTerm::try_triple_term(GroundAtom {
                predicate: PredicateSymbol::new("p"),
                args: vec![ground],
            })
            .unwrap();
        }
    }
    assert!(
        Term::try_triple_term(Atom {
            predicate: PredicateSymbol::new("p"),
            terms: vec![term]
        })
        .is_err()
    );
    assert!(
        GroundTerm::try_triple_term(GroundAtom {
            predicate: PredicateSymbol::new("p"),
            args: vec![ground]
        })
        .is_err()
    );
    let skolem = Term::try_skolem(
        ConstantSymbol::new("label"),
        vec![Term::Constant(ConstantSymbol::new("argument"))],
    )
    .unwrap();
    assert_eq!(
        postcard::from_bytes::<Term>(&postcard::to_allocvec(&skolem).unwrap()).unwrap(),
        skolem
    );
    assert_eq!(
        serde_json::from_str::<Term>(&serde_json::to_string(&skolem).unwrap()).unwrap(),
        skolem
    );
}
