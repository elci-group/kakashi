# form3 (vendored)

This crate is a vendored copy of `form3` (checked out at `~/3form` on this
machine, MIT licensed, © 2026 sal), included directly in this workspace
rather than depended on by path.

## Why vendored, not a path/git dependency

`~/3form` and `~/kakashi` are sibling checkouts on this machine (the same
layout `kakashi-elci`'s adapter registry already assumes for the rest of
the ELCI ecosystem — see `crates/kakashi-elci/src/registry.rs`). A path
dependency like `form3 = { path = "../../../3form" }` would resolve fine
here, but would break:

- `cargo build` for anyone who clones `elci-group/kakashi` alone, without
  also having `~/3form` checked out next to it;
- the `.github/workflows/ci.yml` build, which checks out only this repo.

`form3` has no dependencies of its own and no `elci-group/form3` repo
exists yet to depend on via git, so vendoring the ~800 lines verbatim is
the option that keeps `kakashi` buildable standalone. `kakashi-elci`'s
adapter registry now reports `3form` as `AdapterKind::Library` (it was
previously misclassified as `Binary` — the sibling checkout at `~/3form`
has no `[[bin]]` target; the unrelated `3form` executable on `PATH` is not
produced by this crate).

**If `form3` gets its own repo, or is published to crates.io, prefer
depending on that over this vendored copy** — this directory will then be
stale by construction and should be replaced, not kept in sync by hand.

## What it's used for here

`kakashi-cli`'s `theme` module (`crates/kakashi-cli/src/theme.rs`) uses
`form3::compat::Colorize` for the "Copycat Ninja" human-output color
scheme, `form3::table::Table` for the `status`/`validate` tables, and
`form3::anim::Spinner` for the progress indicator shown while `observe`/
`probe`/`characterise`/`replicate` wait on a sandboxed process. None of
this touches `--format json` output — JSON stays exactly as before.
