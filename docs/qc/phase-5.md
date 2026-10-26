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
- [x] PDF printing (P5-2b): dependency-free PDF writer in
      `cadence-core/src/pdf.rs` (text with wrapping and pagination, image
      pages, ASCII-hex streams), `cadence print <file> <out.pdf>`, and a GUI
      **Print report…** button using a native save dialog. Dialog behaviour
      itself needs interactive verification (below).
- [x] Keyboard shortcuts (P5-3): Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y undo-redo,
      Ctrl+1..5 view switching, Up/Down table selection, Ctrl+Plus/Minus/0
      Gantt zoom, and the F1 **Keyboard shortcuts** window. Every view was
      reviewed for pointer-only paths; drag-to-link has an inspector form.
- [x] Accessibility (P5-3): AccessKit is enabled by eframe's default
      features, standard widgets expose their labels and roles, and the Gantt
      canvas carries a `WidgetInfo` summary with the Table view as the
      per-activity semantic alternative. WCAG AA (4.5:1) contrast is verified
      by tests for every colour the app draws text in, on both themes
      (`accessibility_tests` in `cadence-ui-egui`), including white-on-accent
      for selections and 3:1 non-text contrast for the accent itself. Limits
      are recorded in `ACCESSIBILITY.md`.
- [x] Performance gates (P5-3, partial): 10,000-activity schedule under 1 s,
      10,000 committed resource days analysed under 2 s, and 100 parallel
      activities leveled under 3 s — all asserted in CI on every platform.
      Budgets catch algorithmic regressions, not runner speed.
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

- Manual screen-reader passes on each release platform (P5-3): the code-level
  audit and AccessKit wiring are complete, but a human with NVDA on Windows,
  VoiceOver on macOS or Orca on Linux must verify each platform, which CI
  cannot do. The native save dialogs in **Print report…** and **Print Gantt…**
  are likewise interactive-only.
- First tagged release (P5-4): a release decision — what to include, version
  number, and merging PR #1 to `main` first (now done).
