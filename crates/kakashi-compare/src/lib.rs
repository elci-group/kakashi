//! The differential comparator (§25–§26). Every comparison here produces
//! structured [`kakashi_core::Difference`] records, never a bare
//! pass/fail, and every score is computed only from differences that were
//! actually measured.

use kakashi_core::{Difference, EquivalenceDimension, EquivalenceScores, Severity};
use kakashi_probe::ProbeResult;
use std::collections::BTreeSet;
use std::path::Path;
use walkdir::WalkDir;

/// Compare the file sets of two trees (§25 structural dimension).
/// Entry points and build-system markers missing on one side are treated
/// as higher severity than an arbitrary extra file.
pub fn diff_structure(reference_root: &Path, replica_root: &Path) -> Vec<Difference> {
    let ref_files = relative_files(reference_root);
    let rep_files = relative_files(replica_root);

    let mut diffs = Vec::new();
    for missing in ref_files.difference(&rep_files) {
        diffs.push(Difference::new(
            "structural",
            format!("file present in reference but missing in replica: {missing}"),
            serde_json::json!(missing),
            serde_json::Value::Null,
            severity_for_path(missing),
        ));
    }
    for extra in rep_files.difference(&ref_files) {
        diffs.push(Difference::new(
            "structural",
            format!("file present in replica but absent from reference: {extra}"),
            serde_json::Value::Null,
            serde_json::json!(extra),
            Severity::Low,
        ));
    }
    diffs
}

fn severity_for_path(path: &str) -> Severity {
    let name = path.rsplit('/').next().unwrap_or(path);
    const CRITICAL_MARKERS: &[&str] =
        &["Cargo.toml", "package.json", "go.mod", "main.rs", "main.go"];
    if CRITICAL_MARKERS.contains(&name) {
        Severity::High
    } else {
        Severity::Low
    }
}

fn relative_files(root: &Path) -> BTreeSet<String> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !kakashi_observe::is_ignored(e.path()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(root)
                .unwrap_or(e.path())
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// Compare matched probe results between reference and replica (§25
/// behavioural dimension). Probes are matched by label; a probe present on
/// only one side is itself a difference.
pub fn diff_probe_results(reference: &[ProbeResult], replica: &[ProbeResult]) -> Vec<Difference> {
    let mut diffs = Vec::new();

    for r in reference {
        let Some(p) = replica.iter().find(|p| p.probe.label == r.probe.label) else {
            diffs.push(Difference::new(
                "behavioural",
                format!(
                    "probe '{}' has no matching replica observation",
                    r.probe.label
                ),
                serde_json::json!(r.report.exit_code),
                serde_json::Value::Null,
                Severity::High,
            ));
            continue;
        };

        if r.report.exit_code != p.report.exit_code {
            diffs.push(Difference::new(
                "behavioural",
                format!("exit code mismatch on probe '{}'", r.probe.label),
                serde_json::json!(r.report.exit_code),
                serde_json::json!(p.report.exit_code),
                Severity::Critical,
            ));
        }
        if r.report.stdout.trim() != p.report.stdout.trim() {
            diffs.push(Difference::new(
                "behavioural",
                format!("stdout mismatch on probe '{}'", r.probe.label),
                serde_json::json!(r.report.stdout),
                serde_json::json!(p.report.stdout),
                Severity::High,
            ));
        }
    }

    diffs
}

/// Roll a set of differences up into per-dimension scores. Dimensions with
/// no observed differences AND no evidence they were measured stay `None`
/// — the caller is responsible for only calling this once a dimension has
/// actually been exercised.
pub fn score_dimension(
    dimension: EquivalenceDimension,
    differences: &[Difference],
    total_observations: usize,
) -> Option<f64> {
    if total_observations == 0 {
        return None;
    }
    let dim_key = match dimension {
        EquivalenceDimension::Behavioural => "behavioural",
        EquivalenceDimension::Interface => "interface",
        EquivalenceDimension::Configuration => "configuration",
        EquivalenceDimension::Schema => "schema",
        EquivalenceDimension::Structural => "structural",
        EquivalenceDimension::Operational => "operational",
        EquivalenceDimension::Performance => "performance",
        EquivalenceDimension::Binary => "binary",
    };
    let penalty: f64 = differences
        .iter()
        .filter(|d| d.dimension == dim_key)
        .map(|d| match d.severity {
            Severity::Critical => 1.0,
            Severity::High => 0.5,
            Severity::Low => 0.1,
            Severity::Informational => 0.0,
        })
        .sum();
    Some((1.0 - penalty / total_observations as f64).clamp(0.0, 1.0))
}

pub fn build_scores(
    structural_diffs: &[Difference],
    behavioural_diffs: &[Difference],
    structural_total: usize,
    behavioural_total: usize,
) -> EquivalenceScores {
    let mut scores = EquivalenceScores::default();
    if let Some(s) = score_dimension(
        EquivalenceDimension::Structural,
        structural_diffs,
        structural_total,
    ) {
        scores.set(EquivalenceDimension::Structural, s);
    }
    if let Some(s) = score_dimension(
        EquivalenceDimension::Behavioural,
        behavioural_diffs,
        behavioural_total,
    ) {
        scores.set(EquivalenceDimension::Behavioural, s);
    }
    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn identical_trees_have_no_structural_differences() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        fs::write(a.path().join("main.rs"), b"fn main() {}").unwrap();
        fs::write(b.path().join("main.rs"), b"fn main() {}").unwrap();

        let diffs = diff_structure(a.path(), b.path());
        assert!(diffs.is_empty());
    }

    #[test]
    fn missing_replica_file_is_reported() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        fs::write(a.path().join("Cargo.toml"), b"[package]").unwrap();

        let diffs = diff_structure(a.path(), b.path());
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].dimension, "structural");
        assert_eq!(diffs[0].severity, Severity::High);
    }

    #[test]
    fn score_dimension_returns_none_when_unmeasured() {
        assert_eq!(
            score_dimension(EquivalenceDimension::Structural, &[], 0),
            None
        );
    }

    #[test]
    fn score_dimension_penalises_by_severity() {
        let diffs = vec![Difference::new(
            "structural",
            "x",
            serde_json::Value::Null,
            serde_json::Value::Null,
            Severity::Critical,
        )];
        let score = score_dimension(EquivalenceDimension::Structural, &diffs, 1).unwrap();
        assert_eq!(score, 0.0);
    }
}
