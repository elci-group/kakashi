use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Severity of an observation or a detected difference (§10, §25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Informational,
    Low,
    High,
    Critical,
}

/// The common ELCI evidence envelope (§21), with Kakashi's extensions.
///
/// Every subsystem in this workspace emits this shape into
/// `.kakashi/evidence/*.jsonl` and `.kakashi/audit.jsonl`. Nothing invents
/// a competing format — that's the whole point of a shared substrate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEnvelope {
    pub id: String,
    pub subject: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub evidence: Value,
    pub confidence: f64,
    pub severity: Severity,
    #[serde(default)]
    pub measurements: Value,
    #[serde(default)]
    pub relationships: Vec<String>,
    #[serde(default)]
    pub proposed_actions: Vec<String>,
    pub provenance: String,

    // Kakashi-specific extensions.
    #[serde(default)]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub replica_id: Option<String>,
    #[serde(default)]
    pub experiment_id: Option<String>,
    #[serde(default)]
    pub equivalence_dimension: Option<String>,
    #[serde(default)]
    pub observation_method: Option<String>,
    #[serde(default)]
    pub comparison_method: Option<String>,
    #[serde(default)]
    pub environment_fingerprint: Option<String>,
    #[serde(default)]
    pub input_fingerprint: Option<String>,
    #[serde(default)]
    pub output_fingerprint: Option<String>,
}

impl EvidenceEnvelope {
    pub fn new(
        subject: impl Into<String>,
        source: impl Into<String>,
        provenance: impl Into<String>,
    ) -> Self {
        Self {
            id: crate::new_id(),
            subject: subject.into(),
            source: source.into(),
            timestamp: crate::now(),
            evidence: Value::Null,
            confidence: 1.0,
            severity: Severity::Informational,
            measurements: Value::Null,
            relationships: Vec::new(),
            proposed_actions: Vec::new(),
            provenance: provenance.into(),
            reference_id: None,
            replica_id: None,
            experiment_id: None,
            equivalence_dimension: None,
            observation_method: None,
            comparison_method: None,
            environment_fingerprint: None,
            input_fingerprint: None,
            output_fingerprint: None,
        }
    }

    pub fn with_evidence(mut self, evidence: Value) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

/// A single structured difference between reference and replica
/// observations (§25). Never a bare pass/fail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    pub id: String,
    pub dimension: String,
    pub description: String,
    pub reference_value: Value,
    pub replica_value: Value,
    pub severity: Severity,
    pub confidence: f64,
}

impl Difference {
    pub fn new(
        dimension: impl Into<String>,
        description: impl Into<String>,
        reference_value: Value,
        replica_value: Value,
        severity: Severity,
    ) -> Self {
        Self {
            id: crate::new_id(),
            dimension: dimension.into(),
            description: description.into(),
            reference_value,
            replica_value,
            severity,
            confidence: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering_places_critical_highest() {
        // score_dimension (kakashi-compare) and the report renderer both
        // rely on this ordering to sort/compare severities.
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Low);
        assert!(Severity::Low > Severity::Informational);
    }

    #[test]
    fn severity_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&Severity::Critical).unwrap(),
            "\"critical\""
        );
    }

    #[test]
    fn new_envelope_has_no_extensions_set_and_full_confidence() {
        let e = EvidenceEnvelope::new("subject", "source", "prov");
        assert_eq!(e.confidence, 1.0);
        assert_eq!(e.severity, Severity::Informational);
        assert!(e.reference_id.is_none());
        assert!(e.experiment_id.is_none());
        assert!(e.relationships.is_empty());
    }

    #[test]
    fn builder_methods_only_change_the_field_they_target() {
        let e = EvidenceEnvelope::new("s", "src", "prov")
            .with_severity(Severity::Critical)
            .with_confidence(0.5)
            .with_evidence(serde_json::json!({"k": "v"}));
        assert_eq!(e.severity, Severity::Critical);
        assert_eq!(e.confidence, 0.5);
        assert_eq!(e.evidence, serde_json::json!({"k": "v"}));
        assert_eq!(e.subject, "s");
    }

    #[test]
    fn envelope_round_trips_through_json_including_missing_optional_fields() {
        // Extensions use `#[serde(default)]` so evidence written by an
        // older Kakashi build (without a field this version knows about)
        // must still deserialize.
        let minimal = serde_json::json!({
            "id": "x",
            "subject": "s",
            "source": "src",
            "timestamp": "2026-01-01T00:00:00Z",
            "evidence": null,
            "confidence": 1.0,
            "severity": "informational",
            "provenance": "p"
        });
        let e: EvidenceEnvelope = serde_json::from_value(minimal).unwrap();
        assert!(e.reference_id.is_none());
        assert!(e.proposed_actions.is_empty());
    }

    #[test]
    fn difference_defaults_to_full_confidence() {
        let d = Difference::new(
            "structural",
            "desc",
            Value::Null,
            Value::Null,
            Severity::Low,
        );
        assert_eq!(d.confidence, 1.0);
        assert_eq!(d.dimension, "structural");
    }
}
