use crate::experiment::Experiment;
use crate::output::{emit, OutputFormat};
use anyhow::{Context, Result};
use kakashi_core::{
    Disposition, EquivalenceScores, EvidenceEnvelope, ReplicationManifest, Severity,
};
use kakashi_evidence::Store;
use kakashi_probe::ProbeResult;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn target_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "target".to_string())
}

fn resolve_program(program: &str) -> Result<PathBuf> {
    if program.contains(std::path::MAIN_SEPARATOR) {
        return Ok(PathBuf::from(program));
    }
    which::which(program).with_context(|| format!("'{program}' not found on PATH"))
}

/// PATH and HOME are explicitly (not silently) admitted to the sandbox
/// allow-list — without them most real-world binaries can't even resolve
/// their own subprocesses or config directories. Every other environment
/// variable is excluded (§31).
fn default_allowed_env() -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in ["PATH", "HOME"] {
        if let Ok(v) = std::env::var(key) {
            env.insert(key.to_string(), v);
        }
    }
    env
}

// ---------------------------------------------------------------- init ---

pub fn init(state: &str, target: &str, source: Option<String>, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let manifest = ReplicationManifest::new(target, source.unwrap_or_else(|| ".".to_string()));
    std::fs::write(store.manifest_path(), serde_json::to_vec_pretty(&manifest)?)?;
    emit(format, &manifest, |m| {
        println!(
            "initialised .kakashi/ for target '{}' (id {})",
            m.target, m.id
        );
        println!("state dir: {}", store.root().display());
    });
    Ok(())
}

// ------------------------------------------------------------- inspect ---

pub fn inspect(state: &str, path: &str, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let discovery = kakashi_observe::discover(Path::new(path))?;
    let name = target_name(path);
    store.save_json("models", &format!("{name}-discovery"), &discovery)?;

    let evidence = EvidenceEnvelope::new(path, "kakashi-observe::discover", "kakashi-cli::inspect")
        .with_evidence(json!(discovery));
    store.record(&evidence)?;

    emit(format, &discovery, |d| {
        println!("path:            {}", d.root);
        println!("files:           {}", d.file_count);
        println!("bytes:           {}", d.total_bytes);
        println!("build systems:   {}", d.build_systems.join(", "));
        println!("entry points:    {}", d.entry_points.join(", "));
        println!("languages (ext -> count):");
        for (ext, count) in &d.languages_by_extension {
            println!("  .{ext:<10} {count}");
        }
    });
    Ok(())
}

// ---------------------------------------------------------- fingerprint ---

pub fn fingerprint(
    state: &str,
    path: &str,
    target: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = Store::open(state)?;
    let name = target.unwrap_or_else(|| target_name(path));
    let fp = kakashi_observe::compute_fingerprint(Path::new(path), &name)?;
    store.save_json("fingerprints", &name, &fp)?;

    let evidence = EvidenceEnvelope::new(
        &name,
        "kakashi-observe::compute_fingerprint",
        "kakashi-cli::fingerprint",
    )
    .with_evidence(json!(fp));
    store.record(&evidence)?;

    emit(format, &fp, |fp| {
        println!("target:          {}", fp.target);
        println!(
            "source_sha:      {}",
            fp.source_sha.as_deref().unwrap_or("N/A")
        );
        println!(
            "dependency_sha:  {}",
            fp.dependency_sha.as_deref().unwrap_or("N/A")
        );
        println!(
            "config_sha:      {}",
            fp.config_sha.as_deref().unwrap_or("N/A")
        );
        println!(
            "environment_sha: {}",
            fp.environment_sha.as_deref().unwrap_or("N/A")
        );
        println!("interface_sha:   N/A (run `kakashi characterise` with --program)");
        println!("behaviour_sha:   N/A (run `kakashi probe`)");
        println!("binary_sha:      N/A (requires a build step)");
    });
    Ok(())
}

// --------------------------------------------------------- characterise ---

#[derive(Debug, Serialize)]
struct Characterisation {
    discovery: kakashi_observe::StaticDiscovery,
    cli_surface: Option<kakashi_probe::CliSurface>,
}

pub fn characterise(
    state: &str,
    path: &str,
    program: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = Store::open(state)?;
    let discovery = kakashi_observe::discover(Path::new(path))?;

    let cli_surface = match &program {
        None => None,
        Some(program) => {
            let resolved = resolve_program(program)?;
            let policy = kakashi_sandbox::ExecutionPolicy {
                timeout: Duration::from_secs(5),
                allowed_env: default_allowed_env(),
                ..kakashi_sandbox::ExecutionPolicy::new(std::env::current_dir()?)
            };
            let program_str = resolved.to_str().unwrap().to_string();
            let report = crate::theme::spin_while("characterising", move || {
                kakashi_sandbox::run_confined(&program_str, &["--help".to_string()], &policy)
            })?;
            Some(kakashi_probe::extract_cli_surface(&report.stdout))
        }
    };

    let model = Characterisation {
        discovery,
        cli_surface,
    };
    let name = target_name(path);
    store.save_json("models", &format!("{name}-characterisation"), &model)?;

    let evidence = EvidenceEnvelope::new(
        path,
        "kakashi-observe+kakashi-probe",
        "kakashi-cli::characterise",
    )
    .with_evidence(json!(model));
    store.record(&evidence)?;

    emit(format, &model, |m| {
        println!(
            "files: {}, build systems: {}",
            m.discovery.file_count,
            m.discovery.build_systems.join(", ")
        );
        match &m.cli_surface {
            Some(s) => {
                println!("flags:        {}", s.flags.join(", "));
                println!("subcommands:  {}", s.subcommands.join(", "));
            }
            None => println!("cli surface:  not extracted (pass --program)"),
        }
    });
    Ok(())
}

// -------------------------------------------------------------- observe ---

#[allow(clippy::too_many_arguments)]
pub fn observe(
    state: &str,
    program: &str,
    args: Vec<String>,
    timeout_secs: u64,
    watch_dir: Option<String>,
    label: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = Store::open(state)?;
    let resolved = resolve_program(program)?;
    let policy = kakashi_sandbox::ExecutionPolicy {
        timeout: Duration::from_secs(timeout_secs),
        allowed_env: default_allowed_env(),
        watch_dir: watch_dir.map(PathBuf::from),
        ..kakashi_sandbox::ExecutionPolicy::new(std::env::current_dir()?)
    };
    let program_str = resolved.to_str().unwrap().to_string();
    let spin_args = args.clone();
    let spin_policy = policy.clone();
    let report = crate::theme::spin_while("observing", move || {
        kakashi_sandbox::run_confined(&program_str, &spin_args, &spin_policy)
    })?;

    let experiment = Experiment {
        id: kakashi_core::new_id(),
        label: label.unwrap_or_else(|| "manual_observe".to_string()),
        program: resolved.display().to_string(),
        args,
        policy,
        report: report.clone(),
    };
    store.save_json("experiments", &experiment.id, &experiment)?;

    let severity = if report.timed_out {
        Severity::High
    } else if report.exit_code.map(|c| c != 0).unwrap_or(true) {
        Severity::Low
    } else {
        Severity::Informational
    };
    let mut evidence = EvidenceEnvelope::new(
        program,
        "kakashi-sandbox::run_confined",
        "kakashi-cli::observe",
    )
    .with_evidence(json!(experiment))
    .with_severity(severity);
    evidence.experiment_id = Some(experiment.id.clone());
    store.record(&evidence)?;

    emit(format, &experiment, |e| {
        println!("experiment id:   {}", e.id);
        println!(
            "exit code:       {}",
            crate::theme::exit_code_label(e.report.exit_code)
        );
        println!(
            "timed out:       {}",
            crate::theme::bool_label(e.report.timed_out)
        );
        println!("duration_ms:     {}", e.report.duration_ms);
        println!("stdout:\n{}", truncate(&e.report.stdout, 2000));
        if !e.report.stderr.is_empty() {
            println!("stderr:\n{}", truncate(&e.report.stderr, 2000));
        }
        println!("(replay with: kakashi replay {})", e.id);
    });
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}... [truncated, {} bytes total]", &s[..max], s.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_name_uses_the_final_path_component() {
        assert_eq!(target_name("/a/b/refproj"), "refproj");
        assert_eq!(target_name("refproj"), "refproj");
        assert_eq!(target_name("./refproj/"), "refproj");
    }

    #[test]
    fn target_name_falls_back_when_no_file_name_component_exists() {
        assert_eq!(target_name("/"), "target");
        assert_eq!(target_name(""), "target");
    }

    #[test]
    fn truncate_leaves_short_strings_untouched() {
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn truncate_cuts_long_strings_and_reports_original_length() {
        let long = "a".repeat(20);
        let out = truncate(&long, 5);
        assert!(out.starts_with("aaaaa..."));
        assert!(out.contains("20 bytes total"));
    }

    #[test]
    fn default_allowed_env_only_ever_admits_path_and_home() {
        // (§31) — every other ambient environment variable must be
        // excluded from what gets handed to an observed target.
        let env = default_allowed_env();
        for key in env.keys() {
            assert!(key == "PATH" || key == "HOME", "unexpected key: {key}");
        }
    }

    #[test]
    fn resolve_program_treats_a_path_with_a_separator_as_literal() {
        let resolved = resolve_program("./does-not-exist-anywhere").unwrap();
        assert_eq!(resolved, PathBuf::from("./does-not-exist-anywhere"));
    }

    #[test]
    fn resolve_program_errors_when_not_on_path() {
        assert!(resolve_program("kakashi-definitely-not-a-real-command").is_err());
    }
}

// ---------------------------------------------------------------- probe ---

pub fn probe(state: &str, program: &str, timeout_secs: u64, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let resolved = resolve_program(program)?;
    let policy = kakashi_sandbox::ExecutionPolicy {
        timeout: Duration::from_secs(timeout_secs),
        allowed_env: default_allowed_env(),
        ..kakashi_sandbox::ExecutionPolicy::new(std::env::current_dir()?)
    };
    let probes = kakashi_probe::boundary_probes();
    let program_str = resolved.to_str().unwrap().to_string();
    let spin_policy = policy.clone();
    let results = crate::theme::spin_while("probing", move || {
        kakashi_probe::run_probes(&program_str, &probes, &spin_policy)
    });

    let mut experiment_ids = Vec::new();
    for result in &results {
        let experiment = Experiment {
            id: kakashi_core::new_id(),
            label: result.probe.label.clone(),
            program: resolved.display().to_string(),
            args: result.probe.args.clone(),
            policy: policy.clone(),
            report: result.report.clone(),
        };
        store.save_json("experiments", &experiment.id, &experiment)?;
        experiment_ids.push(experiment.id.clone());

        let severity = if experiment.report.timed_out {
            Severity::Critical
        } else if experiment.report.exit_code.is_none() {
            Severity::High
        } else {
            Severity::Informational
        };
        let mut evidence = EvidenceEnvelope::new(
            program,
            "kakashi-probe::boundary_probes",
            "kakashi-cli::probe",
        )
        .with_evidence(json!(experiment))
        .with_severity(severity);
        evidence.experiment_id = Some(experiment.id.clone());
        store.record(&evidence)?;
    }

    let corpus_name = format!("{}-boundary", target_name(program));
    store.save_json("corpus", &corpus_name, &experiment_ids)?;

    emit(format, &results, |results: &Vec<ProbeResult>| {
        println!("boundary probe corpus: {corpus_name}");
        let mut table = form3::table::Table::new();
        table.set_header(vec!["Probe", "Exit", "Timed out", "Duration (ms)"]);
        for r in results {
            table.add_row(vec![
                r.probe.label.clone(),
                crate::theme::exit_code_label(r.report.exit_code).to_string(),
                crate::theme::bool_label(r.report.timed_out).to_string(),
                r.report.duration_ms.to_string(),
            ]);
        }
        print!("{table}");
        println!("(use --reference-corpus/--replica-corpus {corpus_name} with `kakashi validate`)");
    });
    Ok(())
}

// ---------------------------------------------------------------- model ---

#[derive(Debug, Serialize)]
struct AdapterHealthRecord {
    name: String,
    kind: String,
    available: bool,
    detail: String,
}

#[derive(Debug, Serialize)]
struct SystemModel {
    target: String,
    discovery: kakashi_observe::StaticDiscovery,
    fingerprint: kakashi_core::Fingerprint,
    elci_adapters: Vec<AdapterHealthRecord>,
}

pub fn model(state: &str, path: &str, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let name = target_name(path);
    let discovery = kakashi_observe::discover(Path::new(path))?;
    let fingerprint = kakashi_observe::compute_fingerprint(Path::new(path), &name)?;

    let elci_adapters: Vec<AdapterHealthRecord> = kakashi_elci::component_registry()
        .into_iter()
        .map(|a| {
            let health = a.health();
            AdapterHealthRecord {
                name: a.name().to_string(),
                kind: format!("{:?}", a.kind()),
                available: health.available,
                detail: health.detail,
            }
        })
        .collect();

    let model = SystemModel {
        target: name.clone(),
        discovery,
        fingerprint,
        elci_adapters,
    };
    store.save_json("models", &format!("{name}-model"), &model)?;

    let unavailable: Vec<&str> = model
        .elci_adapters
        .iter()
        .filter(|a| !a.available)
        .map(|a| a.name.as_str())
        .collect();
    let evidence = EvidenceEnvelope::new(path, "kakashi-cli::model", "kakashi-cli::model")
        .with_evidence(json!({"unavailable_adapters": unavailable}));
    store.record(&evidence)?;

    emit(format, &model, |m| {
        println!("target: {}  files: {}", m.target, m.discovery.file_count);
        println!("ELCI adapters:");
        let mut table = form3::table::Table::new();
        table.set_header(vec!["Adapter", "Status", "Kind", "Detail"]);
        for a in &m.elci_adapters {
            table.add_row(vec![
                a.name.clone(),
                crate::theme::adapter_status_label(a.available).to_string(),
                a.kind.clone(),
                a.detail.clone(),
            ]);
        }
        print!("{table}");
    });
    Ok(())
}

// ------------------------------------------------------------ replicate ---

pub fn replicate(
    state: &str,
    path: &str,
    program: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = Store::open(state)?;
    let name = target_name(path);
    let discovery = kakashi_observe::discover(Path::new(path))?;
    let fingerprint = kakashi_observe::compute_fingerprint(Path::new(path), &name)?;

    let (cli_surface, boundary_probes) = match &program {
        None => (None, Vec::new()),
        Some(program) => {
            let resolved = resolve_program(program)?;
            let policy = kakashi_sandbox::ExecutionPolicy {
                timeout: Duration::from_secs(5),
                allowed_env: default_allowed_env(),
                ..kakashi_sandbox::ExecutionPolicy::new(std::env::current_dir()?)
            };
            let program_str = resolved.to_str().unwrap().to_string();
            let (surface, results) = crate::theme::spin_while("replicating", move || {
                let help =
                    kakashi_sandbox::run_confined(&program_str, &["--help".to_string()], &policy)?;
                let surface = kakashi_probe::extract_cli_surface(&help.stdout);
                let probes = kakashi_probe::boundary_probes();
                let results = kakashi_probe::run_probes(&program_str, &probes, &policy);
                Ok::<_, anyhow::Error>((surface, results))
            })?;
            (Some(surface), results)
        }
    };

    let brief =
        kakashi_replicate::build_brief(&name, discovery, fingerprint, cli_surface, boundary_probes);
    store.save_json("models", &format!("{name}-brief"), &brief)?;

    let mut evidence = EvidenceEnvelope::new(
        path,
        "kakashi-replicate::build_brief",
        "kakashi-cli::replicate",
    )
    .with_evidence(json!(brief))
    .with_severity(Severity::Informational);
    evidence.proposed_actions = brief.notes.clone();
    store.record(&evidence)?;

    emit(format, &brief, |b| {
        println!(
            "replication brief for '{}' written to .kakashi/models/{}-brief.json",
            b.target, b.target
        );
        println!(
            "disposition: {} (§46) — Kakashi does not generate replica source itself",
            b.intervention
        );
        for note in &b.notes {
            println!("  - {note}");
        }
    });
    Ok(())
}

// ----------------------------------------------------------------- diff ---

pub fn diff(state: &str, reference: &str, replica: &str, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let diffs = kakashi_compare::diff_structure(Path::new(reference), Path::new(replica));

    let name = format!("{}-vs-{}", target_name(reference), target_name(replica));
    store.save_json("differences", &name, &diffs)?;

    let critical = diffs
        .iter()
        .filter(|d| d.severity == Severity::Critical)
        .count();
    let high = diffs
        .iter()
        .filter(|d| d.severity == Severity::High)
        .count();
    let evidence = EvidenceEnvelope::new(
        format!("{reference} vs {replica}"),
        "kakashi-compare::diff_structure",
        "kakashi-cli::diff",
    )
    .with_evidence(json!(diffs))
    .with_severity(if critical > 0 {
        Severity::Critical
    } else if high > 0 {
        Severity::High
    } else {
        Severity::Informational
    });
    store.record(&evidence)?;

    emit(format, &diffs, |diffs: &Vec<kakashi_core::Difference>| {
        if diffs.is_empty() {
            println!("no structural differences detected");
        }
        for d in diffs {
            println!(
                "[{}] {}",
                crate::theme::severity_label(d.severity),
                d.description
            );
        }
    });
    Ok(())
}

// ------------------------------------------------------------- validate ---

fn load_corpus(store: &Store, name: &str) -> Result<Vec<ProbeResult>> {
    let ids: Vec<String> = store.load_json("corpus", name)?;
    ids.into_iter()
        .map(|id| {
            let experiment: Experiment = store.load_json("experiments", &id)?;
            Ok(ProbeResult {
                probe: kakashi_probe::Probe {
                    label: experiment.label,
                    args: experiment.args,
                },
                report: experiment.report,
            })
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub scores: EquivalenceScores,
    pub structural_differences: Vec<kakashi_core::Difference>,
    pub behavioural_differences: Vec<kakashi_core::Difference>,
    pub gate: bool,
    pub disposition: String,
}

fn build_validation(
    reference: &str,
    replica: &str,
    reference_corpus: &Option<String>,
    replica_corpus: &Option<String>,
    store: &Store,
) -> Result<ValidationReport> {
    let structural_diffs =
        kakashi_compare::diff_structure(Path::new(reference), Path::new(replica));
    let ref_files = kakashi_observe::discover(Path::new(reference))?.file_count;
    let rep_files = kakashi_observe::discover(Path::new(replica))?.file_count;
    let structural_total = ref_files.max(rep_files).max(1);

    let (behavioural_diffs, behavioural_total) = match (reference_corpus, replica_corpus) {
        (Some(rc), Some(pc)) => {
            let ref_probes = load_corpus(store, rc)?;
            let rep_probes = load_corpus(store, pc)?;
            let total = ref_probes.len();
            (
                kakashi_compare::diff_probe_results(&ref_probes, &rep_probes),
                total,
            )
        }
        _ => (Vec::new(), 0),
    };

    let scores = kakashi_compare::build_scores(
        &structural_diffs,
        &behavioural_diffs,
        structural_total,
        behavioural_total,
    );
    let has_critical = structural_diffs
        .iter()
        .chain(behavioural_diffs.iter())
        .any(|d| d.severity == Severity::Critical);
    let gate = scores.composite().is_some() && !has_critical;

    let disposition = match (scores.composite(), gate) {
        (None, _) => Disposition::Inconclusive,
        (Some(_), false) => Disposition::NonEquivalent,
        (Some(c), true) if c >= 0.999 => Disposition::Equivalent,
        (Some(c), true) if c >= 0.9 => Disposition::ConditionallyEquivalent,
        (Some(_), true) => Disposition::PartiallyEquivalent,
    };

    Ok(ValidationReport {
        scores,
        structural_differences: structural_diffs,
        behavioural_differences: behavioural_diffs,
        gate,
        disposition: disposition.to_string(),
    })
}

pub fn validate(
    state: &str,
    reference: &str,
    replica: &str,
    reference_corpus: Option<String>,
    replica_corpus: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = Store::open(state)?;
    let report = build_validation(
        reference,
        replica,
        &reference_corpus,
        &replica_corpus,
        &store,
    )?;

    let name = format!("{}-vs-{}", target_name(reference), target_name(replica));
    store.save_json("reports", &format!("{name}-validation"), &report)?;

    let evidence = EvidenceEnvelope::new(
        format!("{reference} vs {replica}"),
        "kakashi-compare",
        "kakashi-cli::validate",
    )
    .with_evidence(json!(report));
    store.record(&evidence)?;

    emit(format, &report, |r| {
        let mut table = form3::table::Table::new();
        table.set_header(vec!["Dimension", "Score"]);
        for (dim, score) in &r.scores.scores {
            let cell = match score {
                Some(s) => format!("{s:.2}"),
                None => "N/A".to_string(),
            };
            table.add_row(vec![dim.clone(), cell]);
        }
        let overall = match r.scores.composite() {
            Some(c) => format!("{c:.2}"),
            None => "N/A".to_string(),
        };
        table.add_row(vec!["Overall".to_string(), overall]);
        print!("{table}");

        println!("Gate            {}", crate::theme::gate_label(r.gate));
        println!(
            "Disposition     {}",
            crate::theme::disposition_label(&r.disposition)
        );
        println!("  {}", crate::theme::disposition_tagline(&r.disposition));
        if !r.structural_differences.is_empty() {
            println!("Structural differences: {}", r.structural_differences.len());
        }
        if !r.behavioural_differences.is_empty() {
            println!(
                "Behavioural differences: {}",
                r.behavioural_differences.len()
            );
        }
    });
    Ok(())
}

// ------------------------------------------------------------- converge ---

pub fn converge(
    state: &str,
    reference: &str,
    replica: &str,
    reference_corpus: Option<String>,
    replica_corpus: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = Store::open(state)?;
    let name = format!("{}-vs-{}", target_name(reference), target_name(replica));

    let before: EquivalenceScores = store
        .load_json("models", &format!("{name}-scores"))
        .unwrap_or_default();
    let report = build_validation(
        reference,
        replica,
        &reference_corpus,
        &replica_corpus,
        &store,
    )?;
    let after = report.scores.clone();

    let previous_entries: Vec<kakashi_converge::LedgerEntry> = store
        .read_ledger("reports", &format!("{name}-ledger"))
        .unwrap_or_default();
    let iteration = previous_entries.len() as u64 + 1;
    let regressions = report
        .structural_differences
        .iter()
        .chain(report.behavioural_differences.iter())
        .filter(|d| d.severity == Severity::Critical)
        .count();

    let entry = kakashi_converge::evaluate_iteration(
        iteration,
        &before,
        &after,
        Vec::new(),
        report.structural_differences.len() + report.behavioural_differences.len(),
        regressions,
    );
    store.append_ledger("reports", &format!("{name}-ledger"), &entry)?;
    store.save_json("models", &format!("{name}-scores"), &after)?;

    let evidence = EvidenceEnvelope::new(
        format!("{reference} vs {replica}"),
        "kakashi-converge::evaluate_iteration",
        "kakashi-cli::converge",
    )
    .with_evidence(json!(entry));
    store.record(&evidence)?;

    emit(format, &entry, |e| {
        println!("iteration:    {}", e.iteration);
        println!(
            "before:       {}",
            e.before_score
                .map(|s| format!("{s:.2}"))
                .unwrap_or_else(|| "N/A".into())
        );
        println!(
            "after:        {}",
            e.after_score
                .map(|s| format!("{s:.2}"))
                .unwrap_or_else(|| "N/A".into())
        );
        println!(
            "delta:        {}",
            e.delta
                .map(|d| format!("{d:+.2}"))
                .unwrap_or_else(|| "N/A".into())
        );
        println!("regressions:  {}", e.regressions);
        let accepted = e.decision == kakashi_converge::Decision::Accept;
        println!(
            "decision:     {}",
            crate::theme::decision_label(&format!("{:?}", e.decision), accepted)
        );
        if e.decision == kakashi_converge::Decision::Rollback {
            println!("(convergence budget not exhausted, but this iteration did not improve — see §39 monotonic progress requirement)");
        }
    });
    Ok(())
}

// --------------------------------------------------------------- report ---

pub fn report(state: &str, _id: Option<String>, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let manifest: Option<ReplicationManifest> = std::fs::read(store.manifest_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok());
    let evidence_count = store.read_evidence().map(|v| v.len()).unwrap_or(0);
    let models = store.list("models").unwrap_or_default();
    let differences = store.list("differences").unwrap_or_default();
    let experiments = store.list("experiments").unwrap_or_default();

    // `report` summarises accumulated evidence; it does not itself run
    // `validate`, so it never has a scored gate to report a disposition
    // other than INCONCLUSIVE from (§41) — run `kakashi validate` for a
    // real gate.
    let disposition = Disposition::Inconclusive;

    let summary = json!({
        "manifest": manifest,
        "evidence_count": evidence_count,
        "models": models,
        "differences": differences,
        "experiment_count": experiments.len(),
        "disposition": disposition.to_string(),
    });

    emit(format, &summary, |s| {
        if let Some(m) = s.get("manifest").filter(|m| !m.is_null()) {
            println!(
                "target:          {}",
                m.get("target").and_then(|v| v.as_str()).unwrap_or("?")
            );
        } else {
            println!("(no manifest — run `kakashi init` first)");
        }
        println!("evidence rows:   {}", s["evidence_count"]);
        println!("models:          {}", s["models"]);
        println!("differences:     {}", s["differences"]);
        println!("experiments:     {}", s["experiment_count"]);
        println!(
            "disposition:     {}",
            crate::theme::disposition_label(s["disposition"].as_str().unwrap_or("?"))
        );
        println!("(disposition is INCONCLUSIVE by default in this build — run `kakashi validate` and wire its result in for a real gate)");
    });
    Ok(())
}

// --------------------------------------------------------------- replay ---

pub fn replay(state: &str, id: &str, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let original: Experiment = store
        .load_json("experiments", id)
        .with_context(|| format!("no experiment recorded with id {id}"))?;

    let rerun = kakashi_sandbox::run_confined(&original.program, &original.args, &original.policy)?;

    let reproducible = rerun.exit_code == original.report.exit_code
        && rerun.stdout == original.report.stdout
        && rerun.stderr == original.report.stderr;

    let result = json!({
        "experiment_id": id,
        "reproducible": reproducible,
        "original_exit_code": original.report.exit_code,
        "rerun_exit_code": rerun.exit_code,
    });
    let evidence = EvidenceEnvelope::new(
        &original.program,
        "kakashi-sandbox::run_confined",
        "kakashi-cli::replay",
    )
    .with_evidence(result.clone())
    .with_severity(if reproducible {
        Severity::Informational
    } else {
        Severity::High
    });
    store.record(&evidence)?;

    emit(format, &result, |_| {
        println!(
            "REPRODUCIBILITY: {}",
            if reproducible { "PASSED" } else { "FAILED" }
        );
        if !reproducible {
            println!(
                "  original exit_code={:?} stdout_len={}",
                original.report.exit_code,
                original.report.stdout.len()
            );
            println!(
                "  rerun    exit_code={:?} stdout_len={}",
                rerun.exit_code,
                rerun.stdout.len()
            );
        }
    });
    Ok(())
}

// --------------------------------------------------------------- export ---

pub fn export(state: &str, out_dir: &str, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let out = PathBuf::from(out_dir);
    let mut copied = 0usize;
    for sub in ["models", "reports", "evidence", "differences", "corpus"] {
        let src = store.path(sub);
        let dst = out.join(sub);
        std::fs::create_dir_all(&dst)?;
        if src.exists() {
            for entry in walkdir::WalkDir::new(&src)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let rel = entry.path().strip_prefix(&src).unwrap();
                    let dest_path = dst.join(rel);
                    if let Some(parent) = dest_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(entry.path(), &dest_path)?;
                    copied += 1;
                }
            }
        }
    }
    emit(
        format,
        &json!({"exported_files": copied, "out_dir": out_dir}),
        |_| {
            println!("exported {copied} files to {out_dir}");
        },
    );
    Ok(())
}

// --------------------------------------------------------------- status ---

pub fn status(state: &str, format: OutputFormat) -> Result<()> {
    let store = Store::open(state)?;
    let manifest: Option<ReplicationManifest> = std::fs::read(store.manifest_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok());
    let adapters: Vec<AdapterHealthRecord> = kakashi_elci::component_registry()
        .into_iter()
        .map(|a| {
            let health = a.health();
            AdapterHealthRecord {
                name: a.name().to_string(),
                kind: format!("{:?}", a.kind()),
                available: health.available,
                detail: health.detail,
            }
        })
        .collect();

    let summary = json!({
        "manifest": manifest,
        "evidence_count": store.read_evidence().map(|v| v.len()).unwrap_or(0),
        "experiment_count": store.list("experiments").unwrap_or_default().len(),
        "model_count": store.list("models").unwrap_or_default().len(),
        "elci_adapters": adapters,
    });

    emit(format, &summary, |s| {
        println!("{}", crate::theme::banner());
        println!("state dir:       {}", store.root().display());
        if let Some(m) = s.get("manifest").filter(|m| !m.is_null()) {
            println!("target:          {}", m["target"]);
        } else {
            println!("target:          (uninitialised — run `kakashi init`)");
        }
        println!("evidence rows:   {}", s["evidence_count"]);
        println!("experiments:     {}", s["experiment_count"]);
        println!("models:          {}", s["model_count"]);
        println!("ELCI adapters (known jutsu):");
        let mut table = form3::table::Table::new();
        table.set_header(vec!["Adapter", "Status", "Kind"]);
        for a in &adapters {
            table.add_row(vec![
                a.name.clone(),
                crate::theme::adapter_status_label(a.available).to_string(),
                a.kind.clone(),
            ]);
        }
        print!("{table}");
    });
    Ok(())
}
