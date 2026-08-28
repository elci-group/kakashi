//! Controlled execution (§13, §31). Kakashi treats every reference and
//! replica as potentially hostile: no ambient environment inheritance, an
//! explicit timeout that is actually enforced, and an honest record of
//! which parts of the declared policy this version can and cannot back
//! with a real guarantee.
//!
//! CPU/memory/process-count limits are declared in [`ExecutionPolicy`] but
//! are **not** enforced by this crate yet (that needs platform-specific
//! rlimits/cgroups) — [`ExecutionReport::policy_caveats`] says so
//! explicitly rather than silently pretending they were applied.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub working_dir: PathBuf,
    /// Only these environment variables are passed to the child. Ambient
    /// environment is never inherited implicitly.
    pub allowed_env: BTreeMap<String, String>,
    pub timeout: Duration,
    pub network_access: bool,
    pub cpu_limit_seconds: Option<u64>,
    pub memory_limit_mb: Option<u64>,
    pub process_limit: Option<u32>,
    /// If set, filesystem entries under this path are diffed before/after
    /// execution to produce a filesystem delta (§6).
    pub watch_dir: Option<PathBuf>,
}

impl ExecutionPolicy {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
            allowed_env: BTreeMap::new(),
            timeout: Duration::from_secs(10),
            network_access: false,
            cpu_limit_seconds: None,
            memory_limit_mb: None,
            process_limit: None,
            watch_dir: None,
        }
    }

    /// Caveats that must be surfaced whenever this policy is reported as
    /// evidence, so a declared-but-unenforced limit is never mistaken for
    /// an enforced one.
    pub fn caveats(&self) -> Vec<String> {
        let mut caveats = Vec::new();
        if !self.network_access {
            caveats.push(
                "network_access=false is declared but not enforced by this build (no namespace/firewall isolation available)".into(),
            );
        }
        if self.cpu_limit_seconds.is_some() {
            caveats.push("cpu_limit_seconds is declared but not enforced".into());
        }
        if self.memory_limit_mb.is_some() {
            caveats.push("memory_limit_mb is declared but not enforced".into());
        }
        if self.process_limit.is_some() {
            caveats.push("process_limit is declared but not enforced".into());
        }
        caveats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemDelta {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub program: String,
    pub args: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub filesystem_delta: Option<FilesystemDelta>,
    pub policy_caveats: Vec<String>,
}

/// Run `program args...` under `policy`, enforcing the timeout for real by
/// killing the child if it overruns.
pub fn run_confined(
    program: &str,
    args: &[String],
    policy: &ExecutionPolicy,
) -> anyhow::Result<ExecutionReport> {
    let before = policy.watch_dir.as_deref().map(snapshot_dir).transpose()?;

    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(&policy.working_dir)
        .env_clear()
        .envs(&policy.allowed_env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        // Put the child in its own process group so a timeout can kill the
        // whole group, not just the immediate child. Without this, a
        // shell that forks (rather than exec's) a grandchild leaves that
        // grandchild running with the piped stdout fd still open, and
        // `wait_with_output` below blocks until it exits on its own —
        // silently defeating the timeout.
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    }

    let start = Instant::now();
    let mut child = cmd.spawn()?;

    let timed_out = loop {
        if let Some(_status) = child.try_wait()? {
            break false;
        }
        if start.elapsed() >= policy.timeout {
            kill_process_tree(&mut child);
            let _ = child.wait();
            break true;
        }
        std::thread::sleep(Duration::from_millis(20));
    };

    let output = child.wait_with_output()?;
    let duration_ms = start.elapsed().as_millis();

    let after = policy.watch_dir.as_deref().map(snapshot_dir).transpose()?;
    let filesystem_delta = match (before, after) {
        (Some(b), Some(a)) => Some(diff_snapshots(&b, &a)),
        _ => None,
    };

    Ok(ExecutionReport {
        program: program.to_string(),
        args: args.to_vec(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code(),
        timed_out,
        duration_ms,
        filesystem_delta,
        policy_caveats: policy.caveats(),
    })
}

/// Kill the child's entire process group (§13 — a timed-out experiment
/// must not leave orphaned work running). Falls back to killing just the
/// direct child on non-Unix targets, where no process-group kill is set up.
fn kill_process_tree(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL);
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
}

fn snapshot_dir(dir: &Path) -> anyhow::Result<BTreeMap<String, (u64, u64)>> {
    let mut snapshot = BTreeMap::new();
    if !dir.exists() {
        return Ok(snapshot);
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(dir).unwrap_or(entry.path());
            if let Ok(meta) = entry.metadata() {
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                snapshot.insert(rel.to_string_lossy().into_owned(), (meta.len(), mtime));
            }
        }
    }
    Ok(snapshot)
}

fn diff_snapshots(
    before: &BTreeMap<String, (u64, u64)>,
    after: &BTreeMap<String, (u64, u64)>,
) -> FilesystemDelta {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    for (path, meta) in after {
        match before.get(path) {
            None => added.push(path.clone()),
            Some(prev) if prev != meta => modified.push(path.clone()),
            _ => {}
        }
    }
    for path in before.keys() {
        if !after.contains_key(path) {
            removed.push(path.clone());
        }
    }
    added.sort();
    removed.sort();
    modified.sort();
    FilesystemDelta {
        added,
        removed,
        modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(dir: &Path) -> ExecutionPolicy {
        let mut p = ExecutionPolicy::new(dir);
        p.timeout = Duration::from_secs(5);
        p
    }

    #[test]
    fn captures_exit_code_and_stdout() {
        let dir = tempfile::tempdir().unwrap();
        let report = run_confined(
            "/bin/sh",
            &["-c".into(), "echo hi; exit 3".into()],
            &policy(dir.path()),
        )
        .unwrap();
        assert_eq!(report.exit_code, Some(3));
        assert_eq!(report.stdout.trim(), "hi");
        assert!(!report.timed_out);
    }

    #[test]
    fn enforces_timeout_by_killing_child() {
        let dir = tempfile::tempdir().unwrap();
        let mut p = policy(dir.path());
        p.timeout = Duration::from_millis(200);
        let report = run_confined("/bin/sh", &["-c".into(), "sleep 30".into()], &p).unwrap();
        assert!(report.timed_out);
        assert!(report.duration_ms < 5000);
    }

    #[test]
    fn does_not_inherit_ambient_environment_by_default() {
        std::env::set_var("KAKASHI_SANDBOX_TEST_VAR", "leaked");
        let dir = tempfile::tempdir().unwrap();
        let report = run_confined(
            "/bin/sh",
            &["-c".into(), "echo -n \"$KAKASHI_SANDBOX_TEST_VAR\"".into()],
            &policy(dir.path()),
        )
        .unwrap();
        std::env::remove_var("KAKASHI_SANDBOX_TEST_VAR");
        assert_eq!(report.stdout, "");
    }

    #[test]
    fn filesystem_delta_reports_added_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut p = policy(dir.path());
        p.watch_dir = Some(dir.path().to_path_buf());
        let report = run_confined(
            "/bin/sh",
            &["-c".into(), "echo hi > new_file.txt".into()],
            &p,
        )
        .unwrap();
        let delta = report.filesystem_delta.unwrap();
        assert_eq!(delta.added, vec!["new_file.txt".to_string()]);
    }
}
