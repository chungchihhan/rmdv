# rmdv — project status

Last verified: 2026-10-17 (Asia/Taipei)
Stale after: 7 days
Canonical repository: `~/Documents/GitHub/mdv`
Expected branch / HEAD / PR: start new work from the live `origin/main`
(`d53f0a3` when verified, before the v0.10.0 docs and version commits). Latest
published release: `v0.9.0` → `2a78558` (2026-10-16). `v0.10.0` is being
prepared (`MDV-041`); it is not tagged.
Authority: This is a routing snapshot. Verify Git, GitHub, runtime identity, and
manual evidence before mutation.

## Current outcome

- `v0.8.0` (2026-10-09) and `v0.9.0` (2026-10-16) are published. v0.9.0 added
  rounded tables, Notion-like typography with Line spacing, and Quick Slot
  assign and confirm. Record:
  [`docs/releases/v0.9.0-content-pack.md`](docs/releases/v0.9.0-content-pack.md).
- `main` has since gained `MDV-040`: selectable and copyable text (drag, double
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

| ID | State | Owner | Outcome | Acceptance | Plan |
| --- | --- | --- | --- | --- | --- |
| MDV-041 | in_progress | Claude | Release v0.10.0. | Version bump, tests green, tag with owner authority, Release workflow green, assets verified. | [`MDV-041`](docs/plans/active/MDV-041-release-v0.10.0.md) |

The roadmap order and every other task are in [`docs/BACKLOG.md`](docs/BACKLOG.md).

## Human decisions / blockers

- Merges, tags, releases, and deployments need explicit owner authority per
  action. Pushing branches and opening PRs after verification is authorized for
  roadmap work.
- Visual work needs owner acceptance on device; batch it into one verify
  build per round.
- Phase-5 tasks (`MDV-032`–`MDV-035`) start with a design for the owner.
- GitHub Actions: the repo is public and uses standard runners only, which do
  not consume the owner's (exhausted) private-repo minutes. Keep it that way.
- Release dispatches default to `dry_run: true`; a real release needs
  `dry_run: false` and owner authority.

## Next safe actions

1. Finish `MDV-041`: bump to `0.10.0`, run the full checks, push, then tag
   `v0.10.0` with owner authority and watch the Release workflow.
2. After the release, the owner decides on the website deploy.
3. Pick the next task from the backlog (`MDV-042` goto offset is the smallest).

## Verification state

### Verified now

- `origin/main` was `d53f0a3` after the cross-block merge (owner authority,
  2026-10-17). That tree passed `cargo fmt --check`, `cargo check
  --all-targets` with no warnings, 536 library tests, and all integration
  suites in release mode.
- v0.9.0 release evidence (assets, signatures, notarization, site) is in the
  v0.9.0 content pack.

### Not verified / follow-up

- Cross-block selection was checked with headless screenshots, unit tests, and
  an owner trial build; there is no automated native-window test.
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
