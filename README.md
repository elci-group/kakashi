# kakashi

The ELCI deterministic system replication & equivalence engine. Full architectural directive: `docs/ARCHITECTURE.md`.

## What's implemented

A working Rust workspace (`cargo build --workspace`, `cargo test --workspace`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo deny check`, and `cargo audit` all pass — enforced on every push/PR by `.github/workflows/ci.yml`) with a real, if v1-scoped, path through the doctrine:

- **`kakashi-core`** — the shared domain model: evidence envelope, fingerprints, equivalence dimensions/scores, disposition, intervention reasons, replication manifest.
- **`kakashi-evidence`** — the `.kakashi/` state directory (§34) and an append-only JSONL evidence/audit store.
- **`kakashi-observe`** — Stage A static discovery and deterministic fingerprinting (`SOURCE_SHA`, `DEPENDENCY_SHA`, `CONFIG_SHA`, `ENVIRONMENT_SHA`) over a real filesystem walk.
- **`kakashi-sandbox`** — confined execution with an *actually enforced* timeout (kills the whole process group, not just the direct child — see the test that caught this), an explicit environment allow-list instead of ambient inheritance, and filesystem-delta capture.
- **`kakashi-probe`** — Stage B CLI-surface extraction from `--help` text, and a fixed Stage D boundary-probe corpus (no-args, help, version, unknown flag, empty/oversized/nonexistent-path args).
- **`kakashi-compare`** — structural (file-set) and behavioural (probe-result) differential comparison, producing structured `Difference` records and per-dimension scores that are `None` (not `0.0`) when unmeasured.
- **`kakashi-replicate`** — builds the evidence-backed `ReplicationBrief`, then stops at `INTERVENTION_REQUIRED` (§46) rather than generating replica source itself — see the LLM boundary in §37.
- **`kakashi-converge`** — the monotonic-progress gate (§39): accepts an iteration only if the composite score didn't regress and no critical regressions were introduced.
- **`kakashi-elci`** — real adapter detection against the sibling ELCI projects on this machine (binary resolution via `PATH`/`target/{release,debug}`, library-kind presence via `Cargo.toml`), reporting `CAPABILITY_UNAVAILABLE` honestly when a sibling isn't there.
- **`kakashi-cli`** — all fifteen §33 subcommands, wired to the above, each supporting `--format human|json`, covered end-to-end by `crates/kakashi-cli/tests/cli.rs` against the compiled binary.

`schemas/` holds hand-maintained JSON Schema for `kakashi-core`'s wire types, validated against real captured CLI output (see `schemas/README.md` for what's covered and what isn't). `fixtures/` holds one real worked example of the current boundary-probe corpus format (see `fixtures/README.md` — this is not the full §24 fixture corpus, which isn't implemented). `tests/` explains why crate-level tests, not a root `tests/` directory, are where `cargo test --workspace` actually finds things (see `tests/README.md`).

### The "Copycat Ninja" theme

`--format human` output (never `--format json` — see `crates/kakashi-cli/src/theme.rs`'s module doc comment) is styled after Kakashi Hatake: Konoha-leaf green for a confirmed copy, Sharingan red for one that doesn't match, hitai-ate steel for plain information, plus a small banner on `kakashi status` and a braille spinner (`kakashi observe`/`probe`/`characterise`/`replicate`) while waiting on a sandboxed process. It's powered by `form3` — vendored into this workspace as `crates/form3` (a dependency-free ANSI styling/table/animation crate checked out at `~/3form` on this machine; see `crates/form3/README.md` for why it's vendored rather than a path dependency). Colors respect `NO_COLOR`, `TERM=dumb`, and drop out entirely when stdout isn't a terminal — piping any command's human output to a file or `less` gets plain text, exactly as before this feature existed.

Try it against a real sibling project:

```bash
kakashi init my-target --source /path/to/reference
kakashi inspect /path/to/reference
kakashi fingerprint /path/to/reference
kakashi probe /path/to/reference/target/release/some-binary
kakashi status   # shows which ELCI adapters are AVAILABLE vs CAPABILITY_UNAVAILABLE on this machine
```

## What's still directive, not code

Per Rule 4 (§2) — a claim needs an equivalence report, not just a paragraph — the following are explicitly **not** done, and the CLI never pretends otherwise:

- No PADAGONIA-backed persistent evidence graph; evidence is flat JSONL for now (§4, §19).
- No Cambrian-ranked intervention selection or Salience-based difference prioritisation (§9, §10) — `converge` runs a single fixed comparison per invocation.
- No LLM proposal step wired in (§37) — `replicate` produces the brief an LLM/human would need, and stops there.
- No cgroup/namespace-backed CPU/memory/process/network limits in the sandbox — declared in `ExecutionPolicy` but reported as unenforced caveats, never silently assumed (§31).
- No Fract/Chakra/Traci/Isopod *invocation* — `kakashi-elci` detects these siblings and reports their health, but doesn't yet shell out to them for deeper analysis; they're `AdapterKind::Library` or unwired `Binary` adapters today.
- No Tempcheq-backed caching (§11) — every command recomputes from scratch; nothing is cached across invocations.
- No anti-cheating or metamorphic testing (§27, §28) — not implemented, not stubbed.
- `ExecutionPolicy::caveats()` (§31) discloses that network/CPU/memory/process limits are unenforced, but does **not** currently disclose that filesystem access is equally unconfined — the child can read/write anywhere the host process can. Treat the sandbox as "structured subprocess execution with a real, tested timeout," not as isolation from the filesystem.
- `ReplicationManifest.random_seed` (§45) exists but is always `0` and unused — no subsystem here generates or consumes seeded randomness yet.

Anything reported as `EQUIVALENT` by `kakashi validate` reflects only the dimensions it actually measured (structural file-set diff, plus behavioural diff of two named boundary-probe corpora if you pass `--reference-corpus`/`--replica-corpus`) — not the full eight-dimension equivalence model in §26. Note also that `validate`'s `gate` only fails on a `Critical`-severity finding: a replica missing every source file still scores `High` severity (not `Critical`), so it can pass the gate while the composite score and disposition (`PARTIALLY_EQUIVALENT` or worse) show the degradation instead — see `crates/kakashi-cli/tests/cli.rs::validate_downgrades_disposition_when_structure_differs_but_gate_still_passes`.

## Man pages

`man/` holds one man page per subcommand (`man/kakashi.1`, `man/kakashi-probe.1`, …). They are generated from the real `clap` definitions in `crates/kakashi-cli/src/main.rs` — never hand-edit them; regenerate instead:

```bash
cargo run -p kakashi-cli --example gen-man
```
