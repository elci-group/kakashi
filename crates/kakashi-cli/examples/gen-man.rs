// Copyright (c) 2026 sal
// SPDX-License-Identifier: MIT
//! Regenerates `man/kakashi.1` (plus one page per subcommand) from the real
//! `clap` definitions in `src/main.rs`, so the shipped man pages can never
//! drift from the actual CLI surface. Run after changing the `Cli` enum:
//!
//!   cargo run --example gen-man
use clap::CommandFactory;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod kakashi_main;

use kakashi_main::Cli;
// Re-exports so `crate::{theme,experiment,output}` paths inside the included
// sources keep resolving (they normally live at the binary crate root).
use kakashi_main::{experiment, output, theme};

fn write_man(cmd: clap::Command, dest: &Path) -> std::io::Result<()> {
    let man = clap_mangen::Man::new(cmd);
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    std::fs::write(dest, buf)
}

fn gen(cmd: clap::Command, prefix: &str, man_dir: &Path, written: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut cmd = cmd;
    cmd.build();
    let dest = man_dir.join(format!("{prefix}.1"));
    write_man(cmd.clone(), &dest)?;
    println!("Wrote {}", dest.display());
    written.push(dest);
    for sub in cmd.get_subcommands() {
        // Skip clap's implicit `help` subcommand; document only real commands.
        if sub.get_name() == "help" {
            continue;
        }
        gen(
            sub.clone(),
            &format!("{prefix}-{}", sub.get_name()),
            man_dir,
            written,
        )?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    // man/ lives at the workspace root, next to README.md.
    let man_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("man");
    std::fs::create_dir_all(&man_dir)?;
    let mut written = Vec::new();
    gen(Cli::command(), "kakashi", &man_dir, &mut written)?;
    Ok(())
}
