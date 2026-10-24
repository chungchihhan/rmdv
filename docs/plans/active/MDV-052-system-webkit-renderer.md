# MDV-052 — System WebKit Mermaid renderer

## Outcome and authority

Use system WebKit on macOS to remove Node/headless Chrome runtime dependencies
while retaining official Mermaid.js and AWS/Logos artwork. The owner confirmed
the final transparency fix works and authorized moving this work from the
local experiment into the existing PR branch, `codex/mermaid-js-integration`,
and updating [PR #80](https://github.com/minchenlee/rmdv/pull/80).
Delete the obsolete local `codex/mermaid-webkit` branch after preserving its work.
This authorizes the push/PR update, not merging, release or publication.

## Current contract

- One lazy native helper/WKWebView serves all Mermaid requests and files,
  serially. Release after 60–75 seconds idle; cached images remain in Iced.
- Embed the compiled helper, compressed Mermaid.js, both icon packs and license
  notices. Extract lazily into a private temporary directory. Node/npm and
  Swift command-line tools are build dependencies only on this macOS path.
- Use a local allowlisted URL scheme and CSP; no HTTP listener, external asset
  requests or persistent browser profile. Keep source/SVG/raster/time bounds.
- Send each palette as structured Mermaid configuration. Explicit source
  styles and brand artwork keep their own colors. The diagram canvas is clear;
  the preview modal supplies the opaque theme background.
- On macOS 14+, capture with public transparent PDF output and rasterize
  directly into a cleared sRGB premultiplied RGBA context. Keep original SVG.
  Ordinary WebKit snapshots flatten the frame base color and are not used.
- Non-macOS retains the previous Node/Chrome worker. Older macOS reports that
  this transparent native path requires macOS 14+. Do not imply all platforms
  have the new dependency footprint.

## Evidence and acceptance

- Latest `cargo build --release --bin rmdv --offline` passed on macOS arm64,
  including the embedded Swift helper and asset packaging.
- Focused worker probe: 559×200 PNG, four corner alpha values all zero,
  64,927 transparent and 45,367 opaque pixels; original SVG retains CJK text
  and #ED7100 Lambda artwork. No broad suite or owner-window automation.
- The owner explicitly confirmed the final fix works on the native app.
- Runtime resource usage, full platform/fixture coverage, helper signing and
  notarization, installer integration and CI asset provisioning remain open.
  The helper and WebKit can launch system processes; no measured memory claim.

The superseded attempts and source-level diagnosis are retained in
[trial history](../../status-history/2026-10-24-webkit-transparency-trial.md).
