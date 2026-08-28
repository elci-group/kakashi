use serde::{Deserialize, Serialize};

/// Where a replication currently sits in the convergence loop (§3, §38).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationState {
    Reference,
    Observation,
    Characterisation,
    SystemModel,
    Replica,
    Validation,
    DifferentialAnalysis,
    Repair,
    Revalidation,
    HumanIntervention,
    Converged,
    Blocked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&ReplicationState::DifferentialAnalysis).unwrap(),
            "\"differential_analysis\""
        );
    }

    #[test]
    fn human_intervention_is_a_distinct_state_not_an_error() {
        // (§46) — human intervention is first-class, not folded into
        // `Blocked` or represented only as an `Err`.
        assert_ne!(
            ReplicationState::HumanIntervention,
            ReplicationState::Blocked
        );
    }
}
