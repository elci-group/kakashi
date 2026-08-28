use serde::{Deserialize, Serialize};

/// Domain-specific hashes for one characterised system (§22).
///
/// Deliberately not a single monolithic hash: a change to configuration
/// should not be indistinguishable from a change to source, so each
/// dimension gets its own digest and `None` means "not computed", not
/// "empty".
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Fingerprint {
    pub target: String,
    pub computed_at: chrono::DateTime<chrono::Utc>,
    pub source_sha: Option<String>,
    pub dependency_sha: Option<String>,
    pub interface_sha: Option<String>,
    pub config_sha: Option<String>,
    pub behaviour_sha: Option<String>,
    pub binary_sha: Option<String>,
    pub environment_sha: Option<String>,
}

impl Fingerprint {
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            computed_at: crate::now(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fingerprint_leaves_every_hash_unmeasured() {
        // `Default` gives every `Option<String>` field `None`, not an
        // empty string — a fingerprint dimension nobody has computed yet
        // must render as "not computed", never as "empty" (§22).
        let fp = Fingerprint::new("t");
        assert_eq!(fp.target, "t");
        assert!(fp.source_sha.is_none());
        assert!(fp.dependency_sha.is_none());
        assert!(fp.interface_sha.is_none());
        assert!(fp.config_sha.is_none());
        assert!(fp.behaviour_sha.is_none());
        assert!(fp.binary_sha.is_none());
        assert!(fp.environment_sha.is_none());
    }
}
