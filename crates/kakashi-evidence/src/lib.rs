//! The `.kakashi/` state directory (§34) and the append-only evidence
//! store built on top of it. Nothing under `.kakashi/` is source code —
//! it is Kakashi's own working memory and must never be read back as if
//! it were part of the reference or the replica.

use anyhow::{Context, Result};
use kakashi_core::EvidenceEnvelope;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const STATE_DIR: &str = ".kakashi";

/// Handle onto one `.kakashi/` state directory, created on demand.
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open (creating if necessary) the `.kakashi/` directory under `workdir`.
    pub fn open(workdir: impl AsRef<Path>) -> Result<Self> {
        let root = workdir.as_ref().join(STATE_DIR);
        for sub in [
            "fingerprints",
            "observations",
            "traces",
            "experiments",
            "corpus",
            "models",
            "evidence",
            "differences",
            "interventions",
            "reports",
            "snapshots",
        ] {
            fs::create_dir_all(root.join(sub))
                .with_context(|| format!("creating {}/{sub}", root.display()))?;
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, sub: &str) -> PathBuf {
        self.root.join(sub)
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.root.join("manifest.json")
    }

    pub fn audit_log_path(&self) -> PathBuf {
        self.root.join("audit.jsonl")
    }

    /// Append one evidence envelope to both the dimension-specific evidence
    /// log and the overall audit trail. Evidence is append-only: nothing in
    /// this crate offers a way to edit or delete a past entry.
    pub fn record(&self, envelope: &EvidenceEnvelope) -> Result<()> {
        append_jsonl(&self.path("evidence").join("evidence.jsonl"), envelope)?;
        append_jsonl(&self.audit_log_path(), envelope)?;
        Ok(())
    }

    pub fn read_evidence(&self) -> Result<Vec<EvidenceEnvelope>> {
        read_jsonl(&self.path("evidence").join("evidence.jsonl"))
    }

    /// Append one record to an arbitrary JSONL ledger under `sub/name.jsonl`
    /// (used for the convergence ledger, §39).
    pub fn append_ledger<T: Serialize>(&self, sub: &str, name: &str, value: &T) -> Result<()> {
        append_jsonl(&self.path(sub).join(format!("{name}.jsonl")), value)
    }

    pub fn read_ledger<T: DeserializeOwned>(&self, sub: &str, name: &str) -> Result<Vec<T>> {
        read_jsonl(&self.path(sub).join(format!("{name}.jsonl")))
    }

    pub fn save_json<T: Serialize>(&self, sub: &str, name: &str, value: &T) -> Result<PathBuf> {
        let path = self.path(sub).join(format!("{name}.json"));
        let bytes = serde_json::to_vec_pretty(value)?;
        fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }

    pub fn load_json<T: DeserializeOwned>(&self, sub: &str, name: &str) -> Result<T> {
        let path = self.path(sub).join(format!("{name}.json"));
        let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn list(&self, sub: &str) -> Result<Vec<String>> {
        let dir = self.path(sub);
        let mut names = Vec::new();
        if dir.exists() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                if let Some(stem) = entry.path().file_stem() {
                    names.push(stem.to_string_lossy().into_owned());
                }
            }
        }
        names.sort();
        Ok(names)
    }
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("opening {}", path.display()))?;
    let line = serde_json::to_string(value)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| Ok(serde_json::from_str(l)?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kakashi_core::Severity;

    #[test]
    fn state_dir_layout_is_created_on_open() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        for sub in [
            "models",
            "evidence",
            "differences",
            "experiments",
            "reports",
        ] {
            assert!(store.path(sub).is_dir(), "{sub} should exist");
        }
    }

    #[test]
    fn recorded_evidence_round_trips_and_is_append_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();

        let e1 = EvidenceEnvelope::new("subject-a", "source", "prov").with_severity(Severity::High);
        let e2 = EvidenceEnvelope::new("subject-b", "source", "prov");
        store.record(&e1).unwrap();
        store.record(&e2).unwrap();

        let read = store.read_evidence().unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].subject, "subject-a");
        assert_eq!(read[1].subject, "subject-b");

        // Audit log gets the same rows independently of the evidence log.
        let audit = fs::read_to_string(store.audit_log_path()).unwrap();
        assert_eq!(audit.lines().count(), 2);
    }

    #[test]
    fn save_and_load_json_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        store.save_json("models", "widget", &vec![1, 2, 3]).unwrap();
        let loaded: Vec<i32> = store.load_json("models", "widget").unwrap();
        assert_eq!(loaded, vec![1, 2, 3]);
    }
}
