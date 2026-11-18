//! Canonical preimages of programs, rules, literals, atoms and terms.

use super::{
    domain::{CanonicalPreimage, DomainDigest},
    output::ProgramDigest,
    sink::PreimageSink,
};
use crate::{
    AggregateBody, Aggregator, Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term,
    VariableSymbol,
};

impl DomainDigest for Program {
    const DOMAIN: &'static str = "datalog-structural-model/program-digest/v1";
    type Output = ProgramDigest;
}

impl CanonicalPreimage for Program {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        Rule::write_sequence(&self.rules, sink);
    }
}

impl CanonicalPreimage for Rule {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        self.head.write_preimage(sink);
        Literal::write_sequence(&self.body, sink);
    }
}

impl CanonicalPreimage for Atom {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        self.predicate.write_preimage(sink);
        Term::write_sequence(&self.terms, sink);
    }
}

impl CanonicalPreimage for PredicateSymbol {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        sink.text(self.as_str());
    }
}

impl CanonicalPreimage for VariableSymbol {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        sink.text(self.as_str());
    }
}

impl CanonicalPreimage for ConstantSymbol {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        sink.text(self.as_str());
    }
}

impl CanonicalPreimage for Term {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        match self {
            Self::Variable(var) => {
                sink.tag(0);
                var.write_preimage(sink);
            }
            Self::Constant(constant) => {
                sink.tag(1);
                constant.write_preimage(sink);
            }
            Self::TripleTerm(atom) => {
                sink.tag(2);
                atom.write_preimage(sink);
            }
            Self::Skolem { name, args } => {
                sink.tag(3);
                name.write_preimage(sink);
                Self::write_sequence(args, sink);
            }
        }
    }
}

impl CanonicalPreimage for Literal {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        match self {
            Self::Positive(atom) => {
                sink.tag(0);
                atom.write_preimage(sink);
            }
            Self::Negative(atom) => {
                sink.tag(1);
                atom.write_preimage(sink);
            }
            Self::Filter(filter) => {
                sink.tag(2);
                filter.write_preimage(sink);
            }
            Self::Bind { var, expr } => {
                sink.tag(3);
                var.write_preimage(sink);
                expr.write_preimage(sink);
            }
            Self::Aggregate(body) => {
                sink.tag(4);
                body.write_preimage(sink);
            }
        }
    }
}

impl CanonicalPreimage for Aggregator {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        sink.tag(match self {
            Self::Count => 0,
            Self::Sum => 1,
            Self::Min => 2,
            Self::Max => 3,
            Self::Avg => 4,
        });
    }
}

impl CanonicalPreimage for AggregateBody {
    fn write_preimage(&self, sink: &mut PreimageSink) {
        self.result_var.write_preimage(sink);
        self.aggregator.write_preimage(sink);
        self.input_var.write_preimage(sink);
        VariableSymbol::write_sequence(&self.group_by, sink);
        Literal::write_sequence(&self.inner_body, sink);
    }
}

impl Program {
    /// Canonical content identity of this program as authored.
    ///
    /// Rule order, body literal order and argument order are significant.
    /// The digest is computed from a versioned, length-framed preimage and
    /// never from a Serde encoding.
    #[must_use]
    pub fn digest(&self) -> ProgramDigest {
        DomainDigest::digest(self)
    }
}
