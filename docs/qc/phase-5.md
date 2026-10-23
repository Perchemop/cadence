# Phase 5 Quality Gate

Checklist for the Phase-5 polish and interop slices. Phase-1..4 gates live in
`phase-1.md`..`phase-3.md`.

## Functionality

- [x] Open interchange, all four formats (P5-1): Microsoft Project XML (MSPDI),
      MPX, CSV activity tables and Cadence JSON, import and export, with
      in-crate XML and JSON parsers and no new dependencies.
- [x] Markdown project reports (P5-2a): schedule table with dates, float and
      criticality; per-resource committed and over-allocated days;
      earned-value metrics against the most recent baseline; baseline list.
      `cadence report <file> [out]`; printable via any browser.
- [x] Keyboard shortcuts (P5-3, partial): Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y
      undo-redo and Ctrl+1..4 view switching from any view.
- [x] Accessibility (P5-3, partial): WCAG AA (4.5:1) contrast verified by
      tests for every colour the app draws text in, on both themes
      (`accessibility_tests` in `cadence-ui-egui`), including white-on-accent
      for selections and 3:1 non-text contrast for the accent itself.
- [x] Performance gates (P5-3, partial): 10,000-activity schedule under 500 ms
      (existing), 10,000 committed resource days analysed under 750 ms, and
      100 parallel activities leveled under 3 s — all asserted in CI.
- [x] Documentation and examples (P5-4, partial): `docs/usage.md` command,
      format, keyboard and report reference; `bridge_report` example.
- [x] Property tests over the new surfaces: leveling invariants on arbitrary
      projects, and round-trip properties for all four interchange formats
      within each format's documented scope. These found and closed two real
      defects (MSPDI percent truncation, undocumented CSV resource
      renumbering).
- [x] The GUI writes the Markdown report through a **Save report** button.

## Engineering

- [x] `cargo fmt --check --all` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes
      (this includes the example, which is an `--all-targets` target).
- [x] `cargo test --workspace` passes (197 tests).
- [x] CI green on `opencode/ci` (run 39898422581), pending merge to `main`.

## Deliberately deferred (user decisions, not engineering gaps)

- Native print/PDF from the GUI (P5-2b): egui has no print pipeline and the
  project's dependency-light stance argues against a PDF writer for a feature
  browsers already provide. The Markdown report is the printable artifact.
- Full keyboard-only operation audit and screen-reader (AccessKit) pass
  (P5-3): enabling AccessKit adds its dependency tree and needs interactive
  verification with a screen reader, which CI cannot do.
- First tagged release (P5-4): a release decision — what to include, version
  number, and merging PR #1 to `main` first.
