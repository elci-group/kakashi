//! Stage B interface discovery and Stage D boundary probing (§23).
//! Every probe here runs through [`kakashi_sandbox::run_confined`] so the
//! same execution guarantees (and caveats) apply.

use kakashi_sandbox::{ExecutionPolicy, ExecutionReport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub label: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub probe: Probe,
    pub report: ExecutionReport,
}

/// A fixed, deterministic set of boundary cases (§24 "boundary/" fixture
/// class): no args, help, version, an unrecognised flag, an empty
/// argument, an oversized argument, and a path to a file that does not
/// exist. Not fuzzing — a reproducible minimum floor every target should
/// be probed with before anything target-specific is added.
pub fn boundary_probes() -> Vec<Probe> {
    vec![
        Probe {
            label: "no_args".into(),
            args: vec![],
        },
        Probe {
            label: "help".into(),
            args: vec!["--help".into()],
        },
        Probe {
            label: "version".into(),
            args: vec!["--version".into()],
        },
        Probe {
            label: "unknown_flag".into(),
            args: vec!["--kakashi-does-not-recognise-this-flag".into()],
        },
        Probe {
            label: "empty_arg".into(),
            args: vec!["".into()],
        },
        Probe {
            label: "oversized_arg".into(),
            args: vec!["A".repeat(1 << 16)],
        },
        Probe {
            label: "nonexistent_path".into(),
            args: vec!["/nonexistent/kakashi-probe-path-does-not-exist".into()],
        },
    ]
}

pub fn run_probes(program: &str, probes: &[Probe], policy: &ExecutionPolicy) -> Vec<ProbeResult> {
    probes
        .iter()
        .map(|probe| {
            let report = kakashi_sandbox::run_confined(program, &probe.args, policy)
                .unwrap_or_else(|e| ExecutionReport {
                    program: program.to_string(),
                    args: probe.args.clone(),
                    stdout: String::new(),
                    stderr: format!("kakashi-probe: failed to execute: {e}"),
                    exit_code: None,
                    timed_out: false,
                    duration_ms: 0,
                    filesystem_delta: None,
                    policy_caveats: policy.caveats(),
                });
            ProbeResult {
                probe: probe.clone(),
                report,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliSurface {
    pub flags: Vec<String>,
    pub subcommands: Vec<String>,
}

/// Heuristic extraction of an interface surface from `--help` text (§23
/// Stage B). This is a best-effort text scrape, not a parser for any
/// specific help-formatting convention — it is meant to seed the interface
/// fingerprint, not to be authoritative.
pub fn extract_cli_surface(help_text: &str) -> CliSurface {
    let mut flags = Vec::new();
    let mut subcommands = Vec::new();
    let mut in_commands_section = false;

    for line in help_text.lines() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("commands:") || lower.starts_with("subcommands:") {
            in_commands_section = true;
            continue;
        }
        if lower.ends_with(':') && !trimmed.starts_with('-') {
            in_commands_section = lower.starts_with("commands") || lower.starts_with("subcommands");
            continue;
        }

        if let Some(token) = trimmed.split_whitespace().next() {
            if token.starts_with("--") {
                let flag = token
                    .trim_end_matches(',')
                    .split('=')
                    .next()
                    .unwrap_or(token);
                push_unique(&mut flags, flag.to_string());
            } else if token.starts_with('-') && token.len() >= 2 && !token.starts_with("---") {
                push_unique(&mut flags, token.trim_end_matches(',').to_string());
            } else if in_commands_section
                && token
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                push_unique(&mut subcommands, token.to_string());
            }
        }
    }

    flags.sort();
    subcommands.sort();
    CliSurface { flags, subcommands }
}

fn push_unique(v: &mut Vec<String>, item: String) {
    if !v.contains(&item) {
        v.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_probes_cover_the_fixed_corpus_from_the_doctrine() {
        // (§23 Stage D / §24 "boundary/") — this is a reproducible minimum
        // floor, not fuzzing, so the exact label set matters.
        let probes = boundary_probes();
        let labels: Vec<&str> = probes.iter().map(|p| p.label.as_str()).collect();
        assert_eq!(
            labels,
            vec![
                "no_args",
                "help",
                "version",
                "unknown_flag",
                "empty_arg",
                "oversized_arg",
                "nonexistent_path",
            ]
        );
    }

    #[test]
    fn oversized_arg_probe_is_actually_oversized() {
        let probes = boundary_probes();
        let oversized = probes.iter().find(|p| p.label == "oversized_arg").unwrap();
        assert!(oversized.args[0].len() >= 1 << 16);
    }

    #[test]
    fn run_probes_reports_a_synthetic_failure_report_when_the_program_cannot_run() {
        let policy = ExecutionPolicy::new(std::env::temp_dir());
        let probes = vec![Probe {
            label: "no_args".into(),
            args: vec![],
        }];
        let results = run_probes(
            "/nonexistent/kakashi-probe-tests-not-a-real-binary",
            &probes,
            &policy,
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].report.exit_code.is_none());
        assert!(results[0].report.stderr.contains("failed to execute"));
    }

    #[test]
    fn extract_cli_surface_finds_long_and_short_flags() {
        // This is a best-effort line-scrape (only the first token per
        // line is inspected), not a full help-text parser — so a combined
        // "-v, --verbose" line only yields "-v". That's the documented
        // behaviour this test pins down, not a bug.
        let help = "\
Usage: widget [OPTIONS]

Options:
  -v, --verbose  Enable verbose output
  --format <FMT>  Output format
  -h, --help     Print help
";
        let surface = extract_cli_surface(help);
        assert!(surface.flags.contains(&"--format".to_string()));
        assert!(surface.flags.contains(&"-v".to_string()));
        assert!(surface.flags.contains(&"-h".to_string()));
    }

    #[test]
    fn extract_cli_surface_finds_subcommands_under_a_commands_heading() {
        let help = "\
Usage: widget <COMMAND>

Commands:
  build  Build the project
  test   Run tests
  help   Print this message

Options:
  -h, --help  Print help
";
        let surface = extract_cli_surface(help);
        assert_eq!(surface.subcommands, vec!["build", "help", "test"]);
        // Options heading must end the commands section — "-h, --help"
        // must not itself be misread as a subcommand.
        assert!(!surface.subcommands.contains(&"--help".to_string()));
    }

    #[test]
    fn extract_cli_surface_deduplicates_and_sorts() {
        let help = "--flag one\n--flag two\n--aaa\n";
        let surface = extract_cli_surface(help);
        assert_eq!(surface.flags, vec!["--aaa", "--flag"]);
    }
}
