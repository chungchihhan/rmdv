# MDV-041 — Release rmdv v0.10.0

State: done (v0.10.0 published 2026-10-17)
Owner / accountable lead: Claude (gh-rmdv-orch)
Active writer: Claude
Created: 2026-10-17
Updated: 2026-10-17

## Outcome

Publish v0.10.0 from `main`: selectable and copyable text (MDV-040) and the
CJK reading changes, with the package version bumped, release notes written,
signed artifacts built by the Release workflow, and every platform asset
verified.

## Acceptance

- `Cargo.toml` and the root `rmdv` package in `Cargo.lock` say `0.10.0`.
- `cargo fmt --check`, `cargo check`, `cargo test --release` pass on the tagged
  commit.
- The `v0.10.0` tag is pushed with owner authority; the Release workflow
  succeeds on every job.
- Assets match `SHA256SUMS`; `latest.json` reports `0.10.0`.
- Not part of this task: the website deploy (separate owner decision).

## Evidence

Release notes: [`docs/releases/v0.10.0-release-notes.md`](../../releases/v0.10.0-release-notes.md).
Release: <https://github.com/minchenlee/rmdv/releases/tag/v0.10.0>, tag `v0.10.0` at `9e876c0`, Release run 39039978898 (all jobs green).
Release notes now follow `docs/releases/TEMPLATE.md`; the Release workflow publishes `docs/releases/<tag>-release-notes.md` as the release body.
