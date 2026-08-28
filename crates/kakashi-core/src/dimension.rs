use serde::{Deserialize, Serialize};

/// The eight equivalence dimensions Kakashi distinguishes (§1, §26).
///
/// A replication job declares which of these it requires; passing on
/// dimensions it never measured is not the same as passing on all eight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquivalenceDimension {
    Behavioural,
    Interface,
    Configuration,
    Schema,
    Structural,
    Operational,
    Performance,
    Binary,
}

impl EquivalenceDimension {
    pub const ALL: [EquivalenceDimension; 8] = [
        EquivalenceDimension::Behavioural,
        EquivalenceDimension::Interface,
        EquivalenceDimension::Configuration,
        EquivalenceDimension::Schema,
        EquivalenceDimension::Structural,
        EquivalenceDimension::Operational,
        EquivalenceDimension::Performance,
        EquivalenceDimension::Binary,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Behavioural => "Behaviour",
            Self::Interface => "Interface",
            Self::Configuration => "Configuration",
            Self::Schema => "Schema",
            Self::Structural => "Structure",
            Self::Operational => "Operations",
            Self::Performance => "Performance",
            Self::Binary => "Binary",
        }
    }
}

/// A score for one dimension, or an explicit statement that it was never
/// measured. `None` must render as `N/A`, never as `0.0` or `1.0` — a
/// dimension nobody tested is not evidence of anything (§26).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EquivalenceScores {
    pub scores: std::collections::BTreeMap<String, Option<f64>>,
}

impl EquivalenceScores {
    pub fn set(&mut self, dim: EquivalenceDimension, score: f64) {
        self.scores.insert(dim.label().to_string(), Some(score));
    }

    pub fn get(&self, dim: EquivalenceDimension) -> Option<f64> {
        self.scores.get(dim.label()).copied().flatten()
    }

    /// Composite score over only the measured dimensions. Returns `None`
    /// if nothing was measured — a composite over zero evidence is not a
    /// score, it's a fabrication (§26).
    pub fn composite(&self) -> Option<f64> {
        let measured: Vec<f64> = self.scores.values().filter_map(|v| *v).collect();
        if measured.is_empty() {
            None
        } else {
            Some(measured.iter().sum::<f64>() / measured.len() as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_is_none_when_nothing_measured() {
        assert_eq!(EquivalenceScores::default().composite(), None);
    }

    #[test]
    fn composite_averages_only_measured_dimensions() {
        let mut s = EquivalenceScores::default();
        s.set(EquivalenceDimension::Behavioural, 1.0);
        s.set(EquivalenceDimension::Structural, 0.5);
        // Binary was never measured — must not silently count as 0.0,
        // which would drag the composite down to 0.5 instead of 0.75.
        assert_eq!(s.get(EquivalenceDimension::Binary), None);
        assert_eq!(s.composite(), Some(0.75));
    }

    #[test]
    fn get_returns_none_for_a_dimension_never_set() {
        let s = EquivalenceScores::default();
        assert_eq!(s.get(EquivalenceDimension::Performance), None);
    }

    #[test]
    fn all_eight_dimensions_have_distinct_labels() {
        let labels: std::collections::BTreeSet<&str> = EquivalenceDimension::ALL
            .iter()
            .map(|d| d.label())
            .collect();
        assert_eq!(labels.len(), EquivalenceDimension::ALL.len());
    }
}
