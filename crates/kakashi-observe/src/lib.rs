//! Stage A/B characterisation (§23): static discovery and deterministic
//! fingerprinting of a reference tree. Nothing in this crate executes the
//! target — that's `kakashi-probe`'s job. This crate only looks.

pub mod discovery;
pub mod fingerprint;

pub use discovery::{discover, StaticDiscovery};
pub use fingerprint::compute_fingerprint;

/// Directories that are build/VCS/dependency-cache noise, never source
/// evidence, regardless of language (§12 — generated state must never
/// silently become source evidence).
pub const IGNORED_DIRS: &[&str] = &[
    ".git",
    ".kakashi",
    "target",
    "node_modules",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "venv",
    ".mypy_cache",
    ".pytest_cache",
];

pub fn is_ignored(path: &std::path::Path) -> bool {
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        IGNORED_DIRS.contains(&s.as_ref())
    })
}
