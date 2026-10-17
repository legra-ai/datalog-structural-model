//! Round-trip every modeled operator, including non-self-describing encoding.
use datalog_structural_model::*;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    assert_eq!(
        &serde_json::from_str::<T>(&serde_json::to_string(value).unwrap()).unwrap(),
        value
    );
    assert_eq!(
        &postcard::from_bytes::<T>(&postcard::to_allocvec(value).unwrap()).unwrap(),
        value
    );
}
fn term() -> Term {
    Term::Constant(ConstantSymbol::new(r#"exact literal: "quoted" @EN"#))
}
fn expr() -> Box<BindExpr> {
    Box::new(BindExpr::Term(term()))
}
fn atom() -> Atom {
    Atom {
        predicate: PredicateSymbol::new("input"),
        terms: vec![Term::Variable(VariableSymbol::new("x"))],
    }
}
fn program(literal: Literal) -> Program {
    Program::try_new(vec![
        Rule::try_new(atom(), vec![Literal::Positive(atom()), literal]).unwrap(),
    ])
    .unwrap()
}
fn binding_cases() -> [BindExpr; 42] {
    [
        BindExpr::Term(term()),
        BindExpr::Add(expr(), expr()),
        BindExpr::Sub(expr(), expr()),
        BindExpr::Mul(expr(), expr()),
        BindExpr::Div(expr(), expr()),
        BindExpr::Neg(expr()),
        BindExpr::Abs(expr()),
        BindExpr::Ceil(expr()),
        BindExpr::Floor(expr()),
        BindExpr::Round(expr()),
        BindExpr::Str(expr()),
        BindExpr::StrLen(expr()),
        BindExpr::UCase(expr()),
        BindExpr::LCase(expr()),
        BindExpr::Concat(vec![BindExpr::Term(term())]),
        BindExpr::SubStr {
            value: expr(),
            start: expr(),
            len: Some(expr()),
        },
        BindExpr::StrBefore(expr(), expr()),
        BindExpr::StrAfter(expr(), expr()),
        BindExpr::Replace {
            value: expr(),
            pattern: expr(),
            replacement: expr(),
            flags: Some(expr()),
        },
        BindExpr::EncodeForUri(expr()),
        BindExpr::Md5(expr()),
        BindExpr::Sha1(expr()),
        BindExpr::Sha256(expr()),
        BindExpr::Sha384(expr()),
        BindExpr::Sha512(expr()),
        BindExpr::CastInteger(expr()),
        BindExpr::CastDecimal(expr()),
        BindExpr::CastDouble(expr()),
        BindExpr::CastFloat(expr()),
        BindExpr::CastString(expr()),
        BindExpr::Year(expr()),
        BindExpr::Month(expr()),
        BindExpr::Day(expr()),
        BindExpr::Hours(expr()),
        BindExpr::Minutes(expr()),
        BindExpr::Seconds(expr()),
        BindExpr::Timezone(expr()),
        BindExpr::Tz(expr()),
        BindExpr::Now,
        BindExpr::Lang(expr()),
        BindExpr::Iri(expr()),
        BindExpr::BNode(expr()),
    ]
}
fn filter_cases() -> [BuiltinFilter; 18] {
    [
        BuiltinFilter::Eq(term(), term()),
        BuiltinFilter::Ne(term(), term()),
        BuiltinFilter::Lt(term(), term()),
        BuiltinFilter::Le(term(), term()),
        BuiltinFilter::Gt(term(), term()),
        BuiltinFilter::Ge(term(), term()),
        BuiltinFilter::StrStarts(term(), term()),
        BuiltinFilter::StrEnds(term(), term()),
        BuiltinFilter::Contains(term(), term()),
        BuiltinFilter::RegexMatch {
            value: term(),
            pattern: term(),
            flags: Some(term()),
        },
        BuiltinFilter::IsIri(term()),
        BuiltinFilter::IsLiteral(term()),
        BuiltinFilter::IsBlank(term()),
        BuiltinFilter::IsNumeric(term()),
        BuiltinFilter::LangMatches(term(), term()),
        BuiltinFilter::And(
            Box::new(BuiltinFilter::IsLiteral(term())),
            Box::new(BuiltinFilter::IsLiteral(term())),
        ),
        BuiltinFilter::Or(
            Box::new(BuiltinFilter::IsLiteral(term())),
            Box::new(BuiltinFilter::IsLiteral(term())),
        ),
        BuiltinFilter::Not(Box::new(BuiltinFilter::IsLiteral(term()))),
    ]
}

#[test]
fn every_binding_round_trips_and_requires_its_exact_capability() {
    let values = binding_cases();
    assert_eq!(values.len(), BindingKind::ALL.len());
    for (value, kind) in values.into_iter().zip(BindingKind::ALL) {
        assert_eq!(value.kind(), *kind);
        round_trip(&value);
        let p = program(Literal::Bind {
            var: VariableSymbol::new("out"),
            expr: value,
        });
        assert!(
            p.required_capabilities()
                .contains(&DatalogFeature::Binding(*kind))
        );
        round_trip(&p);
    }
}
#[test]
fn every_filter_round_trips_and_requires_its_exact_capability() {
    let values = filter_cases();
    assert_eq!(values.len(), FilterKind::ALL.len());
    for (value, kind) in values.into_iter().zip(FilterKind::ALL) {
        assert_eq!(value.kind(), *kind);
        round_trip(&value);
        let p = program(Literal::Filter(value));
        assert!(
            p.required_capabilities()
                .contains(&DatalogFeature::Filter(*kind))
        );
        round_trip(&p);
    }
}
#[test]
fn every_aggregate_round_trips_with_nested_requirements() {
    for aggregator in [
        Aggregator::Count,
        Aggregator::Sum,
        Aggregator::Min,
        Aggregator::Max,
        Aggregator::Avg,
    ] {
        let aggregate = AggregateBody {
            result_var: VariableSymbol::new("count"),
            aggregator,
            input_var: VariableSymbol::new("x"),
            group_by: vec![],
            inner_body: vec![
                Literal::Positive(atom()),
                Literal::Filter(BuiltinFilter::IsLiteral(Term::Variable(
                    VariableSymbol::new("x"),
                ))),
            ],
        };
        round_trip(&aggregate);
        let rule = Rule::try_new(
            Atom {
                predicate: PredicateSymbol::new("result"),
                terms: vec![Term::Variable(VariableSymbol::new("count"))],
            },
            vec![Literal::Aggregate(aggregate)],
        )
        .unwrap();
        let p = Program::try_new(vec![rule]).unwrap();
        let requirements = p.required_capabilities();
        assert!(requirements.contains(&DatalogFeature::Aggregate(aggregator)));
        assert!(requirements.contains(&DatalogFeature::Filter(FilterKind::IsLiteral)));
        p.validate_for(&DatalogProfile::new(requirements, EqualityMode::Default))
            .unwrap();
        round_trip(&p);
    }
}
#[test]
fn reports_and_all_capability_values_round_trip() {
    let p = program(Literal::Filter(BuiltinFilter::IsLiteral(term())));
    round_trip(&p.stratify().unwrap());
    let error = p.validate_for(&DatalogProfile::positive()).unwrap_err();
    round_trip(&error);
    round_trip(&DatalogProfile::new(
        DatalogCapabilities::all_model_features(),
        EqualityMode::SameAs,
    ));
    round_trip(&EqualityMode::Default);
    round_trip(&EqualityMode::SameAs);
}
