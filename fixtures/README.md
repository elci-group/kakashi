# fixtures/

`docs/ARCHITECTURE.md` §24 describes a "Reference Behaviour Corpus" laid
out as `fixtures/{cli,api,malformed,boundary,filesystem,configuration,
concurrency,failure,performance}/`, where each fixture carries `input`,
`environment`, `expected_observables`, `comparison_policy`, and
`provenance`. **That corpus format is not implemented.** No crate in this
workspace reads or writes anything matching that shape yet.

What *is* real and implemented is the boundary-probe corpus `kakashi
probe` produces today: a fixed set of `Probe`/`ExecutionReport` pairs
(`crates/kakashi-probe/src/lib.rs`), saved as `.kakashi/experiments/*.json`
and indexed by `.kakashi/corpus/<name>.json`, consumed by `kakashi
validate --reference-corpus/--replica-corpus`.

`boundary/bin_true.json` in this directory is exactly that: the real,
unmodified output of `kakashi probe /bin/true` against this build (captured
2026-08-28), reformatted with `python -m json.tool` for readability. The
one exception is the `oversized_arg` probe's argument, which is truncated
in this file for readability — the real corpus sends the full 65536-byte
argument described in `boundary_probes()`.

Treat this directory as "one real worked example of the current corpus
format," not as a populated §24 fixture library. If you implement the §24
shape, this README (and the `fixtures/{cli,api,...}` layout it describes)
is the place to update.
