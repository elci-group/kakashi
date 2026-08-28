use serde::{Deserialize, Serialize};

/// Human intervention is a first-class state, not an error path (§46).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InterventionReason {
    AmbiguousReference,
    UnsafeExecution,
    ConflictingEvidence,
    LowConfidence,
    NondeterministicBehaviour,
    IrreducibleDifference,
    InterventionRequired,
}

impl std::fmt::Display for InterventionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Must track the `SCREAMING_SNAKE_CASE` serde rename above — human
        // output and JSON output must agree on the same string for the
        // same value, never two different spellings of the same fact.
        let s = match self {
            Self::AmbiguousReference => "AMBIGUOUS_REFERENCE",
            Self::UnsafeExecution => "UNSAFE_EXECUTION",
            Self::ConflictingEvidence => "CONFLICTING_EVIDENCE",
            Self::LowConfidence => "LOW_CONFIDENCE",
            Self::NondeterministicBehaviour => "NONDETERMINISTIC_BEHAVIOUR",
            Self::IrreducibleDifference => "IRREDUCIBLE_DIFFERENCE",
            Self::InterventionRequired => "INTERVENTION_REQUIRED",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bug this guards against: `Display` derived from `{:?}` silently
    /// diverges from the `serde(rename_all = "SCREAMING_SNAKE_CASE")`
    /// wire format, so human output and JSON output disagree about the
    /// same value.
    #[test]
    fn display_matches_serde_wire_format() {
        for reason in [
            InterventionReason::AmbiguousReference,
            InterventionReason::UnsafeExecution,
            InterventionReason::ConflictingEvidence,
            InterventionReason::LowConfidence,
            InterventionReason::NondeterministicBehaviour,
            InterventionReason::IrreducibleDifference,
            InterventionReason::InterventionRequired,
        ] {
            let json = serde_json::to_string(&reason).unwrap();
            let wire_value: String = serde_json::from_str(&json).unwrap();
            assert_eq!(reason.to_string(), wire_value);
        }
    }
}
