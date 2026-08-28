use anyhow::Result;
use kakashi_core::Fingerprint;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const DEPENDENCY_MARKERS: &[&str] = &[
    "Cargo.lock",
    "Cargo.toml",
    "package-lock.json",
    "package.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "requirements.txt",
    "Pipfile.lock",
    "go.sum",
    "go.mod",
    "poetry.lock",
];

const CONFIG_EXTENSIONS: &[&str] = &["toml", "yaml", "yml", "ini", "cfg"];

/// Compute all deterministic fingerprint dimensions this crate is able to
/// measure without executing the target (§22). `interface_sha`,
/// `behaviour_sha`, and `binary_sha` are left `None` here — they require
/// probing or a build, which belong to other crates.
pub fn compute_fingerprint(root: &Path, target: &str) -> Result<Fingerprint> {
    let mut fp = Fingerprint::new(target);
    fp.source_sha = Some(hash_source_tree(root)?);
    fp.dependency_sha = hash_named_files(root, DEPENDENCY_MARKERS)?;
    fp.config_sha = hash_config_files(root)?;
    fp.environment_sha = Some(hash_environment());
    Ok(fp)
}

/// Hash every non-ignored file's relative path and content, in sorted path
/// order, so the digest doesn't depend on filesystem iteration order.
fn hash_source_tree(root: &Path) -> Result<String> {
    let mut paths: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !crate::is_ignored(e.path()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    paths.sort();

    let mut hasher = Sha256::new();
    for path in paths {
        let rel = path.strip_prefix(root).unwrap_or(&path);
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update([0u8]);
        if let Ok(bytes) = fs::read(&path) {
            hasher.update(&bytes);
        }
        hasher.update([0u8]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn hash_named_files(root: &Path, names: &[&str]) -> Result<Option<String>> {
    let mut found: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !crate::is_ignored(e.path()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| names.contains(&n))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if found.is_empty() {
        return Ok(None);
    }
    found.sort();

    let mut hasher = Sha256::new();
    for path in found {
        let rel = path.strip_prefix(root).unwrap_or(&path);
        hasher.update(rel.to_string_lossy().as_bytes());
        if let Ok(bytes) = fs::read(&path) {
            hasher.update(&bytes);
        }
    }
    Ok(Some(hex::encode(hasher.finalize())))
}

fn hash_config_files(root: &Path) -> Result<Option<String>> {
    let mut found: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !crate::is_ignored(e.path()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| CONFIG_EXTENSIONS.contains(&x))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if found.is_empty() {
        return Ok(None);
    }
    found.sort();

    let mut hasher = Sha256::new();
    for path in found {
        let rel = path.strip_prefix(root).unwrap_or(&path);
        hasher.update(rel.to_string_lossy().as_bytes());
        if let Ok(bytes) = fs::read(&path) {
            hasher.update(&bytes);
        }
    }
    Ok(Some(hex::encode(hasher.finalize())))
}

fn hash_environment() -> String {
    let descriptor = format!(
        "os={} arch={} family={} cpus={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::consts::FAMILY,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    );
    let mut hasher = Sha256::new();
    hasher.update(descriptor.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_sha_is_stable_across_recomputation() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), b"fn main() {}").unwrap();

        let a = compute_fingerprint(dir.path(), "t").unwrap();
        let b = compute_fingerprint(dir.path(), "t").unwrap();
        assert_eq!(a.source_sha, b.source_sha);
    }

    #[test]
    fn source_sha_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), b"fn main() {}").unwrap();
        let before = compute_fingerprint(dir.path(), "t").unwrap();

        fs::write(
            dir.path().join("main.rs"),
            b"fn main() { println!(\"hi\"); }",
        )
        .unwrap();
        let after = compute_fingerprint(dir.path(), "t").unwrap();

        assert_ne!(before.source_sha, after.source_sha);
    }

    #[test]
    fn ignored_dirs_do_not_affect_source_sha() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), b"fn main() {}").unwrap();
        let before = compute_fingerprint(dir.path(), "t").unwrap();

        fs::create_dir_all(dir.path().join("target").join("debug")).unwrap();
        fs::write(
            dir.path().join("target").join("debug").join("junk"),
            b"build output",
        )
        .unwrap();
        let after = compute_fingerprint(dir.path(), "t").unwrap();

        assert_eq!(before.source_sha, after.source_sha);
    }

    #[test]
    fn dependency_sha_is_none_when_no_manifest_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), b"fn main() {}").unwrap();
        let fp = compute_fingerprint(dir.path(), "t").unwrap();
        assert!(fp.dependency_sha.is_none());
    }
}
