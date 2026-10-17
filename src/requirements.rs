//! Exhaustive nested feature discovery, independent of evaluation.
use crate::ir::walk::Visitor;
use crate::{BindExpr, BuiltinFilter, DatalogCapabilities, DatalogFeature, Literal, Program, Term};
#[derive(Default)]
struct Requirements(DatalogCapabilities);
impl Visitor for Requirements {
    fn term(&mut self, term: &Term) {
        match term {
            Term::Variable(_) | Term::Constant(_) => {}
            Term::TripleTerm(_) => self.0.insert(DatalogFeature::TripleTerms),
            Term::Skolem { .. } => self.0.insert(DatalogFeature::SkolemGeneration),
        }
    }
    fn binding(&mut self, expr: &BindExpr) {
        self.0.insert(DatalogFeature::Binding(expr.kind()));
    }
    fn filter(&mut self, expr: &BuiltinFilter) {
        self.0.insert(DatalogFeature::Filter(expr.kind()));
    }
    fn literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Negative(_) => self.0.insert(DatalogFeature::StratifiedNegation),
            Literal::Aggregate(aggregate) => self
                .0
                .insert(DatalogFeature::Aggregate(aggregate.aggregator)),
            Literal::Positive(_) | Literal::Filter(_) | Literal::Bind { .. } => {}
        }
    }
}
impl Program {
    /// Discover every required language feature, including inside aggregate scopes.
    /// This inspects authored structure and does not validate or execute it.
    #[must_use]
    pub fn required_capabilities(&self) -> DatalogCapabilities {
        let mut requirements = Requirements::default();
        for rule in &self.rules {
            rule.walk(&mut requirements);
        }
        requirements.0
    }
}
