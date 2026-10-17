//! Finite sets of explicitly supported Datalog language features.
use crate::{Aggregator, BindingKind, FilterKind};
use std::collections::BTreeSet;

/// A language feature required by a rule program.
/// Positive flat atoms require no extension capabilities.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum DatalogFeature {
    /// Stratified negation-as-failure.
    StratifiedNegation,
    /// Structured triple terms in rule arguments.
    TripleTerms,
    /// Deterministic head-only Skolem generation.
    SkolemGeneration,
    /// An exact built-in filter operation.
    Filter(FilterKind),
    /// An exact built-in binding operation.
    Binding(BindingKind),
    /// An exact aggregation operation.
    Aggregate(Aggregator),
}

/// Language feature set. This describes a contract; it does not certify an engine.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct DatalogCapabilities(BTreeSet<DatalogFeature>);
impl DatalogCapabilities {
    /// Pure positive Datalog over flat terms, with no extension operations.
    #[must_use]
    pub fn positive() -> Self {
        Self::default()
    }
    /// Declare one additional supported feature.
    #[must_use]
    pub fn with(mut self, feature: DatalogFeature) -> Self {
        self.0.insert(feature);
        self
    }
    /// Inspect one exact feature.
    #[must_use]
    pub fn contains(&self, feature: &DatalogFeature) -> bool {
        self.0.contains(feature)
    }
    /// Inspect features in stable structural order.
    pub fn iter(&self) -> impl Iterator<Item = &DatalogFeature> {
        self.0.iter()
    }
    /// All features representable by this model, without claiming engine support.
    #[must_use]
    pub fn all_model_features() -> Self {
        let mut result = Self::positive()
            .with(DatalogFeature::StratifiedNegation)
            .with(DatalogFeature::TripleTerms)
            .with(DatalogFeature::SkolemGeneration);
        for kind in FilterKind::ALL {
            result.0.insert(DatalogFeature::Filter(*kind));
        }
        for kind in BindingKind::ALL {
            result.0.insert(DatalogFeature::Binding(*kind));
        }
        for kind in [
            Aggregator::Count,
            Aggregator::Sum,
            Aggregator::Min,
            Aggregator::Max,
            Aggregator::Avg,
        ] {
            result.0.insert(DatalogFeature::Aggregate(kind));
        }
        result
    }
    pub(crate) fn insert(&mut self, feature: DatalogFeature) {
        self.0.insert(feature);
    }
    pub(crate) fn missing_from(&self, supported: &Self) -> Self {
        Self(self.0.difference(&supported.0).copied().collect())
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
