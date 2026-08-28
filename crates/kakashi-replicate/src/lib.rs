//! The generation boundary (§37, §46). Kakashi does not generate replica
//! source code itself — that step is explicitly an LLM- or human-authored
//! proposal in the governing architecture (`LLM proposes → Kakashi
//! records → deterministic tests → evidence → Kakashi decides`). What this
//! crate *does* produce is the [`ReplicationBrief`]: the complete,
//! evidence-backed bundle that proposal step needs, plus the honest
//! [`InterventionReason`] state when generation can't proceed
//! automatically.

use kakashi_core::{Fingerprint, InterventionReason};
use kakashi_observe::StaticDiscovery;
use kakashi_probe::{CliSurface, ProbeResult};
use serde::{Deserialize, Serialize};

/// Everything Kakashi has established about a reference system, packaged
/// for whoever (human or LLM) implements the replica next. This is the
/// artifact `kakashi replicate` produces — it is not itself a replica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationBrief {
    pub target: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub discovery: StaticDiscovery,
    pub fingerprint: Fingerprint,
    pub cli_surface: Option<CliSurface>,
    pub boundary_probes: Vec<ProbeResult>,
    /// Why this brief stops short of a replica: automatic generation is
    /// out of scope for the deterministic layer by design (§37).
    pub intervention: InterventionReason,
    pub notes: Vec<String>,
}

pub fn build_brief(
    target: impl Into<String>,
    discovery: StaticDiscovery,
    fingerprint: Fingerprint,
    cli_surface: Option<CliSurface>,
    boundary_probes: Vec<ProbeResult>,
) -> ReplicationBrief {
    let mut notes = vec![
        "Kakashi does not generate replica source code (§37): this brief is the evidence bundle for the next proposal step.".to_string(),
    ];
    if cli_surface.is_none() {
        notes.push("no CLI surface was extracted — run `kakashi probe` against a runnable entry point first".to_string());
    }
    if boundary_probes.is_empty() {
        notes.push("no boundary probes recorded — behavioural equivalence cannot be assessed until some exist".to_string());
    }

    ReplicationBrief {
        target: target.into(),
        generated_at: kakashi_core::now(),
        discovery,
        fingerprint,
        cli_surface,
        boundary_probes,
        intervention: InterventionReason::InterventionRequired,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kakashi_observe::StaticDiscovery;

    fn empty_discovery() -> StaticDiscovery {
        StaticDiscovery {
            root: "r".into(),
            file_count: 0,
            total_bytes: 0,
            languages_by_extension: Default::default(),
            build_systems: Vec::new(),
            entry_points: Vec::new(),
        }
    }

    #[test]
    fn brief_always_stops_at_intervention_required() {
        // (§37) — Kakashi never proposes a replica itself, regardless of
        // how much evidence the brief carries.
        let brief = build_brief(
            "t",
            empty_discovery(),
            Fingerprint::new("t"),
            Some(CliSurface::default()),
            vec![],
        );
        assert_eq!(brief.intervention, InterventionReason::InterventionRequired);
    }

    #[test]
    fn brief_notes_flag_missing_cli_surface_and_missing_probes() {
        let brief = build_brief("t", empty_discovery(), Fingerprint::new("t"), None, vec![]);
        assert!(brief.notes.iter().any(|n| n.contains("no CLI surface")));
        assert!(brief.notes.iter().any(|n| n.contains("no boundary probes")));
    }

    #[test]
    fn brief_omits_the_missing_surface_note_once_a_surface_is_present() {
        let brief = build_brief(
            "t",
            empty_discovery(),
            Fingerprint::new("t"),
            Some(CliSurface::default()),
            vec![],
        );
        assert!(!brief.notes.iter().any(|n| n.contains("no CLI surface")));
    }
}
