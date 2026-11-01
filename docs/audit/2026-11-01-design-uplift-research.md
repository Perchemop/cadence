# Design Uplift Research — Free / Open-Source Resources

**Date:** 2026-11-01
**Scope:** Research only. No source code was modified.
**Author:** design-research subagent
**Benchmarks:** Microsoft Project, Primavera P6, Linear, Affinity, Figma-class chrome.

## 0. Headline constraint — read this first

Cadence pins **egui 0.35 / eframe 0.35** (`[workspace.dependencies]` in the root
`Cargo.toml`). The upstream ecosystem has already moved to **egui 0.37.0**
(egui + eframe released 2026-10-15). Every crate below therefore needs the
**last release that still depends on `egui ^0.35`**, which is usually two or
three minor versions behind the crate's "latest". Picking the latest crate
version will not compile — it will want egui 0.36/0.37.

Two further traps found during verification:

- Several popular crates have **no 0.35-compatible release at all** right now
  (`egui_notify`, `egui-modal`, `egui_grid`, `catppuccin-egui`) — details below.
- All version numbers were pulled live from the crates.io HTTP API
  (`https://crates.io/api/v1/...`) and each "0.35-compatible" claim was checked
  against that version's actual `egui` dependency requirement. Licences are the
  SPDX string crates.io publishes for that exact version.

Recommendation for the medium term: **plan an egui 0.35 → 0.37 bump** as its own
slice. It unlocks the current widget ecosystem and is the single highest-leverage
maintenance move; the research below still holds after the bump (versions move
forward, licences do not change).

---

## (a) Ranked recommendation table

Impact = visual/behavioural payoff for a dense scheduler desktop app.
Effort = integration cost in the existing `crates/ui-egui` structure.

| Rank | Item | What it buys | crate@version (egui 0.35) | License | Impact | Effort |
|---|---|---|---|---|---|---|
| 1 | **egui_extras** | Virtualised `TableBuilder` (sticky header/footer, striped rows, resizable columns, `DatePicker`, `Image`, syntect highlighting) — the backbone of a Project-class task/Resource grid | `egui_extras@0.35.0` | MIT OR Apache-2.0 | High | Low |
| 2 | **egui-phosphor** | 9,000+ outline/fill/duotone icon glyphs as a bundled font — professional iconography for toolbar, tree, inspector, Gantt bar affordances | `egui-phosphor@0.13.0` | MIT OR Apache-2.0 (icons MIT) | High | Low |
| 3 | **egui_dock** | Dockable / tabbed / resizable panels — the Linear/Figma-style splittable inspector + timeline + resource layout, with persisted layout state | `egui_dock@0.20.1` | MIT | High | Medium |
| 4 | **egui_table** | Lower-level virtualised table: fixed header **and** footer rows, per-column resizable/hide, cell-level custom paint — better than `egui_extras` when rows are drawn from a project model rather than a simple record list | `egui_table@0.9.0` | MIT OR Apache-2.0 | High | Medium |
| 5 | **egui-toast** | Non-blocking toast/notification stack (save, schedule computed, import finished) with severity + timeout | `egui-toast@0.21.0` | MIT | Medium | Low |
| 6 | **egui_dnd** | Drag-and-drop reordering for WBS/task lists and column sets, with drop indicators | `egui_dnd@0.16.0` | MIT | Medium | Low |
| 7 | **egui_tiles** | Tiling layout engine (drag/resize tiles) for a multi-pane schedule workspace; complements `egui_dock` for free-form arrangements | `egui_tiles@0.16.0` | MIT OR Apache-2.0 | Medium | Medium |
| 8 | **egui-file-dialog** | Fully themed in-app file open/save dialog, so dialogs match Cadence chrome on every OS (`rfd` stays for native shell integration only) | `egui-file-dialog@0.14.1` | MIT | Medium | Low |
| 9 | **egui_ltreeview** | Virtualised hierarchical tree with multi-select and drag — ideal for the WBS / activity hierarchy | `egui_ltreeview@0.8.0` | MIT | Medium | Low |
| 10 | **egui_plot** | Immediate-mode plots — S-curves, resource histograms, cash-flow / float distributions | `egui_plot@0.36.0` | MIT OR Apache-2.0 | Medium | Low |
| 11 | **egui_commonmark** | Renders Markdown — project notes, changelog, in-app help | `egui_commonmark@0.24.0` | MIT OR Apache-2.0 | Low | Low |
| 12 | **egui-snarl** | Node graph canvas — dependency / network-logic visualisation, what-if scenario graphs | `egui-snarl@0.11.0` | MIT OR Apache-2.0 | Low | Medium |
| 13 | **egui_virtual_list** | Variable-height virtual list — long activity/resource lists without jank | `egui_virtual_list@0.11.0` | MIT | Medium | Low |
| 14 | **egui_flex** | Flexbox-like `Flex`/`FlexItem` layout — predictable toolbars, inspector rows, wrapping button bars | `egui_flex@0.7.0` | MIT | Medium | Low |
| 15 | **egui_taffy** | Grid/Flexbox/Block layout via the `taffy` engine — heavier alternative when a real CSS-like grid is needed | `egui_taffy@0.13.0` | MIT | Low | Medium |
| 16 | **egui-data-table** (`egui_data_table`) | Higher-level editable data grid (row selection, editing) built on `egui_table` | `egui_data_table@0.11.0` | **non-standard (flag: review licence text before use)** | Medium | Low |
| — | *rfd (already used)* | Native OS file dialogs; keep for "Open recent"/OS integration | `rfd@0.17.2` | MIT | — | in place |

### Do NOT use at egui 0.35 (verified incompatible or unmaintained)

| Crate | Latest | Why not | Status |
|---|---|---|---|
| `egui_notify` | 0.24.0 requires egui ^0.37; last 0.35-era release is 0.22.0 (**egui ^0.34** — also incompatible) | No egui 0.35 release exists. Use **egui-toast@0.21.0** instead. | Skip |
| `egui-modal` (`egui_modal`) | 0.6.0 requires egui **^0.30** only | Stale since 2024-12; nothing newer. Build modals with `egui::Modal` (built into egui ≥0.30) instead. | Skip |
| `egui_grid` | 0.5.1 requires egui **^0.27.2** | Effectively abandoned (2024-05). Replace with `egui_table` / `egui_flex`. | Skip |
| `catppuccin-egui` | 5.7.0 supports egui **^0.26–^0.33** | No 0.35 support. Lift the Catppuccin **palette values** (MIT) into `design.rs` tokens manually if desired. | Skip |
| `egui-theme` | 0.2.0 | Abandoned 2022 (egui ~0.18). Serialisable theme container that never kept pace. | Skip |
| `egui-icon-typeist` / `egui-icontypist` | — | **Does not exist on crates.io (HTTP 404).** | Skip |
| `egui_dropdown`, `egui-component`, `egui_code_editor` | — | No egui 0.35 release (0.34 / 0.37 only) or out of scope. | Skip |

---

## 2. Icon systems

egui renders text via fonts and images via textures — it has no SVG pipeline, so
the practical choice is a **bundled icon font mapped to a private-use unicode
range**, or hand-embedded SVGs converted to textures.

| Approach | Licence | Verdict |
|---|---|---|
| **`egui-phosphor@0.13.0`** — Phosphor as a ready-made egui font (`egui_phosphor::add_to_fonts`, `egui_phosphor::regular::*` / `bold::*`) | MIT OR Apache-2.0 (crate); icons MIT | **Recommended primary.** Zero-asset-pipeline, 6 weights, covers schedule/data/toolbar concepts, permissive, no attribution required. |
| **Tabler Icons** (6,100+ outline, 24 px grid, `-filled` variants) | MIT | Best second choice for **data-dense admin/dashboard** idioms; would need your own font-build step (embed as font) — more work than Phosphor. |
| **Lucide** (1,700+, 2 px stroke, currentColor) | ISC | Clean Linear-class outline look. ISC ≈ MIT. Requires an embed/convert step; no maintained egui crate. |
| **Material Symbols** (variable, 4,000+) | Apache-2.0 (attribution recommended, not required) | Viable; largest set; but the "Google look" is recognisable and less neutral for a P6/Project benchmark. |
| **Fluent UI System Icons** (2,900+, 4 styles) | MIT | Good if you want Windows 11-native feel; no egui crate — needs embedding. |
| **`egui-icontypist`** | — | **Does not exist on crates.io.** Disregard. |

**Recommendation:** ship `egui-phosphor@0.13.0` now (permissive, no pipeline),
keep Lucide/Tabler SVGs on the roadmap only if a bespoke look is later required.
Record the icon font in the same font-loading path as Inter so glyph fallback
is deterministic.

---

## 3. Theme systems — token-driven approach

Cadence already owns the right pattern: a typed `Palette` + `palette(dark)` in
`crates/ui-egui/src/design.rs`, applied to `egui::Visuals`. **Do not adopt an
external theme crate** — the viable ones either lag the egui version
(`catppuccin-egui` ≤0.33, `egui-theme` dead) or impose a structure that fights
the existing token file.

Recommended shape (extends what exists, no new dependency):

1. **Two-tier tokens**: primitive (raw colour/space/radius ramps) →
   semantic (`surface`, `surface-raised`, `border-hairline`, `text-muted`,
   `critical`, `warning`, `ok`, `accent`, `grid-line`). Keep the semantic names
   Cadence already uses; add surface/focus/overlay roles.
2. **Roles to add for a "paid chrome" feel**: elevation levels (surface
   containers 0/1/2), a dedicated **focus-ring** token, a **state-layer** alpha
   (hover 6 %, pressed 10 %), and separate **grid** vs **border** hairlines.
3. **Spacing/radius as tokens**: 4 px rhythm already noted in `design.rs`; add
   radius ramp (4 / 6 / 8) and control heights (24 / 28 / 32) for density.
4. **Derive everything, never hard-code**: map tokens into `egui::Visuals` +
   `Style` (widgets, selection, window shadow, panel fill, stroke widths).
5. If a named palette is wanted, **copy Catppuccin's colour values by hand**
   (`catppuccin-egui` values are MIT) into the primitive tier — do not take the
   crate dependency.

Licence notes: egui itself is MIT OR Apache-2.0; Catppuccin palette values MIT;
`egui-theme` MIT but abandoned. No paid service is required by any of this.

---

## 4. Typography

Cadence already bundles **Inter** (Regular/Medium/SemiBold/Bold) — a correct,
free, SIL OFL 1.1 choice and the same family Linear-class apps use. Recommended
changes are about **scale and numeric rendering**, not new fonts.

**Type scale for a dense desktop scheduler** (base 13 px, Linear-like density,
tighter than Material's 16 px base):

| Token | Size / line-height | Use |
|---|---|---|
| `caption` | 11 / 15 | Units, hints, axis labels, footnote chrome |
| `body-sm` | 12 / 16 | Grid cells, inspector values, secondary text |
| `body` | 13 / 18 | Default UI text, menus, buttons |
| `label` | 13 / 18 (Medium) | Form labels, column headers |
| `subtitle` | 15 / 20 | Panel titles, dialog titles |
| `title` | 18 / 24 | View headings |
| `display` | 24 / 30 | Empty-state / onboarding hero only |

**Professional details that read as "paid":**

- **Tabular figures** in every numeric column and the Gantt axis — enable Inter's
  `tnum` feature (open-type feature) so dates/durations/currency columns align.
- Consistent **Medium for headers, Regular for body**; avoid mixing sizes within
  a row.
- **Numeric right-alignment** in the task/resource grid; left-align text.
- Reuse the same weights the fonts already ship — no synthetic bold.
- Optional free complements (all permissive) if a second voice is wanted later:
  **JetBrains Mono** (SIL OFL) for IDs/durations, **IBM Plex Sans** (SIL OFL)
  if Carbon-style enterprise tone is preferred. Neither is required.

Font licences: Inter SIL OFL 1.1; JetBrains Mono SIL OFL 1.1; IBM Plex SIL OFL
1.1. All free for commercial bundling with the OFL notice.

---

## 5. Design systems & platform HIGs — borrow this, not that

### Material 3 — code Apache-2.0, docs CC BY 4.0
- **Borrow:** semantic **colour roles** (surface containers, on-surface, outline)
  and the surface-tint **elevation model**; state layers for hover/press; clear
  "leading/trailing" component anatomy.
- **Not that:** touch-first 48 dp targets, ripples, large FABs, 16 px body —
  all wrong for mouse/keyboard desktop density.

### IBM Carbon — Apache-2.0 (code), docs CC BY 4.0
- **Borrow:** the **2 px-based spacing scale** and 16-column layout; the
  **data-table** playbook (sticky header, batch-action bar, inline edit,
  overflow row menu, empty/loading/error states); "productive vs expressive"
  token split — Cadence wants **productive**.
- **Not that:** forcing IBM Plex everywhere; Carbon's heavy left navigation and
  large guidance panels.

### Atlassian Design System — **proprietary, restrictive licence**
- **Borrow (concepts only):** the **token naming taxonomy**
  (`color.text.accent.*`, `elevation.surface.*`, `color.border.*`) as inspiration
  for your semantic token names; focus-ring behaviour.
- **Not that:** the Atlassian **licence forbids modification, derivative works,
  and redistribution** of the ADS. Do **not** copy their tokens, code, or assets.
  Read for naming ideas, design your own values.

### Microsoft Fluent 2 / Windows 11 — Fluent UI MIT
- **Borrow:** the **menu bar + command bar** structure and grouping; subtle
  Mica/Acrylic-inspired layered surfaces; **revealed focus** (focus ring only on
  keyboard); crisp 1 px separators rather than borders on everything.
- **Not that:** full Fluent motion/gimmick set; oversized controls.

### Apple HIG — **proprietary docs (read-only)**
- **Borrow:** the **inspector pane** layout (contextual properties on the right),
  **sidebar source list** grouping, toolbar **item grouping + overflow**,
  disciplined control sizing and alignment.
- **Not that:** iOS-scale tap targets and consumer-soft styling; HIG assets are
  not licensed for reuse — study structure only.

### GNOME HIG — docs **CC BY-SA** (attribution + share-alike)
- **Borrow:** keyboard-first navigation, consolidated **header bar / menu**
  pattern (fewer chrome rows), preference hierarchy.
- **Not that:** client-side-decoration title bars; if any HIG text is quoted,
  satisfy CC BY-SA attribution — safest to absorb the ideas, not the text.

**Scheduler-specific synthesis:**
- **Toolbar / menu bar** ← Fluent (grouping, overflow) + GNOME (consolidation).
- **Inspector / property panel** ← Apple HIG (right-hand contextual pane) + M3
  roles for surface containers.
- **Data grid** ← Carbon data-table playbook (the closest match to P6/Project
  task sheets).
- **Gantt** ← borrow nothing outright; draw it in-house on `egui` with token
  colours, tabular figures, and the elevation/hairline discipline from Carbon +
  M3. `egui_plot` is not a Gantt engine — keep the custom canvas.

---

## 6. Open-source PM / scheduling UIs worth studying

All of these are **reference material only**. Several are copyleft — study the
visual grammar, never copy code, tokens, or assets.

| Project | Licence | What Cadence can learn visually |
|---|---|---|
| **ProjectLibre** | **GPLv3 (copyleft — flag)** | Direct Gantt-bar interaction, resource sheet density, the classic Project/P6 "sheet + chart" split. |
| **GanttProject** | **GPLv3 (copyleft — flag)** | Clean desktop Gantt + task inspector relationship; compact toolbar. |
| **OpenProject (Community)** | **GPLv3 (copyleft — flag)** | Work-package table ⇄ Gantt ⇄ timeline switching; baseline/variance presentation. |
| **Redmine (+ Gantt plugins)** | **GPLv2 (copyleft — flag)** | Web Gantt with issue links; filter/sidebar layout. |
| **Vikunja** | **AGPLv3 (copyleft — flag)** | Modern list/Kanban/Gantt toggle in one shell; contemporary density and colour use. |
| **Focalboard** | **MIT** (permissive; repo **archived/unmaintained**) | Board/Table/Gallery view switching and card property panels — safe to study, but do not depend on it. |
| **Kanboard** | MIT | Swimlane/board organisation for work streams. |

**Take-away:** Cadence's current tense, hairline-based, token-driven design is
already ahead of the copyleft desktop tools above. Borrow their **information
architecture** (sheet + chart marriage, inspector coupling, view switching), and
borrow **visual polish** only from the permissively licensed modern references
(Focalboard's view switching, Linear-class density).

---

## 7. Summary of licensing posture

- **Permissive, safe to depend on:** `egui_extras`, `egui-phosphor`, `egui_dock`,
  `egui_table`, `egui-toast`, `egui_dnd`, `egui_tiles`, `egui-file-dialog`,
  `egui_ltreeview`, `egui_plot`, `egui_commonmark`, `egui-snarl`,
  `egui_virtual_list`, `egui_flex`, `egui_taffy`, `rfd` — all MIT and/or
  Apache-2.0. Icon fonts: Phosphor MIT, Tabler MIT, Lucide ISC, Material Symbols
  Apache-2.0. Fonts: Inter / JetBrains Mono / IBM Plex all SIL OFL 1.1.
- **Review before use:** `egui_data_table@0.11.0` — crates.io lists
  **"non-standard"** licence; read the repository text first.
- **Copyleft — visual reference only, never copy code:** ProjectLibre, GanttProject,
  OpenProject (GPLv3); Redmine (GPLv2); Vikunja (AGPLv3).
- **Restrictive / proprietary — do not copy tokens or assets:** Atlassian Design
  System (licence forbids derivatives), Apple HIG (read-only docs), GNOME HIG
  (CC BY-SA — attribution/share-alike if text is quoted).
- **Paid services required:** none.

---

*Method note: every crate version and licence above was read from the live
crates.io HTTP API on 2026-11-01, and each "egui 0.35 compatible" version was
confirmed by inspecting that version's declared `egui` dependency requirement.
No files outside `docs/audit/` were touched.*
