# Elite paid-tool UI/UX uplift — investigation

Date: 2026-11-08 · Target: Cadence v0.12.1 (commit `db95ad4`) · Owner: UI/UX
investigation pass. Clean-room: only publicly documented behaviour, permissive
assets and URLs are referenced; no proprietary code, icons, strings or algorithms
are copied. See `docs/ui-patterns.md` (behaviour matrix) and
`docs/audit/2026-11-07-full-visual-functional-audit.md` (V#/F# findings).

This document does **not** supersede `brand.md` or `ui-polish-audit.md`; it
layers an "elite paid scheduler" (MS Project / Primavera P6 / Aphex class) lens
on top of them and ranks the next uplift work.

---

## 1. Visual design tokens and branding (`docs/design/brand.md`)

**Current state.** `design.rs` already implements the token palette (bg/panel/text/
text-muted/accent/critical/normal/ok/warn/grid + `deep`), 4/6 px radii, 4 px
spacing rhythm, Inter type scale. Branding is compliant (audit §3.1 "foundations
remain good"). Gaps are **missing semantic roles**, not wrong values.

**Recommendations.**
- Add surface-elevation tiers (`surface-0/1/2`) for nested panels, a dedicated
  `focus-ring` token (currently accent is reused for focus), a state-layer alpha
  (hover 6 %, pressed 10 %) and separate `grid` vs `border` hairlines (V9/F12).
- Add control-height tokens (24/28/32 px) and radius ramp (4/6/8) so dense vs.
  spacious surfaces are intentional, not ad-hoc (`ui-polish-audit` §10).
- Promote `text-strong` (13 px) and `caption` (11 px) as named styles in
  `install_style` rather than inline `FontId::proportional`.
- Persist theme choice (V4/F18) so the token set survives restarts.

**Clean-room implementation approach.** Extend `Palette` in `design.rs` with the
new roles and map them into `egui::Visuals` exactly as today; no new dependency.
Reference (concept only): Material 3 colour roles + surface-tint elevation
(https://m3.material.io/styles/color/roles), IBM Carbon data-table playbook
(https://carbondesignsystem.com/components/data-table/usage/), Atlassian token
*naming taxonomy* for role names only (ADS licence forbids copying values —
https://atlassian.design/components/tokens/).

**Estimated impact on "elite" feel:** Medium-high (chrome reads intentional).
**Priority:** P2 (after correctness P0/P1).

## 2. Iconography opportunities (SVG under `assets/`, MIT licensed)

**Current state.** Text-only by brand decision (`brand.md:71`, audit V1/V16).
Menus, tabs, toolbar and inspector are all text; ~21-button toolbar is wide.

**Recommendations.**
- Adopt `egui-phosphor@0.13.0` (MIT) as the primary icon font —
  `egui_phosphor::add_to_fonts` + `regular::*`/`bold::*` glyphs in the same
  font-loading path as Inter (`design.rs:install_fonts`), so fallback is
  deterministic. Zero asset pipeline (audit uplift research §2).
- Map icons to: Open/Save/New/Undo/Redo (toolbar), view tabs, inspector fields,
  Gantt bar affordances (milestone diamond, constraint indicator), status
  severity. Keep text labels as tooltips/aria names (F19 AccessKit).
- Optional second voice: Tabler (MIT) or Lucide (ISC) SVGs converted to a font
  only if a bespoke look is later required; Material Symbols (Apache-2.0) if a
  Google-native feel is wanted. Avoid Fluent UI System Icons unless Windows 11
  native feel is a goal.
- If SVG assets are wanted under `assets/icons/`, licence them MIT and record in
  `ATTRIBUTION.md` (brand.md:72). Do **not** reuse proprietary icon sets (MS
  Project/P6/Aphex trade dress).

**Clean-room.** Phosphor icons MIT, no attribution required; URLs:
https://phosphoricons.com/, crate https://crates.io/crates/egui-phosphor
(0.13.0, egui 0.35). SVG alternatives: Tabler https://tabler-icons.io/,
Lucide https://lucide.dev/, Material Symbols https://fonts.google.com/icons.

**Estimated impact:** High (single biggest "paid tool" cue, V1/V16).
**Priority:** P2 item 14 (after V9/V10/V5/V6/V7).

## 3. Typography (Inter font, token scale)

**Current state.** Inter (4 weights) bundled, SIL OFL 1.1; type scale 11/13/16/24
(`brand.md:50-53`, `design.rs:124-128`). Monospace rule not applied outside the
table (V10/F20). No tabular figures; numeric columns may jitter.

**Recommendations.**
- Enable Inter's `tnum` (tabular figures) OpenType feature for all numeric
  columns and the Gantt axis so digits align (uplift research §4).
- Adopt the denser scale from uplift research: caption 11/15, body-sm 12/16,
  body 13/18, label 13/18 Medium, subtitle 15/20, title 18/24, display 24/30 —
  base 13 px (Linear-class density) instead of 11 px body, which reads cramped
  on modern displays. Decide deliberately; keep monospace for numerics.
- Apply `FontFamily::Monospace` to dates/floats/counts in Resources, Baselines,
  Calendars, Diary and inspector grids (V10/F20).
- Right-align numeric columns, left-align text; use Medium for headers only.
- Optional free complements later: JetBrains Mono (SIL OFL) for IDs/durations,
  IBM Plex Sans (SIL OFL) for an enterprise tone. Neither required.

**Clean-room.** Inter https://rsms.me/inter/ (SIL OFL 1.1); JetBrains Mono
https://www.jetbrains.com/lp/mono/; IBM Plex https://www.ibm.com/plex/.
OpenType `tnum` via `egui::FontData` feature settings (egui 0.35 API).

**Estimated impact:** Medium-high (legibility + perceived quality).
**Priority:** P2 (V10 part of item 11).

## 4. Color palette application on canvas and grids (V9 from audit)

**Current state.** `Color32::YELLOW` drag-preview (`lib.rs:5339`), `GRAY`
arrows/ticks (`5396-5429, 5699-5714`), `WHITE` milestone outline + summary
end-caps (`5557-5574`) near-invisible on light panel; `variance_color` literals
(`5030-5038`). Violates brand.md ("colour reserved for meaning") and token
discipline (V9/F12).

**Recommendations.**
- Replace every hard-coded literal on the Gantt canvas with `Palette` tokens:
  drag-preview → `warn` (or a dedicated `drag-preview` token), arrows/ticks →
  `text-muted` or `grid`, milestone outline → `text` (theme-aware), summary
  end-caps → `text-muted`, variance → `critical`/`ok`/`warn` by sign.
- Add non-working (weekend/holiday) shading derived from the assigned calendar
  (V5, high impact) — the single most recognisable professional Gantt texture.
- Two-tier timescale (V6): month-over-week, quarter/year on zoom-out.
- Expand/collapse outline (V7) in table and chart.
- Striped rows already on (`v.striped = true`); keep token-driven.
- Add a critical-path legend (minor gap, ui-patterns "Critical path shown as
  red bars — adopted; no legend").

**Clean-room.** Derive shading from Cadence's own calendar model (domain layer),
not from any product's visuals. Behavioural reference: MS Project/P6 "non-working
time shading" (public help pages). No code copied.

**Estimated impact:** High (V9 is the most visible token-discipline failure).
**Priority:** P2 items 10 (V9), 12 (V5), 13 (V6/V7).

## 5. Status bar design (V2 from audit)

**Current state.** Single non-wrapping row (`lib.rs:2449-2540`): project range +
activity count + critical count + full EVM string + status message + save state +
zoom % + Shortcuts button. Clips at 1000 px min. Zoom figure "143%" has no
tooltip and disagrees with Gantt's own % label (V2/F14).

**Recommendations.**
- Convert to `horizontal_wrapped` (toolbar already wraps) and drop EVM from the
  bar at narrow widths (or move EVM into a tooltip/inspector).
- Tooltip the zoom figure: "Gantt zoom: 143%" and reconcile with Gantt's % label.
- Keep critical count, save state (`✓ saved`/`● unsaved`), zoom, F1 shortcuts
  control always visible; demote project range + activity count to a secondary
  cluster that wraps.
- Persist/clarify status-message severity colours (already `StatusSeverity`).
- Optional: add a progress/zoom slider only if it truly exists (V3 claims a
  "zoom slider in the status bar" that does not exist — fix the claim or build it).

**Clean-room.** Behavioural reference: MS Project footer status bar (counts,
status date, zoom) — `ui-patterns.md` row "Footer status bar — adopted".
Apple HIG toolbar item grouping/overflow for the wrap behaviour.

**Estimated impact:** Medium (clipping is a real usability bug, F14).
**Priority:** P2 item 16 / P1 polish.

## 6. Menu/toolbar design (three-zone shell already done, iconography opportunity)

**Current state.** Three-zone shell: File/Edit/View/Window/Help menus + tab
strip + tool cluster (`lib.rs:2144-2441`). All text. Menu items wide. V1.

**Recommendations.**
- Add Phosphor icons (area 2) to menu items and toolbar buttons; group with
  Fluent 2 command-bar structure (grouping + overflow) + GNOME consolidated
  header bar (uplift research §5).
- Shorten menu labels where safe; move rarely-used actions into overflow/Help.
- Add shortcut badges in tooltips (already partially done S2) and tab tooltips
  (S3-7, done).
- Make primary actions carry accent weight (currently no accent on primary
  buttons — `ui-polish-audit` directive 3 "Partial").
- `egui_flex@0.7.0` (MIT) for predictable wrapping button bars; `egui_dock`
  if dockable panels are desired later.

**Clean-room.** Fluent 2 menu/command bar (https://fluent2.microsoft.design/),
GNOME HIG header bar (https://developer.gnome.org/hig/). Concepts only.

**Estimated impact:** Medium-high (discoverability + density).
**Priority:** P2 (V1, V16) after icon adoption.

## 7. Empty states and accessibility

**Current state.** Empty states now plain sentences + shortcut per view (S6 done).
AccessKit `labelled_by` missing on most fields (F19, recorded "Done" but not —
audit §1). Screen-reader manual verification outstanding. Dialogs lack Esc/Enter
(F13/V13). Deletes without confirmation (F22). Non-token colours (V9).

**Recommendations.**
- Complete F19 `labelled_by` coverage: constraint, calendar, add-link, resource
  editor, baseline name, calendar editor, diary fields, production percent,
  reschedule date, assignment units (`lib.rs` refs in F19).
- Dialog keyboard behaviour (V13/F13): Esc cancels, Enter accepts primary,
  gate global shortcuts behind modals (`dialog_open` check before shortcut
  handling, `lib.rs:1933-1988`).
- Confirm all destructive deletes (F22): resource, baseline, diary note,
  production entry.
- Empty states: keep brand voice ("Nothing is scheduled in this window yet");
  add a single primary action button + shortcut; consider an illustration only
  if original (no stock art).
- Manual screen-reader verification (NVDA/VoiceOver/Orca) as a release gate;
  document in `ACCESSIBILITY.md`.
- Visual-evidence pipeline (V15): `egui_kittest` golden renders in CI + README
  screenshots (guide II.27.12, §12.1) — root cause of "auditing looks without
  seeing them".

**Clean-room.** WAI-ARIA authoring practices (https://www.w3.org/WAI/ARIA/apg/)
for dialog/empty-state patterns; egui AccessKit docs. No proprietary assets.

**Estimated impact:** High (a11y + correctness credibility; F1/F13 blockers).
**Priority:** P0/P1 for dialog & labelled_by; P4 for visual golden tests.

---

## Consolidated priority ranking (elite-tool feel)

| Priority | Items | Audit refs |
|---|---|---|
| P0 correctness | F1 open discards unsaved; F5 stale state on open; F2 units field; F3/F4/F10/F11 silent errors | F1-F5, F10-F11 |
| P1 dialog/feedback | F13 Esc/Enter + shortcut gating; F22 confirm deletes; F17 terminology; F15/F18 release-notes; F16 crash; V2 status bar; V13 | F13, F15, F16, F17, F18, F22, V2, V13 |
| P2 visual uplift | V9 token colours; V10 monospace grids; V11 header/duplicate cell; V5 non-working shading; V6 two-tier timescale; V7 outline; V1 icons (Phosphor); V12 DatePicker/tables/toasts; V2 wrap; V14 terminology | V1, V2, V5, V6, V7, V9, V10, V11, V12, V14, V16 |
| P3 depth/persistence | V4 persist theme+recent; F6/F7 cache; F8 show-in-Gantt; F9 decouple link form; progress/status-date line; constraints; drag-to-pan | V4, F6, F7, F8, F9 |
| P4 infrastructure | V15 golden renders + screenshots; egui 0.35→0.37 migration | V15, V16 |

## Clean-room compliance

- No proprietary code, UI, strings, icons or algorithms copied from MS Project,
  Primavera P6, Aphex, ProjectLibre, GanttProject, OpenProject, Redmine,
  Vikunja, Focalboard or Kanboard. All references are behavioural concepts
  restated in Cadence's own words (see `docs/ui-patterns.md`).
- Copyleft (GPL/AGPL) projects are **visual reference only**; never copy code.
- Atlassian Design System is **proprietary/restrictive** — borrow naming ideas
  only, never tokens or assets.
- All proposed dependencies are MIT and/or Apache-2.0 (or ISC for Lucide);
  icon fonts Phosphor (MIT), fonts Inter/JetBrains Mono/IBM Plex (SIL OFL 1.1).
  `egui_data_table` licence is non-standard — review before use.
- `egui 0.35` pin means use 0.35-compatible crate versions (research §0); plan
  the 0.35→0.37 bump as its own slice.

## Domain gaps noticed (for domain-dev)

- Non-working shading (V5) requires the calendar model to expose working/non-
  working days per date; verify `cadence-core` calendar API supports a
  `is_working_day(date)` query (or add one) before the UI slice.
- Two-tier timescale (V6) and outline (V7) need WBS hierarchy + date-range
  helpers in the domain; check whether collapse state should be persisted in the
  project model vs. UI-only.
- Drag-to-move/resize bars (high gap in §5 benchmark) is a domain command
  (`RescheduleActivity`/`SetDates`) + hit-testing; not yet wired in UI.
- Progress line / status-date line on the Gantt needs earned-value/progress data
  already modelled (baselines exist) — confirm domain exposes a progress series.
- Constraint/deadline indicators need constraint fields surfaced in the model.
- Status bar EVM string should be driven by domain-computed metrics, not
  recomputed in the view (F6/F7 caching).
