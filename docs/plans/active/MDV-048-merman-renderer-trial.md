# MDV-048 — Merman renderer trial

Owner request: try Merman as this checkout's native Mermaid renderer.

## Scope

- Replace mermaid-rs-renderer with pinned Merman 0.9.1 in `src/diagram.rs`.
- Keep the existing blocking-worker, cache, SVG/RGBA, DOT, and math boundaries.
- Use Merman's resvg-safe pipeline and preserve application palette defaults
  and user init directives.
- Enable all diagram families, SVG, and Cytoscape; omit optional ELK and
  Mermaid math-label support in this trial. Standalone LaTeX stays unchanged.
- Update the declared Rust minimum to 1.95, as required by Merman.

## Acceptance

- Flowchart (including CJK), sequence, and representative additional diagram
  families render through the actual SVG-to-RGBA path.
- Both light/dark themes and user init directives work; invalid and oversized
  sources return errors.
- Focused renderer tests, cargo check, formatting, library and integration
  tests, release build, and diff checks are recorded with actual results.
- Supply a runnable local trial; native visual acceptance remains the owner's.

## Authority and evidence

Owner authorized a local branch and commit on 2026-10-19. Work branch:
`codex/merman-renderer`. No push, release, or deployment is authorized.
Started from clean `main@09d6394` at
`/Users/harry_chung/Developer/Personal/rmdv`. Remotes are the owner's fork
(`origin`) and `minchenlee/rmdv` (`upstream`). GitHub latest release was
verified as v0.10.1. No installed `rmdv` command was found.

## Local verification (2026-10-19)

- `cargo test --test diagram_render`: 8 passed.
- `cargo test --locked --lib --tests`: 627 passed, 2 ignored, no failures.
- `cargo check --locked`: passed with the default PDF feature.
- `cargo fmt --all --check` and `git diff --check`: passed.
- `cargo run --locked --example merman_smoke -- /tmp/rmdv-merman`: exported
  light/dark SVG and PNG through the app's actual worker and raster pipeline.
  Both PNGs are 1348 × 380; visually checked CJK glyphs, labels, arrows, and
  node geometry. This is not native-window acceptance.
- Existing warnings remain in `src/app/mod.rs` (ambiguous import visibility),
  `tests/ipc_protocol.rs` (unused `Section`), and dependency `block` (future
  incompatibility). These unrelated files were not changed.
- Only rustc 1.98.1 on macOS arm64 was used. Rust 1.95 is Merman's declared
  minimum, not a separately tested toolchain. Linux and Windows were not run.
- `cargo build --locked --release --bin rmdv`: passed in 3m09s;
  `./target/release/rmdv --version` reports `rmdv 0.10.1`. The local executable
  is about 48 MiB. No installation or publication was performed.
- Implementation and automated verification are complete for the local trial.
  Owner native-window acceptance remains pending; this plan is not accepted
  or complete. The local trial is retained on `codex/merman-renderer`.

## Reproduction

Run `cargo build --locked --release --bin rmdv`, then
`./target/release/rmdv tests/fixtures/merman_trial.md`. Check all diagram
sections, light/dark themes, user theme overrides, zoom/pan, and the error
card for the intentionally invalid block.

## Alpha-color follow-up (2026-10-19)

Owner requested a fix for the RGB encoder dropping theme alpha. Mermaid theme
fills and edges are now flattened over the opaque document background;
node labels and borders are flattened over the resolved node fill. Explicit
user init directives still bypass application theme injection. DOT and math
paths retain their existing behavior.

Regression coverage checks transparent, half-transparent, and opaque overlays,
and the actual SVG border colors: `#33363c` for One Dark and `#e3e3e3` for One
Light. All 627 tests passed (2 ignored), cargo check, formatting and diff checks
passed, and regenerated light/dark PNGs were visually inspected. The updated
release executable rebuilt successfully; native-window acceptance remains pending.
