use serde::{Deserialize, Serialize};

/// A replication manifest (§32): everything needed to reproduce a
/// replication attempt bit-for-bit. If a field required for reproduction
/// isn't here, reproducibility is a lie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationManifest {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub target: String,
    pub target_version: Option<String>,
    pub reference_hash: Option<String>,
    pub source_location: String,
    pub environment: EnvironmentFingerprint,
    pub dependencies: Vec<String>,
    pub tool_versions: std::collections::BTreeMap<String, String>,
    pub equivalence_dimensions: Vec<String>,
    pub random_seed: u64,
}

impl ReplicationManifest {
    pub fn new(target: impl Into<String>, source_location: impl Into<String>) -> Self {
        Self {
            id: crate::new_id(),
            created_at: crate::now(),
            target: target.into(),
            target_version: None,
            reference_hash: None,
            source_location: source_location.into(),
            environment: EnvironmentFingerprint::current(),
            dependencies: Vec::new(),
            tool_versions: std::collections::BTreeMap::new(),
            equivalence_dimensions: Vec::new(),
            random_seed: 0,
        }
    }
}

/// Environmental observation (§5) — recorded so environmental accidents
/// are never mistaken for reference behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentFingerprint {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub num_cpus: usize,
}

impl EnvironmentFingerprint {
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
            num_cpus: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manifest_captures_the_current_environment() {
        let m = ReplicationManifest::new("target", "/some/path");
        assert_eq!(m.target, "target");
        assert_eq!(m.source_location, "/some/path");
        assert_eq!(m.environment.os, std::env::consts::OS);
        assert!(m.environment.num_cpus >= 1);
        assert!(m.dependencies.is_empty());
        assert!(m.equivalence_dimensions.is_empty());
    }

    #[test]
    fn manifest_round_trips_through_json() {
        let m = ReplicationManifest::new("t", "src");
        let json = serde_json::to_string(&m).unwrap();
        let back: ReplicationManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, m.id);
        assert_eq!(back.environment.arch, m.environment.arch);
    }
}
