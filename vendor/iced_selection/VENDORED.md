# Vendored iced_selection

Upstream: <https://crates.io/crates/iced_selection> version 0.5.0 (MIT, see
`LICENSE`). Copied from the published crate unchanged in the first commit;
later commits add only what rmdv needs to select across reader blocks:

- `text::rich::State` is public, with methods to hit-test a point, read the
  text between two positions, clear the widget's own selection, and draw a
  selection range set from outside.
- `Rich::operate` hands its `State` to `Operation::custom`, so a parent
  widget can find every reader text widget and its bounds.

Keep changes small and listed here so a later upstream release can replace
this copy.
