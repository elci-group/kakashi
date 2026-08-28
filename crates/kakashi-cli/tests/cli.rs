//! End-to-end tests against the compiled `kakashi` binary — the surface
//! that was previously the least tested part of the workspace despite
//! being the one every user actually touches.
//!
//! Every test spawns its own process with `current_dir` set to a fresh
//! tempdir, rather than mutating the test process's own working
//! directory, so tests stay safe to run concurrently.

use std::path::Path;
use std::process::{Command, Output};

fn kakashi() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kakashi"))
}

fn run(dir: &Path, args: &[&str]) -> Output {
    kakashi()
        .current_dir(dir)
        .args(args)
        .output()
        .expect("failed to spawn kakashi binary")
}

fn run_json(dir: &Path, args: &[&str]) -> serde_json::Value {
    let mut full_args = vec!["--format", "json"];
    full_args.extend_from_slice(args);
    let output = run(dir, &full_args);
    assert!(
        output.status.success(),
        "kakashi {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {:?}", output.stdout))
}

fn write_toy_project(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), b"[package]\nname = \"toy\"\n").unwrap();
    std::fs::write(
        dir.join("src").join("main.rs"),
        b"fn main() { println!(\"hi\"); }\n",
    )
    .unwrap();
}

#[test]
fn init_creates_state_dir_and_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("ref"));

    let output = run(tmp.path(), &["init", "toy", "--source", "ref"]);
    assert!(output.status.success());
    assert!(tmp.path().join(".kakashi").join("manifest.json").is_file());

    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(tmp.path().join(".kakashi/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["target"], "toy");
}

#[test]
fn inspect_reports_file_count_and_build_system() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("ref"));

    let discovery = run_json(tmp.path(), &["inspect", "ref"]);
    assert_eq!(discovery["file_count"], 2);
    assert_eq!(discovery["build_systems"], serde_json::json!(["cargo"]));
    assert_eq!(
        discovery["entry_points"],
        serde_json::json!(["src/main.rs"])
    );
}

#[test]
fn fingerprint_is_stable_across_repeated_invocations() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("ref"));

    let first = run_json(tmp.path(), &["fingerprint", "ref"]);
    let second = run_json(tmp.path(), &["fingerprint", "ref"]);
    assert_eq!(first["source_sha"], second["source_sha"]);
    assert_ne!(first["source_sha"], serde_json::Value::Null);
}

#[test]
fn fingerprint_changes_when_source_changes() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("ref"));
    let before = run_json(tmp.path(), &["fingerprint", "ref"]);

    std::fs::write(
        tmp.path().join("ref/src/main.rs"),
        b"fn main() { println!(\"changed\"); }\n",
    )
    .unwrap();
    let after = run_json(tmp.path(), &["fingerprint", "ref"]);

    assert_ne!(before["source_sha"], after["source_sha"]);
}

#[test]
fn status_before_init_reports_uninitialised() {
    let tmp = tempfile::tempdir().unwrap();
    let output = run(tmp.path(), &["status"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("uninitialised"), "got: {stdout}");
}

#[test]
fn diff_between_identical_trees_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("a"));
    write_toy_project(&tmp.path().join("b"));

    let diffs = run_json(tmp.path(), &["diff", "a", "b"]);
    assert_eq!(diffs, serde_json::json!([]));
}

#[test]
fn diff_reports_a_missing_critical_file_as_high_severity() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("a"));
    std::fs::create_dir_all(tmp.path().join("b")).unwrap();

    let diffs = run_json(tmp.path(), &["diff", "a", "b"]);
    let diffs = diffs.as_array().unwrap();
    assert!(!diffs.is_empty());
    let cargo_toml_diff = diffs
        .iter()
        .find(|d| d["description"].as_str().unwrap().contains("Cargo.toml"))
        .expect("Cargo.toml diff not found");
    assert_eq!(cargo_toml_diff["severity"], "high");
}

#[test]
fn validate_self_vs_self_is_equivalent_with_passing_gate() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("ref"));

    let report = run_json(tmp.path(), &["validate", "ref", "ref"]);
    assert_eq!(report["gate"], true);
    assert_eq!(report["disposition"], "EQUIVALENT");
    assert_eq!(report["scores"]["scores"]["Structure"], 1.0);
}

#[test]
fn validate_downgrades_disposition_when_structure_differs_but_gate_still_passes() {
    // Missing files are scored `High`, not `Critical` (severity_for_path
    // in kakashi-compare), and `gate` only fails on a `Critical` finding.
    // So an entirely missing replica still passes the gate — the
    // degradation shows up in the composite score and disposition
    // instead, not in the gate. This test documents that split rather
    // than assuming "structurally broken" implies "gate fails".
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("a"));
    std::fs::create_dir_all(tmp.path().join("b")).unwrap();

    let report = run_json(tmp.path(), &["validate", "a", "b"]);
    assert_eq!(report["gate"], true);
    assert_eq!(report["disposition"], "PARTIALLY_EQUIVALENT");
    let score = report["scores"]["scores"]["Structure"].as_f64().unwrap();
    assert!(score < 1.0, "expected a degraded score, got {score}");
}

#[test]
fn observe_records_a_replayable_experiment() {
    let tmp = tempfile::tempdir().unwrap();

    let experiment = run_json(
        tmp.path(),
        &["observe", "/bin/sh", "--", "-c", "echo hi; exit 3"],
    );
    assert_eq!(experiment["report"]["exit_code"], 3);
    assert_eq!(experiment["report"]["stdout"], "hi\n");
    let id = experiment["id"].as_str().unwrap();

    let replay = run_json(tmp.path(), &["replay", id]);
    assert_eq!(replay["reproducible"], true);
}

#[test]
fn probe_and_validate_detect_a_behavioural_regression_between_corpora() {
    let tmp = tempfile::tempdir().unwrap();

    // `true` always exits 0; `false` always exits 1 — a real, deterministic
    // behavioural difference on the same fixed boundary-probe corpus.
    run(tmp.path(), &["probe", "/bin/true"]);
    run(tmp.path(), &["probe", "/bin/false"]);

    let report = run_json(
        tmp.path(),
        &[
            "validate",
            "/bin/true",
            "/bin/false",
            "--reference-corpus",
            "true-boundary",
            "--replica-corpus",
            "false-boundary",
        ],
    );
    let behavioural = report["behavioural_differences"].as_array().unwrap();
    assert!(
        !behavioural.is_empty(),
        "expected an exit-code mismatch between /bin/true and /bin/false"
    );
    assert_eq!(report["disposition"], "NON_EQUIVALENT");
}

#[test]
fn converge_rolls_back_on_a_regressing_iteration() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("a"));
    std::fs::create_dir_all(tmp.path().join("b")).unwrap();

    // First iteration: nothing measured before, gets measured now, no
    // regressions recorded against it yet -> accept.
    let first = run_json(tmp.path(), &["converge", "a", "a"]);
    assert_eq!(first["decision"], "accept");

    // Second iteration compares against the *previous* saved score for
    // this pair; swapping in a structurally worse "replica" must roll
    // back rather than silently accept a regression.
    let tmp2 = tempfile::tempdir().unwrap();
    write_toy_project(&tmp2.path().join("a"));
    write_toy_project(&tmp2.path().join("b"));
    run(tmp2.path(), &["converge", "a", "b"]);
    std::fs::remove_file(tmp2.path().join("b/Cargo.toml")).unwrap();
    let second = run_json(tmp2.path(), &["converge", "a", "b"]);
    assert_eq!(second["decision"], "rollback");
}

#[test]
fn export_copies_evidence_out_of_the_state_directory() {
    let tmp = tempfile::tempdir().unwrap();
    write_toy_project(&tmp.path().join("ref"));
    run(tmp.path(), &["inspect", "ref"]);

    let out = run_json(tmp.path(), &["export", "out"]);
    assert!(out["exported_files"].as_u64().unwrap() > 0);
    assert!(tmp.path().join("out/evidence").exists());
}
