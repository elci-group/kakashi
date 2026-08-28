# tests/

`docs/ARCHITECTURE.md` §35 lists a top-level `tests/` directory alongside
`fixtures/`, `schemas/`, and `docs/`. This workspace's root `Cargo.toml` is
a virtual manifest (`[workspace]` only, no `[package]`), so Cargo has no
package here to own integration tests directly — a `tests/*.rs` file at
this path is never compiled or run by `cargo test`.

The tests actually live at the crate level, where Cargo can run them:

- `crates/kakashi-cli/tests/cli.rs` — end-to-end tests that spawn the
  compiled `kakashi` binary and exercise the full CLI surface: `init`,
  `inspect`, `fingerprint`, `diff`, `validate`, `observe`/`replay`,
  `probe`, `converge`, `export`.
- Unit tests inside each crate's `src/*.rs` (`#[cfg(test)] mod tests`) —
  see `crates/kakashi-core`, `kakashi-sandbox`, `kakashi-observe`,
  `kakashi-compare`, `kakashi-converge`, `kakashi-elci`, `kakashi-probe`,
  `kakashi-replicate`.

`cargo test --workspace` from the repository root runs all of the above.

This directory is kept (rather than deleted) so the layout still matches
the doctrine in §35 and so a future cross-crate system test — one that
genuinely needs to live above any single crate — has an obvious home.
