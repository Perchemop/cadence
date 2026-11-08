# Elite paid-tool UI uplift — focused proposals

Date: 2026-11-08 · Owner: UI/UX (ui-dev) · Clean-room: public behaviour +
permissive assets + URLs only. Companion: `docs/design/brand.md`,
`docs/design/elite-ui-ux-uplift.md`, `docs/audit/2026-11-07-full-visual-functional-audit.md`,
`docs/audit/2026-11-08-elite-tool-uplift-plan.md`, `docs/ui-patterns.md`.

This document scopes six concrete proposals that most directly remove the
"engineer-built" read of Cadence (audit §2): **iconography, date picker,
widget layer, release notes, crash dialog, theme persistence**. Each carries
clean-room references (public summaries + URLs only) and egui 0.35
implementation steps. Prioritised by "engineer-built → paid-tool" impact per
effort (audit §2.3 sequencing).

---

## Priority order (impact × effort, eliminating engineer-built look first)

| # | Proposal | Audit ref | Why it kills the demo look | Effort |
|---|---|---|---|---|
| 1 | **Iconography** (Phosphor) | V1/V16 | Text-only 21-button toolbar is the #1 "prototype" cue | M |
| 2 | **Widget layer** (egui_extras + egui-toast) | V16 | Hand-rolled tables/date fields read as scaffolding | M |
| 3 | **Date picker** (egui_extras DatePicker) | V12 | Raw `YYYY-MM-DD` text is the most visible "not a real app" affordance | S–M |
| 4 | **Theme persistence** | V4/F18 | Forgetting the chosen theme each launch contradicts release notes | S |
| 5 | **Crash dialog** (copy/URL/after-open) | F16 | Misleading empty state + giant URL + premature log clear = untrusted | S |
| 6 | **Release notes** (spacing/metrics/zoom claim) | V3/F18 | Stale metrics + phantom "zoom slider" claim erode credibility | S |

Items 1–3 are the visible "paid tool" cues; 4–6 are trust/credibility fixes
that make the visible cues believable. Do 1–2 first, then 3, then 4–6 in any
order (they are independent).

---

## 1. Iconography — `egui-phosphor` (MIT)

**Goal.** Replace the text-only toolbar/tab/menu affordances with a coherent
icon set so the command bar reads like a professional scheduler, while keeping
text as the accessible name (F19 AccessKit, brand voice).

**Clean-room references (public summaries + URLs only).**
- Phosphor Icons — open-source icon family, MIT licence, six weights:
  https://phosphoricons.com/
- Crate: https://crates.io/crates/egui-phosphor (0.13.0, egui 0.35 compatible)
  — `egui_phosphor::add_to_fonts(ctx)` registers the icon font alongside the
  app fonts; glyphs exposed as `egui_phosphor::regular::*` / `bold::*`.
- Concept references (behaviour only, no code): Fluent 2 command-bar grouping
  https://fluent2.microsoft.design/ ; GNOME HIG header-bar consolidation
  https://developer.gnome.org/hig/. No proprietary icons copied.

**egui implementation steps.**
1. Add `egui-phosphor = "0.13"` to `crates/ui-egui/Cargo.toml` (version pinned
   to the egui 0.35-compatible release; verify in CI — guide §12.1; do not
   bump egui itself in this slice).
2. In `design.rs::install_fonts`, call `egui_phosphor::add_to_fonts(ctx)` in
   the same font-loading path as Inter, so fallback is deterministic and the
   icon face is registered once at startup.
3. Introduce a small `icons.rs` (or a `icon(label, glyph)` helper) mapping
   semantic actions → glyph constants: New/Open/Save, Undo/Redo, view tabs
   (Gantt/Task/Resource/…), inspector fields, Gantt affordances (milestone,
   constraint, critical). Keep the mapping in one place so it is the single
   source of truth (mirrors the `Command::label()` ask in
   `docs/design/elite-ui-ux-uplift.md` §2).
4. Render toolbar/menu items as `Button::new(RichText::new(glyph).size(14.0))`
   with `.on_hover_text("Open (Ctrl+O)")` so the text meaning is never lost
   (F19, brand voice). Use `ui.add_sized` for consistent 24 px hit targets
   (control-height token from `elite-ui-ux-uplift.md` §1).
5. Where an icon alone is ambiguous (e.g. "Set status date"), keep a text
   label beside the glyph; icons are cues, not replacements (brand.md voice).
6. Graceful fallback: if the pinned crate version is unavailable, compile with a
   `#[cfg]` text-only path; never block the build on the dependency
   (audit uplift plan §3.3).
7. Record the dependency in `ATTRIBUTION.md` (MIT — no attribution required,
   but the suite standard records third-party assets).

**Domain gaps.** None. Pure UI; consumes existing commands/actions.

---

## 2. Widget layer — `egui_extras` + `egui-toast`

**Goal.** Stop hand-rolling tables, date fields and toast notifications so the
surface matches the reference class (MS Project / P6) and stops reading as
scaffolding (audit V16).

**Clean-room references (public summaries + URLs only).**
- `egui_extras` — official egui companion crate, MIT/Apache-2.0:
  https://docs.rs/egui_extras/ — provides `TableBuilder`, `DatePicker`,
  `StripBuilder`, image loaders.
- `egui-toast` — toast notification widget, MIT:
  https://crates.io/crates/egui-toast
- Concept references: IBM Carbon data-table playbook (naming/behaviour only)
  https://carbondesignsystem.com/components/data-table/usage/ ; Fluent 2
  https://fluent2.microsoft.design/. No code copied.

**egui implementation steps.**
1. Add `egui-extras = { version = "0.35", features = ["datepicker", "table"] }`
   and `egui-toast = "0.13"` (0.35-compatible) to `Cargo.toml`. Verify
   versions in CI; do not migrate egui in this slice.
2. Replace the hand-rolled Resources/Baselines/Calendars/Diary grids with
   `TableBuilder` (sortable headers, striped rows, resizable columns, fixed
   header) — this also fixes V10/V11 (monospace + header row + duplicate cell)
   by configuring per-column `.clip(true)` and a header row.
3. Route all status messages through `egui-toast`'s `Toasts` instead of the
   ad-hoc status string, so success/warn/critical toasts are consistent and
   auto-dismiss (replaces the single status-bar line for transient feedback).
4. Keep the existing token palette: configure `TableBuilder` row heights and
   `egui-toast` colours from `design::Palette` so the widget layer inherits the
   brand tokens rather than egui defaults.
5. `egui_extras::StripBuilder` for the three-zone shell layout where fixed
   columns are wanted (toolbar clusters), replacing manual `ui.horizontal`.

**Domain gaps.** None for the widget layer itself. `TableBuilder` callbacks
must call into existing commands; no new domain types.

---

## 3. Date picker — `egui_extras::DatePicker`

**Goal.** Replace every raw `YYYY-MM-DD` text field with a calendar picker so
date entry matches the reference class and removes the most visible
"not a real app" affordance (audit V12, F13).

**Clean-room references (public summaries + URLs only).**
- `egui_extras::DatePicker` documentation:
  https://docs.rs/egui_extras/latest/egui_extras/struct.DatePicker.html
- Behavioural reference: MS Project / P6 date entry via picker (public help
  pages; `docs/ui-patterns.md` "Data pickers — raw text dates — gap"). No
  proprietary widget copied.

**egui implementation steps.**
1. Ensure the `datepicker` feature is enabled on `egui-extras` (step 2.1).
2. Add a small `date_picker_field(ui, &mut date_string) -> bool` helper that
   parses the current `YYYY-MM-DD` string into a `chrono`/`time` date (the
   crate already depends on `time`), renders `DatePicker::new(...)`, and
   writes back the canonical string only when the user picks a date. Keep the
   text field beside it for keyboard entry (accessibility).
3. Replace call sites: activity start/finish, baseline dates, calendar
   exceptions, diary dates, production reschedule date, lookahead dates, status
   date (audit V12 list). Do this in one slice to avoid mixed affordances.
4. Validation: on invalid text, show the existing inline error (F4/F11
   pattern) and disable the picker's commit; never silently coerce.
5. Keyboard: Enter commits the typed date; Esc cancels the picker (ties into
   F13 dialog keyboard behaviour). Gate global shortcuts while the picker is
   open (F13).
6. Persist the chosen date only via the existing `SetDate`/command path; no
   new domain mutation in UI.

**Domain gaps.** None new; `SetDate`/activity date commands already exist.
Optional: a domain helper `parse_date`/`format_date` canonicalisation if not
already present — check `cadence-core` date utilities; raise if missing.

---

## 4. Theme persistence (V4/F18)

**Goal.** Remember the chosen theme (and recent files) across restarts so the
release notes' claim ("remembers the ten most recent projects") becomes true
and the app stops forgetting user preference (audit V4).

**Clean-room references (public summaries + URLs only).**
- Platform config-dir conventions: XDG Base Directory (Linux)
  https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html ;
  Windows `%APPDATA%` / macOS `~/Library/Application Support` (standard OS
  paths, no proprietary code).
- Serde JSON persistence (already a dependency) for `UiState`.

**egui implementation steps.**
1. Define `UiState { theme: Theme, recent_files: Vec<PathBuf>, first_run: bool,
   … }` with `#[derive(Serialize, Deserialize)]` and `#[serde(default)]` on
   every field (matches the domain persistence pattern in
   `crates/lintel-core`/Cadence core).
2. Resolve the config path: `dirs` crate if available, otherwise
   `std::env::var("HOME"/"APPDATA")` + `/.config/cadence/ui.json` (reuse the
   crash-report path logic; no new heavy dep). Store beside the project file as
   a v0.11 fallback if no config dir is writable.
3. Load `UiState` once at app construction (before first frame) and apply the
   theme via `design::apply_theme(ctx, mode)` immediately.
4. Save on change (theme toggle, file open) and on clean exit; debounce writes
   (e.g. save at most once per second) to avoid disk churn.
5. Apply theme live: `ctx.set_visuals` + style overrides already in
   `design::apply_theme`; ensure both light/dark palettes are WCAG-AA verified
   (brand.md: accent AA on both themes; keep the accessibility tests).
6. Recent files: cap at 10, dedupe, drop missing paths, render in File ▸ Recent
   with the existing menu (audit F18 — make the claim true).
7. Migration: if the old in-memory-only state exists, ignore; `#[serde(default)]`
   keeps old files loadable.

**Domain gaps.** None. UI state is presentation state (acceptable in the UI
crate per the Lintel investigation §2 note); do not push into the domain model.

---

## 5. Crash dialog (F16)

**Goal.** Make the crash dialog trustworthy: honest empty-state copy, a
bounded issue URL, and clearing the log only after the browser is confirmed
opened (audit F16).

**Clean-room references (public summaries + URLs only).**
- GitHub issue URL conventions and query-length limits (public GitHub docs):
  https://docs.github.com/en/issues/tracking-your-work-with-issues/creating-an-issue
- `rfd`/`open` crate for launching the browser (already a dependency).
- Rust `url`/`percent-encoding` crates for safe encoding (standard ecosystem).

**egui implementation steps.**
1. Empty state: replace the misleading "no crash" text with a plain sentence
   per brand voice — e.g. "No crash report has been recorded." (states are
   sentences, never exclamations; brand.md voice).
2. Bound the issue URL: cap the embedded log to N KB (e.g. 4 KB) and truncate
   with a "… (truncated; full log saved to <path>)" note; percent-encode with
   `percent-encoding`, not a hand-rolled encoder (crash.rs:82-88). If the
   encoded URL exceeds a safe length (~2000 chars for browsers), fall back to
   attaching the log file path and instructing the user to paste it.
3. Clear-after-open: only clear the crash log when the browser launch returns
   success; on failure, keep the log and show a toast with the saved path and a
   "Copy path" button. Never clear before confirmation (F16).
4. Add Esc to close and Enter to confirm (F13/V13) and gate global shortcuts
   while the dialog is open.
5. Add an AccessKit label (`labelled_by`) for the dialog and its buttons (F19).
6. Keep the scrubbing at write time (audit §6) — do not add new PII to the
   report.

**Domain gaps.** None; crash-report is a UI-adjacent crate. Optional: a
`crash_report::MAX_URL_BYTES` constant in `crates/crash-report` to keep the
limit in one place — raise with the tooling owner.

---

## 6. Release notes (V3/F18)

**Goal.** Make the release-notes popup accurate and credible: fix the ~30-space
run, drop stale hard-coded token metrics, remove the phantom "zoom slider"
claim, and drive metrics from `docs/tokens.json` (audit V3/F18, suite standard
§12.1).

**Clean-room references (public summaries + URLs only).**
- Suite standard: `docs/tokens.json` ledger (`total_tokens_since_start`,
  `release_tokens`, `release_version`) referenced in release notes
  (audit §7, guide §12.1).
- egui `Window`/`ScrollArea` for the popup layout (official docs
  https://docs.rs/egui/).

**egui implementation steps.**
1. Remove the ~30-space run (`lib.rs:4886`) and replace with a proper
   `ui.separator()` or a `Grid`/`ScrollArea` layout; never use spaces for
   alignment.
2. Remove hard-coded token metrics ("release cycle (0.12.0): ~45K") and the
   "zoom slider in the status bar" claim unless that feature actually ships;
   claims must match shipped behaviour (F15/F18).
3. Drive token metrics from `docs/tokens.json` at build time via
   `include_str!("../../../docs/tokens.json")` parsed with serde, or expose
   them through a build script; render `release_tokens` and
   `total_tokens_since_start` with the release version. Keep the ledger as the
   single source of truth (suite standard).
4. Structure the popup: version header (bottom-right link opens it per
   §12.1), per-release notes list, token metrics block, and an "Update" button
   that opens the release page (label honestly — "Open release page" until
   signed-download verification lands, per F15).
5. Add Esc to close, Enter not needed (no primary action); gate global
   shortcuts while open (F13).
6. Add AccessKit labels (F19) and keep the version subtly in the bottom-right
   corner (guide §12.1, audit V3).

**Domain gaps.** None. Token ledger is a docs/CI concern; the UI reads it.
Optional CI step to update `docs/tokens.json` on semver tags (audit §7) —
raise with the tooling owner.

---

## Cross-cutting implementation notes

- **No heavy toolchain locally.** All changes verified via
  `~/.config/opencode/scripts/ci.sh run "..."` in GitHub Actions (guide
  CI-first policy). Never run cargo locally.
- **Borrow discipline.** egui closures capture `&mut self` fields; split borrows
  (pass `&mut` fields individually) rather than `RefCell` (AGENTS.md rule).
- **Undo rule.** These six proposals add no undo-relevant mutations; theme/
  recent-files/preference writes are presentation state and must not touch the
  undo stack.
- **Clean-room.** All references above are public summaries + URLs; no
  proprietary code, icons, strings, layouts or algorithms are copied. Record
  every new dependency/asset in `ATTRIBUTION.md`.

## Domain gaps consolidated (for domain-dev)

- `cadence-core` date utilities: confirm a canonical `parse_date`/`format_date`
  exists for the DatePicker helper (proposal 3); raise if missing.
- `crates/crash-report`: optional `MAX_URL_BYTES` constant (proposal 5).
- Token ledger CI update on semver tags (proposal 6) — tooling owner.
- None of the six proposals require new domain commands or schema changes.

## What I did not change

No code, no `Cargo.toml`, no tests. This is a design document only, per the
design mandate. Implementation slices can be picked up by ui-dev against the
priority table above.