use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;

/// Result of Stage A static discovery (§23): what's in the tree, before
/// anything is executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticDiscovery {
    pub root: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub languages_by_extension: BTreeMap<String, usize>,
    pub build_systems: Vec<String>,
    pub entry_points: Vec<String>,
}

/// Manifest filenames that identify a build system, mapped to a label.
const BUILD_SYSTEM_MARKERS: &[(&str, &str)] = &[
    ("Cargo.toml", "cargo"),
    ("package.json", "npm/node"),
    ("pyproject.toml", "python/pyproject"),
    ("setup.py", "python/setuptools"),
    ("go.mod", "go"),
    ("Makefile", "make"),
    ("CMakeLists.txt", "cmake"),
    ("pom.xml", "maven"),
    ("build.gradle", "gradle"),
    ("Gemfile", "bundler"),
    ("mix.exs", "mix"),
];

/// Filenames treated as likely entry points regardless of language.
const ENTRY_POINT_MARKERS: &[&str] = &[
    "main.rs",
    "main.go",
    "main.py",
    "index.js",
    "index.ts",
    "main.c",
    "main.cpp",
    "app.py",
    "server.py",
    "cli.rs",
    "cli.py",
];

pub fn discover(root: &Path) -> Result<StaticDiscovery> {
    let mut file_count = 0usize;
    let mut total_bytes = 0u64;
    let mut languages_by_extension: BTreeMap<String, usize> = BTreeMap::new();
    let mut build_systems = Vec::new();
    let mut entry_points = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !crate::is_ignored(e.path()))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        file_count += 1;
        if let Ok(meta) = entry.metadata() {
            total_bytes += meta.len();
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            *languages_by_extension.entry(ext.to_string()).or_insert(0) += 1;
        }

        for (marker, label) in BUILD_SYSTEM_MARKERS {
            if name == *marker {
                let label = label.to_string();
                if !build_systems.contains(&label) {
                    build_systems.push(label);
                }
            }
        }
        if ENTRY_POINT_MARKERS.contains(&name.as_str()) {
            entry_points.push(rel.to_string_lossy().into_owned());
        }
    }

    build_systems.sort();
    entry_points.sort();

    Ok(StaticDiscovery {
        root: root.display().to_string(),
        file_count,
        total_bytes,
        languages_by_extension,
        build_systems,
        entry_points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_cargo_project_and_entry_point() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), b"[package]\nname = \"x\"").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src").join("main.rs"), b"fn main() {}").unwrap();

        let d = discover(dir.path()).unwrap();
        assert_eq!(d.file_count, 2);
        assert_eq!(d.build_systems, vec!["cargo".to_string()]);
        assert_eq!(d.entry_points, vec!["src/main.rs".to_string()]);
        assert_eq!(d.languages_by_extension.get("rs"), Some(&1));
    }

    #[test]
    fn ignores_target_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), b"[package]\nname = \"x\"").unwrap();
        fs::create_dir_all(dir.path().join("target").join("debug")).unwrap();
        fs::write(
            dir.path().join("target").join("debug").join("bin"),
            b"binary",
        )
        .unwrap();

        let d = discover(dir.path()).unwrap();
        assert_eq!(d.file_count, 1);
    }
}
