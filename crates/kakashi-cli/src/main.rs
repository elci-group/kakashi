mod curly_expand;
mod commands;
mod experiment;
mod output;
mod theme;

use clap::{Parser, Subcommand};
use output::OutputFormat;

/// Kakashi — the ELCI deterministic system replication & equivalence
/// engine (§1). See `docs/` for the full architectural directive.
#[derive(Parser)]
#[command(name = "kakashi", version, about)]
struct Cli {
    /// Directory containing (or to contain) the `.kakashi/` state
    /// directory (§34).
    #[arg(long, global = true, default_value = ".")]
    state: String,

    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create the `.kakashi/` state directory and an initial manifest.
    Init {
        target: String,
        #[arg(long)]
        source: Option<String>,
    },
    /// Stage A static discovery of a reference tree.
    Inspect { path: String },
    /// Compute deterministic fingerprints for a tree (§22).
    Fingerprint {
        path: String,
        #[arg(long)]
        target: Option<String>,
    },
    /// Stage A + Stage B: static discovery plus CLI surface extraction.
    Characterise {
        path: String,
        /// Executable to run with `--help` to extract its CLI surface.
        #[arg(long)]
        program: Option<String>,
    },
    /// Run one confined invocation of `program` and record it as an
    /// experiment (§6, §13).
    Observe {
        program: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        #[arg(long, default_value_t = 10)]
        timeout_secs: u64,
        #[arg(long)]
        watch_dir: Option<String>,
        #[arg(long)]
        label: Option<String>,
    },
    /// Stage D boundary probing of `program` (§23, §24).
    Probe {
        program: String,
        #[arg(long, default_value_t = 10)]
        timeout_secs: u64,
    },
    /// Assemble a system model from previously recorded discovery,
    /// fingerprint, and ELCI adapter health (§35, §36).
    Model { path: String },
    /// Produce a replication brief (§37, §46) — evidence for the next
    /// proposal step, not a replica.
    Replicate {
        path: String,
        #[arg(long)]
        program: Option<String>,
    },
    /// Structural (and, if probe records are given, behavioural)
    /// differential comparison of two trees (§25).
    Diff { reference: String, replica: String },
    /// Score measured equivalence dimensions and report a gate
    /// PASS/FAIL disposition (§26, §41).
    Validate {
        reference: String,
        replica: String,
        /// Name of a boundary-probe corpus saved by `kakashi probe`
        /// against the reference (adds the behavioural dimension).
        #[arg(long)]
        reference_corpus: Option<String>,
        #[arg(long)]
        replica_corpus: Option<String>,
    },
    /// Run one convergence iteration and append it to the ledger (§38,
    /// §39).
    Converge {
        reference: String,
        replica: String,
        #[arg(long)]
        reference_corpus: Option<String>,
        #[arg(long)]
        replica_corpus: Option<String>,
    },
    /// Render the executive/machine-readable report (§41).
    Report { id: Option<String> },
    /// Re-run a recorded experiment from its manifest and check for
    /// reproducibility (§32).
    Replay { id: String },
    /// Copy the evidence graph, models, reports, and differences out of
    /// `.kakashi/` into a portable directory.
    Export { out_dir: String },
    /// Summarise manifest, evidence counts, and ELCI adapter health.
    Status,
}

fn __curly_original_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let format = cli.format;
    match cli.command {
        Commands::Init { target, source } => commands::init(&cli.state, &target, source, format),
        Commands::Inspect { path } => commands::inspect(&cli.state, &path, format),
        Commands::Fingerprint { path, target } => {
            commands::fingerprint(&cli.state, &path, target, format)
        }
        Commands::Characterise { path, program } => {
            commands::characterise(&cli.state, &path, program, format)
        }
        Commands::Observe {
            program,
            args,
            timeout_secs,
            watch_dir,
            label,
        } => commands::observe(
            &cli.state,
            &program,
            args,
            timeout_secs,
            watch_dir,
            label,
            format,
        ),
        Commands::Probe {
            program,
            timeout_secs,
        } => commands::probe(&cli.state, &program, timeout_secs, format),
        Commands::Model { path } => commands::model(&cli.state, &path, format),
        Commands::Replicate { path, program } => {
            commands::replicate(&cli.state, &path, program, format)
        }
        Commands::Diff { reference, replica } => {
            commands::diff(&cli.state, &reference, &replica, format)
        }
        Commands::Validate {
            reference,
            replica,
            reference_corpus,
            replica_corpus,
        } => commands::validate(
            &cli.state,
            &reference,
            &replica,
            reference_corpus,
            replica_corpus,
            format,
        ),
        Commands::Converge {
            reference,
            replica,
            reference_corpus,
            replica_corpus,
        } => commands::converge(
            &cli.state,
            &reference,
            &replica,
            reference_corpus,
            replica_corpus,
            format,
        ),
        Commands::Report { id } => commands::report(&cli.state, id, format),
        Commands::Replay { id } => commands::replay(&cli.state, &id, format),
        Commands::Export { out_dir } => commands::export(&cli.state, &out_dir, format),
        Commands::Status => commands::status(&cli.state, format),
    }
}

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    let mut positions: Vec<usize> = Vec::new();
    let mut fields: Vec<Vec<String>> = Vec::new();
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--state" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--state=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--state={}", v))
                    .collect(),
            );
            break;
        }
    }

    if fields.is_empty() || fields.iter().all(|f| f.len() <= 1) {
        __curly_original_main();
        return;
    }

    let combos = curly_expand::cartesian(&fields);
    let exe = std::env::current_exe().expect("resolve current exe");
    let mut had_failure = false;
    for combo in &combos {
        let mut new_args = raw_args.clone();
        for (slot, value) in positions.iter().zip(combo.iter()) {
            new_args[*slot] = value.clone();
        }
        let status = std::process::Command::new(&exe)
            .args(&new_args[1..])
            .status()
            .expect("failed to re-exec self");
        if !status.success() {
            had_failure = true;
        }
    }
    if had_failure {
        std::process::exit(1);
    }
}
