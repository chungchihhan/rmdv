# rmdv — project status

Last verified: 2026-10-24 (Asia/Taipei; local MDV-050 prototype)
Stale after: 7 days
Current checkout: `/Users/harry_chung/Developer/Personal/rmdv`.
Integration base: `5e1b769`; branch: `codex/mermaid-js-integration`.
Remotes: owner fork `chungchihhan/rmdv` as origin; minchenlee/rmdv as upstream.
Latest published upstream release: v0.10.1 (GitHub API, 2026-10-24).
Historical release evidence below was not re-run.
Authority: This is a routing snapshot. Verify Git, GitHub, runtime identity, and
manual evidence before mutation.

## Current outcome

- `v0.8.0` (2026-10-09), `v0.9.0` (2026-10-16), and `v0.10.0` (2026-10-17) are published. v0.9.0 added
  rounded tables, Notion-like typography with Line spacing, and Quick Slot
  assign and confirm. Record:
  [`docs/releases/v0.9.0-content-pack.md`](docs/releases/v0.9.0-content-pack.md).
- `v0.10.0` shipped `MDV-040`: selectable and copyable text (drag, double
  and triple click, ⌘A, ⌘C under any input source, and one drag across blocks
  in the visible range) and CJK reading (bundled Noto Sans TC, taller CJK
  lines, squeezed full-width punctuation). The owner accepted it on a trial
  build. It uses a vendored `iced_selection` (see
  `vendor/iced_selection/VENDORED.md`).
- Known limits of cross-block selection: no selection past the window edge
  (no auto-scroll); an endpoint scrolled out of the virtual window copies its
  whole block; after a double or triple click, dragging into another block
  selects by character; Shift-extend works inside one block.

## Live workstreams

Owner requested [MDV-050](docs/plans/active/MDV-050-official-mermaid-js-prototype.md)
after rejecting the visual output of the MDV-049 Rust trial on 2026-10-24.
A standalone Mermaid.js 12.2.0 / Pele 1.9.0 comparison runs locally at
`http://127.0.0.1:4317`, with the original Rust references in a collapsible
panel. Both live engines support source edits, themes, and SVG export. A shared
local AWS Labs pack supplies 871 icons plus 279 source-derived aliases;
AWS flowchart and architecture examples are provided. AWS icon rendering and
export are now requirements for future desktop renderer selection.
The owner selected official Mermaid.js and authorized desktop integration on
2026-10-24. The integration branch now uses a persistent Node/Chrome worker
for Mermaid SVG and browser-rasterized pixels in the existing Iced cache.
DOT and math retain their native renderers. Local runtime dependencies and AWS
icons are prepared. The owner confirmed the desktop renderer works and requested
a 95%-opaque theme-background backing for enlarged diagrams to reduce document bleed-through.
Packaging remains source-checkout dependent; the owner manually verifies trial
builds, with no additional automated verification requested for this round.

## Human decisions / blockers

- Merges, tags, releases, and deployments need explicit owner authority per
  action. Pushing branches and opening PRs after verification is authorized for
  roadmap work.
- Visual work needs owner acceptance on device; batch it into one verify
  build per round.
- Phase-5 tasks (`MDV-032`–`MDV-035`) start with a design for the owner.
- GitHub Actions: the repo is public and uses standard runners only, which do
  not consume the owner's (exhausted) private-repo minutes. Keep it that way.
- Website deploys run locally with `wrangler deploy` by the owner. The
  `deploy-site` workflow fails because the repo has no `CLOUDFLARE_API_TOKEN`
  secret (run 39041702967); v0.10.0's site went live this way (Worker version
  `6e4b9588`, 2026-10-17).
- Release dispatches default to `dry_run: true`; a real release needs
  `dry_run: false` and owner authority.

## Next safe actions

1. Owner manually tries `target/release/rmdv` on the integration branch.
2. Record native acceptance and decide how to package Node/Chromium.

## Verification state

### Verified now

- MDV-050: exported seven identical sources through the actual Rust parser and
  SVG/RGBA path; Chrome 155 passed 26 browser checks, including both themes,
  CJK editing, errors/recovery, export, zoom, and narrow viewport overflow.
  No unexpected page errors, failed asset requests, or external requests were
  observed. Desktop screenshots were inspected; mobile layout geometry passed,
  but Chrome captures showed repeated strips, so mobile visual acceptance is
  still pending. The comparison was opened in the app browser. Rust
  library/application code was not changed, so the prior 627-test results below
  are historical evidence, not a new full-suite run. Formatting and diff
  checks pass for the new example and prototype.

- MDV-049: 627 tests passed, 2 ignored; cargo check, formatting, and diff
  checks passed. Exported light/dark PNGs were visually inspected. CJK,
  flowchart/sequence/class/state/mindmap, palette alpha blending, user init,
  and error/size handling passed through the SVG/RGBA path. Native-window
  acceptance remains pending. The updated release build passed and reports
  `rmdv 0.10.1`; no installation or publication was performed.

- `v0.10.0` was tagged at `9e876c0` (owner authority, 2026-10-17); the Release
  workflow passed every job and the seven installers match `SHA256SUMS`. The
  `aarch64` dmg and app were also checked by hand: codesign valid, stapled,
  `spctl` accepted as Notarized Developer ID, version `0.10.0`.
- Before the tag, the cross-block tree (`d53f0a3`) passed `cargo fmt --check`, `cargo check
  --all-targets` with no warnings, 536 library tests, and all integration
  suites in release mode.
- v0.9.0 release evidence is in the v0.9.0 content pack; v0.10.0 notes are in
  [`docs/releases/v0.10.0-release-notes.md`](docs/releases/v0.10.0-release-notes.md).

### Not verified / follow-up

- Cross-block selection was checked with headless screenshots, unit tests, and
  an owner trial build; there is no automated native-window test.
- The x86_64, Linux, and Windows v0.10.0 installers were checked by SHA-256
  only, not opened.
- `MDV-038`: a table row clipped at the viewport top can render without text
  after `goto` (pre-existing). `MDV-042`: `goto` stops about one block high.
- `HlCache` is bounded by entry count only (`MDV-002`).

## Routes

- Product contract: [`PRODUCT.md`](PRODUCT.md)
- User-facing overview: [`README.md`](README.md)
- Backlog: [`docs/BACKLOG.md`](docs/BACKLOG.md)
- Active plans: [`docs/plans/active/`](docs/plans/active/)
- Completed plans: [`docs/plans/completed/`](docs/plans/completed/)
- Release records: [`docs/releases/`](docs/releases/)
- Status history: [`docs/status-history/`](docs/status-history/)
- Module map: [`src/AGENTS.md`](src/AGENTS.md)
- Full Mindmap design: [`docs/superpowers/specs/2026-07-10-full-mindmap-mode-design.md`](docs/superpowers/specs/2026-07-10-full-mindmap-mode-design.md)
- CLI and IPC design: [`docs/superpowers/specs/2026-05-17-cli-agent-control-design.md`](docs/superpowers/specs/2026-05-17-cli-agent-control-design.md)

## Update contract

- Start: read the effective `AGENTS.md` chain and this file, then verify Git,
  GitHub, runtime identity, and the selected task's active plan.
- During work: keep one accountable lead and one active writer per mutable
  artifact; preserve unrelated dirty work and authority boundaries.
- End: update only facts and evidence changed by the session, advance task state
  only to the level proven, and keep no more than three next safe actions here.
- Move completed plans to `docs/plans/completed/` and chronological narrative to
  `docs/status-history/`; do not grow this file back into a work diary.
