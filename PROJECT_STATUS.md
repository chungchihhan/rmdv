# rmdv — project status

Last verified: 2026-10-25 (Asia/Taipei; local MDV-048 follow-up)
Stale after: 7 days
Current checkout: `/Users/harry_chung/Developer/Personal/rmdv` (owner's fork).
Trial base: `main@09d6394`; work branch: `codex/merman-renderer` (MDV-048).
Remotes: `origin` = `chungchihhan/rmdv`, `upstream` = `minchenlee/rmdv`.
Latest published upstream release: `v0.10.1` (verified via GitHub API on
2026-10-19). Historical release evidence below was not re-run.
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

Local owner-requested trial: [`MDV-048`](docs/plans/active/MDV-048-merman-renderer-trial.md)
replaces the Mermaid renderer with Merman 0.9.1. It raises the declared Rust
minimum to 1.95; local rustc is 1.98.1. ELK is enabled alongside default Dagre,
and AWS/logos icon packs are embedded for offline rendering. The owner accepted
this renderer trial on 2026-10-25 and requested retaining it instead of
Mermaid.js. [PR #80](https://github.com/minchenlee/rmdv/pull/80) is closed.
Preview modal, pinch/pan gestures and the reading-width control are now ported
from that branch; acceptance of the combined build remains pending. No installed
`rmdv` command was found at trial start.

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

1. Inspect the combined Merman build: modal, pinch/pan, width slider,
   theme colors, footer order and restart persistence.
2. Review the local combined diff before committing or proposing a replacement PR.

## Verification state

### Verified now

- Combined Merman + modal/width UI port: locked release build, formatting and
  diff checks passed on 2026-10-25. Native combined-build acceptance is pending;
  the owner authorized commit, push and a new upstream PR.

- 2026-10-25 MDV-048 follow-up: release build, formatting and diff checks
  passed. A focused Merman probe loaded 3,414 icon/alias entries and rendered
  ELK with AWS/logos through the resvg-safe SVG pipeline. Full tests and native
  visual acceptance were not repeated. This follow-up is included in the owner-authorized PR submission.

- MDV-048 local trial: 627 tests passed, 2 ignored (`cargo test --locked
  --lib --tests`); `cargo check --locked`, formatting and diff checks passed.
  The SVG/RGBA pipeline passed CJK flowchart, sequence, class, state, mindmap,
  both palettes, user init, and error/size checks. Semi-transparent Mermaid
  theme colors are composited onto their backdrop before RGB encoding, with
  regression coverage for light/dark node borders. Exported light/dark PNGs
  were visually inspected; native-window acceptance remains pending.
  `cargo build --locked --release --bin rmdv` passed; the local binary reports
  `rmdv 0.10.1` and has not been installed or published.
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
