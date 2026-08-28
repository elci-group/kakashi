//! Domain model shared by every Kakashi crate: fingerprints, the evidence
//! envelope, equivalence dimensions/scores, differences, and the
//! replication manifest. Nothing here executes anything — this crate only
//! defines the shapes that the rest of the system produces and consumes.

pub mod dimension;
pub mod disposition;
pub mod envelope;
pub mod fingerprint;
pub mod intervention;
pub mod manifest;
pub mod state;

pub use dimension::{EquivalenceDimension, EquivalenceScores};
pub use disposition::Disposition;
pub use envelope::{Difference, EvidenceEnvelope, Severity};
pub use fingerprint::Fingerprint;
pub use intervention::InterventionReason;
pub use manifest::ReplicationManifest;
pub use state::ReplicationState;

/// RFC 3339 timestamp, generated at the point evidence is recorded rather
/// than reconstructed later — recording time is itself evidence.
pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

/// New random identifier for experiments, replications, and evidence rows.
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
