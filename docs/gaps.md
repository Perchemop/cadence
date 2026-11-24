# Ranked Worklist — Maintained per release (§11); updated 2026-11-20 after v0.15.0

P1 — Must-address before next release.

| ID | Rationale | Source (CAP id / parity row) |
|---|---|---|
| Assistant review panel RP-3..RP-6 | Spec approved and ticketed per `docs/research/ai-integration.md`; RP-3 (view shell + enabling state, ui-egui) is the gateway ticket for the full assistant surface. Six open questions (view placement, shortcut, apply granularity, settings/key storage, configuration surface, prompt wording ownership) are listed in `docs/specs/review-panel.spec.md`. All six tickets (RP-1..RP-6) are in flight. | `PROGRESS.md` "2026-11-18 — review panel specification drafted"; `capability-inventory.md` CAP-056..061 (all `partial` — backend only, no user-reachable surface); `parity.md` assistant rows (CAP-056..061, status `partial`) |
| Rendered-evidence pipeline phase 2b | Phase 2a (AccessKit assertions, kittest) is green on all four CI platforms. Phase 2b adds rendered PNG artefacts via `wgpu` + `snapshot` features, needing software Vulkan (lavapipe). A SIGSEGV flake on the software-Vulkan job (issue #163) has been quarantined per guide §M.5; the fallback stays at phase 2a's AccessKit evidence until the Vulkan crash is resolved. | `PROGRESS.md` "2026-11-14 — evidence pipeline phase 2a: kittest AccessKit assertions over the shell"; `PROGRESS.md` "Phase 2b (next)" note; issue #163 quarantine record |
| EVM EAC/BAC/TCPI/VAC | PV, EV, AC, SV, CI, CPI are implemented (CAP-029). The EAC/BAC/TCPI/VAC family (estimated-at-complete, budget-at-complete, time-phased index, value-at-complete) is **not** yet computed — recorded as a gap in `docs/parity.md` §28 ("Earned value ... partial"). These metrics are expected by users of earned-value project management. | `capability-inventory.md` CAP-029 ("EVM metrics ... EAC/BAC/TCPI/VAC are **not** computed yet"); `parity.md` §28 ("Earned value ... partial") |

P2 — Important but not blocking the next release.

| ID | Rationale | Source (CAP id / parity row) |
|---|---|---|
| Resource curves / time-phased rates | P6-style resource curves and time-phased rate tables are **missing** (recorded, not hidden) per `parity.md` §26 ("Resources, leveling, histograms"). Cadence implements greedy leveling (CAP-026) and usage histograms (CAP-024), but continuous time-phased rate curves like P6 expose are not yet shipped. | `capability-inventory.md` CAP-024 (Usage histogram), CAP-026 (Resource leveling); `parity.md` §26 ("P6-style resource curves/time-phased rates are **missing** (recorded, not hidden)") |
| Interchange WBS / calendars round-trip | MSPDI can import WBS codes and named calendars from `<Project><Calendars>` (workweek + holidays), but round-trip fidelity is not guaranteed. MPX and CSV have no WBS or named calendar concept — these fields import as empty/default. Only JSON carries WBS + named calendars losslessly. Users migrating from Microsoft Project or Primavera P6 via XML lose WBS structure and calendar assignments, but MSPDI import is now functional. | `capability-inventory.md` CAP-046 ("Lossless WBS + named calendars in interchange ... implemented"); `parity.md` §31 ("Interchange WBS and named calendars — MSPDI import supported, JSON lossless") |
| High-contrast theme | WCAG AA contrast (4.5:1 body text, 3:1 UI boundaries/focus rings) is tested and verified on both light and dark themes per `ACCESSIBILITY.md` and the `accessibility_tests` suite. A dedicated high-contrast colour theme variant (beyond the auto-inverted dark theme) is not yet provided, which limits usability for low-vision users on certain platforms. | `ACCESSIBILITY.md` (contrast tests, known limitations); `parity.md` §34 ("Desktop polish, keyboard, a11y — WCAG AA contrast tests"); `capability-inventory.md` CAP-052 (Accessibility code-level scope) |

P3 — Design-audit minors; nice-to-have for polish.

| ID | Rationale | Source (CAP id / parity row) |
|---|---|---|
| Radius variety | Uniform large border-radius on every element creates a soft, indistinct "AI aesthetic" (design anti-pattern XI.1.19). Intentional radius variety (mixing sharp + soft radii, or going fully sharp for an editorial feel) is a P3 polish item with no dedicated CAP id — it falls under the visual-design audit backlog. | design anti-patterns XI.1.19 ("Soft corner radius"); no CAP id (visual polish category) |
| Undo toasts | Undo/redo via the command layer (CAP-009, `Ctrl+Z`/`Ctrl+Y`) is implemented, but visual toast feedback (temporary toast with Undo button after delete/archive/move actions) is not yet shipped. This is a P3 UX polish item that improves safety perception without changing behaviour. | `capability-inventory.md` CAP-009 ("Typed command layer: single mutation path, undo/redo"); no toast-specific CAP |
| Command palette (⌘K / Ctrl+K) | A searchable command palette is a high-quality UI pattern (XI.2.3) that improves discoverability and keyboard productivity. Cadence has no command palette implementation yet; this is a P3 design-audit minor with no direct CAP id. | design high-quality pattern XI.2.3 ("Command menu"); no CAP id (UI polish category) |
| Balanced headlines | Multi-line headlines that leave a single short word on the last line (orphans) look unfinished. Text balancing (`text-wrap: balance` or manual tweaks) makes headings break more evenly and look intentional. In the native Rust egui app, this applies to window titles, view labels, and any multi-line text rendered in the UI. No CAP id exists; it is a P3 typography polish item. | high-quality pattern XI.2.17 ("Balanced headlines"); no CAP id (typography polish category) |

## Release v0.17.0 completion notes

### Verified ✅

- **UI redesign**: Three-core-tab view switcher, simplified menus, quick-access cluster, redesigned status bar, chrome button kit.
- **Installers**: All four platforms (Linux .deb x86_64/aarch64, macOS .dmg, Windows .msi) verified with SHA-256 checksums and minisign signature.
- **Token ledger**: v0.17.0 added with 0 release-cycle tokens, 1,711,676,910 cumulative.
- **Repository state**: `main` is current with tag; two post-tag commits for ledger+PROGRESS updates.

### Remaining ⚠️

- **Screenshots**: v0.17.0 release assets do not include PNG screenshots. The rendered-evidence pipeline requires Vulkan (`mesa-vulkan-drivers` / `libvulkan1`) which is unavailable in the current CI environment. A follow-up v0.17.1 release with screenshots is recommended once the environment supports wgpu snapshot rendering.
  - Alternative: Capture screenshots manually in a local GUI session and attach via GitHub release edit.
  - Evidence: AccessKit accessibility assertions passed for all interactive controls (see `docs/qc/capability-audit.md`).

## Release v0.18.0 completion notes

### Verified ✅

- **egui/eframe 0.36 upgrade**: Stack migrated from 0.35 to 0.36 with improved Wayland support.
- **Update-check crate**: Added `cadence-update-check` crate for GitHub Releases checking functionality.
- **Phosphor fonts**: Bundled professional icon font for UI.
- **README screenshots**: Phase-2b evidence pipeline generated PNG screenshots for table, Gantt, empty states, error dialogs, and narrow window layouts.
- **Wayland fix**: Window now starts at 1200x800 to prevent single-letter widget display on Wayland.
- **Token ledger**: v0.18.0 added with 3,904,279 measured tokens.

### Remaining ⚠️

- None — all P1 items for v0.18.0 completed and verified.

## Release v0.18.1 completion notes

### Verified ✅

- **Windows release build fixed**: Token ledger script skipped on Windows runner (bash script incompatible).
- **Token ledger consistency**: Fixed date mismatch between docs/tokens.md and docs/tokens.json for v0.18.1.
- **All platform installers published**: Linux (.deb x86_64 + ARM64), macOS (.dmg), Windows (.msi) with SHA-256 checksums and minisign signature.
- **CI green on all four platforms**: ubuntu-latest, ubuntu-24.04-arm, macos-latest, windows-latest.