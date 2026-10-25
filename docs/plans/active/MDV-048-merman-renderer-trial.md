# MDV-048 — Merman renderer trial

Owner request: try Merman as this checkout's native Mermaid renderer.

## Scope

- Replace mermaid-rs-renderer with pinned Merman 0.9.1 in `src/diagram.rs`.
- Keep the existing blocking-worker, cache, SVG/RGBA, DOT, and math boundaries.
- Use Merman's resvg-safe pipeline and preserve application palette defaults
  and user init directives.
- Enable all diagram families, SVG, Cytoscape, and optional ELK. Bundle
  AWS Labs and Iconify logos packs for offline icons. Mermaid math-label
  support remains disabled; standalone LaTeX stays unchanged.
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

## ELK and offline icon follow-up (2026-10-25)

Owner requested ELK and AWS icon support on this trial branch. Dagre remains
the default; `%%{init: {"layout": "elk"}}%%` selects ELK. Cargo locks the optional
ELK adapter/layered implementation to 0.9.2 while Merman remains pinned to 0.9.1.
The ELK translation's EPL-2.0 text is retained with the third-party notices.

Bundled AWS Labs artwork and Iconify logos are registered lazily once, without
network or browser runtime dependencies. AWS architecture aliases are included.
Three AWS names with punctuation are normalized to hyphens, and unreferenced
duplicate XML IDs in Snowball Edge are removed; artwork geometry/colors remain
unchanged. Upstream provenance and icon licenses are in
`assets/mermaid-icons/NOTICES.md`.

A focused native Rust probe admitted 3,414 icon/alias entries and rendered the
ELK fixture containing both AWS and logos icons through Merman's resvg-safe SVG
pipeline. Formatting and diff checks passed. Full tests and native visual
acceptance were not repeated; the owner will inspect the local build manually.
The fixture is appended to `tests/fixtures/merman_trial.md`.

Final `cargo build --release --bin rmdv --locked` passed after the icon-pack
normalization. The executable is `target/release/rmdv`; existing import and
`block` future-compatibility warnings remain. Changes are local and uncommitted.

## Retained renderer and UI port (2026-10-25)

The owner accepted the Merman rendering result, asked to close Mermaid.js
PR #80, and requested moving the remaining UI work to this branch. The PR is
closed. Ported only `src/app` preview/width controls and `src/prefs.rs` from
`codex/mermaid-js-integration@4aa032b`; renderer, Cargo/build plumbing, JavaScript
workers and native WebKit helper were not copied.

Retained UI: fixed rounded opaque theme modal with translucent scrim, close
button/outside click, diagram pinch zoom and drag pan, palette-aware width
slider bounded by available reader space, saved preference, and shortcut-left /
reading-statistics-right footer row. The viewer's MIT notice is retained.
Existing local ELK/icon edits and unrelated untracked prototype/runtime files
are preserved. Combined native visual acceptance is pending. No commit or push
was requested for this port.

Combined `cargo build --release --bin rmdv --locked`, formatting and diff
checks passed. Automated suites were not rerun, following the owner’s manual
validation preference; native UI acceptance remains pending.

## Upstream submission authority (2026-10-25)

The owner requested a new PR for the combined Merman/ELK/icons and UI work,
authorizing the commit and fork push required to submit it. Earlier statements
about local-only/uncommitted scope describe the prior trial stage. No merge,
release or publication is authorized. The final locked release build and
format/diff checks passed; the full automated suite was last run on the initial
Merman trial, not on this combined follow-up.

Submitted in [PR #82](https://github.com/minchenlee/rmdv/pull/82) from
`chungchihhan:codex/merman-renderer` to `minchenlee:main` on 2026-10-25.
The PR is open; combined native acceptance and upstream merge remain pending.
