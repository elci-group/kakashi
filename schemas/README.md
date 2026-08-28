# schemas/

JSON Schema (draft 2020-12) for the wire types `kakashi-core` actually
serializes, hand-maintained from `crates/kakashi-core/src/*.rs` rather than
generated. There is no `schemars`/build-step wiring yet — if you change a
struct in `kakashi-core`, update the matching schema here in the same
change, or it will silently drift.

Every schema in this directory was validated against real output captured
from a live `kakashi` run (not synthesized by hand) as of 2026-08-28.

| Schema | Rust source | Produced by |
|---|---|---|
| `evidence_envelope.schema.json` | `kakashi-core/src/envelope.rs` | every command, via `Store::record` (`.kakashi/evidence/evidence.jsonl`, `.kakashi/audit.jsonl`) |
| `difference.schema.json` | `kakashi-core/src/envelope.rs` | `kakashi diff`, `kakashi validate` |
| `severity.schema.json` | `kakashi-core/src/envelope.rs` | shared by the two above |
| `fingerprint.schema.json` | `kakashi-core/src/fingerprint.rs` | `kakashi fingerprint` |
| `replication_manifest.schema.json` | `kakashi-core/src/manifest.rs` | `kakashi init` (`.kakashi/manifest.json`) |
| `equivalence_scores.schema.json` | `kakashi-core/src/dimension.rs` | `kakashi validate`, `kakashi converge` |
| `disposition.schema.json` | `kakashi-core/src/disposition.rs` | `kakashi validate` |
| `intervention_reason.schema.json` | `kakashi-core/src/intervention.rs` | `kakashi replicate` |

## Known gaps

- `replication_manifest.schema.json` documents a real but partial
  implementation of docs/ARCHITECTURE.md §32: `tool_versions` is present
  but always empty in this build, and §32's `ELCI_versions`,
  `equivalence_policy`, `execution_policy`, and `test_corpus` fields don't
  exist on `ReplicationManifest` at all yet.
- `fingerprint.schema.json` documents that `interface_sha`,
  `behaviour_sha`, and `binary_sha` are always `null` today — no crate
  computes them yet (see README.md "what's still directive").
- There is no schema here yet for `Experiment`
  (`crates/kakashi-cli/src/experiment.rs`) or `ExecutionReport`/
  `ExecutionPolicy` (`kakashi-sandbox`) — those are CLI-internal
  persistence formats (`.kakashi/experiments/*.json`), not yet declared
  part of the stable evidence contract. See `fixtures/` for a worked
  example of that shape instead.
