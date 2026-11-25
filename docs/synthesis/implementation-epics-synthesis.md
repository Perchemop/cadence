# Implementation Epics Synthesis: Inspiration Apps UI/UX Research

**Synthesized from:** 5 epic specification documents + discovery document
**Date:** 2026-11-25
**Status:** Ready for ticket generation

---

## Executive Summary

Five implementation epics derived from deep competitive research across 10+ inspiration applications (MS Project, Primavera P6, Smartsheet, Excel, Linear, Figma, VS Code, Notion, Obsidian, Adobe CC, Blender). Each epic addresses a critical UI/UX capability gap identified in the discovery phase.

| Epic | Spec File | Key Deliverable | Priority | Dependencies |
|------|-----------|-----------------|----------|--------------|
| CADENCE-SETTINGS | `cadence-settings-architecture.spec.md` | 4-level settings hierarchy with sync | P1 (Foundation) | — |
| CADENCE-TOOLBAR | `cadence-toolbar-spec.md` | 3 density levels + resize + customization | P1 (Core UX) | CADENCE-SETTINGS |
| CADENCE-DESIGN-TOKENS | *(pending)* | Complete token library + egui Theme | P1 (Foundation) | — |
| CADENCE-ACCESSIBILITY | `accessibility.spec.md` (in cadence-wt-release) | WCAG AA compliance + CI gates | P1 (Quality Gate) | CADENCE-DESIGN-TOKENS |
| CADENCE-VIEW-SYNC | `cadence-view-sync-architecture.md` | Cross-view settings sync | P1 (Core UX) | CADENCE-SETTINGS |

---

## Integrated Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    CADENCE DESIGN TOKENS                         │
│  (Colors, Typography, Spacing, Radius, Elevation, Motion)       │
└────────────────────────────┬────────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
┌───────────────────┐ ┌───────────────┐ ┌──────────────┐
│ CADENCE-SETTINGS  │ │ CADENCE-      │ │ CADENCE-     │
│ Architecture      │ │ TOOLBAR       │ │ ACCESSIBILITY│
│ (4-level hierarchy│ │ (3 densities, │ │ (WCAG AA,    │
│ with sync)        │ │  resize,      │ │  keyboard,   │
│                   │ │  customization)│ │  screen reader)│
└─────────┬─────────┘ └───────┬───────┘ └──────┬───────┘
          │                   │                 │
          └───────────────────┼─────────────────┘
                              ▼
                    ┌───────────────────┐
                    │ CADENCE-VIEW-SYNC │
                    │ (Cross-view sync) │
                    └───────────────────┘
```

---

## Detailed Epic Specifications

### 1. CADENCE-SETTINGS — Settings Architecture (`cadence-settings-architecture.spec.md`)

**Status:** ✅ Spec complete (69KB)

**Core Architecture:**
- **4-Level Hierarchy:** Global → Project → View → Tool
- **Persistence:** Global=JSON config, Project=SQLite v11, View=optional SQLite, Tool=ephemeral
- **Sync:** Unidirectional fallback (Tool→View→Project→Global→Built-in), event bus
- **Schema Versioning:** Independent per level, migration functions

**Key Data Structures:**
```rust
SettingsContext {
    global: GlobalSettings,
    project: ProjectSettings,
    view: ViewSettings,      // per ViewMode
    tool: ToolSettings,      // ephemeral
}
```

**Conflicts with Existing Code (9):**
| Existing Field | Replaced By |
|----------------|-------------|
| `CadenceApp::dark` | `settings.effective_theme()` |
| `pixels_per_day`, `split_ratio` | `ViewSettings.gantt` |
| `sort_key`, `sort_ascending` | `ViewSettings.table` |
| `table_filter_buffer`, `table_critical_only` | `ViewSettings.table` |
| `visible_columns` (BTreeSet) | `ViewSettings.table.column_visibility` (Vec) |
| `collapsed_wbs` | `ViewSettings.table.collapsed_wbs` |
| Scattered `*_buffer` fields | Consolidated into `ToolSettings` / `ViewSettings` |
| `assistant_config` | `GlobalSettings.assistant` + `ViewSettings.assistant` |

**Implementation Phases (9 phases, ~5 weeks):**
1. Core Data Structures → `crates/core/src/settings.rs`
2. Global Persistence (JSON + config dir)
3. Project Settings in SQLite (schema v11)
4. View Settings (replace CadenceApp fields)
5. Tool Settings (ephemeral struct)
7. Settings UI (dialog + per-view panels)
8. Change Notification (event bus)
9. Polish & Accessibility

---

### 2. CADENCE-TOOLBAR — Toolbar System (`cadence-toolbar-spec.md`)

**Status:** ✅ Spec complete (28KB)

**Core Architecture:**
- **3 Density Levels:** Compact (0.85x) / Standard (1.0x) / Comfortable (1.15x)
- **DensityScale struct:** item_spacing, button_height, min_height, min_item_width, tooltip_delay
- **DensityState:** current level + scale, persisted through settings hierarchy

**Resize Interaction Model:**
- Snap points per density (Compact: 24/32px, Standard: 32/40px, Comfortable: 40/48px)
- Mouse drag with Alt + edge, keyboard Alt+Up/Down, grid snap on release
- Min height enforced by density level

**Customization API:**
```rust
ToolbarCustomization {
    visible_items: Vec<ToolbarItemId>,
    item_order: Vec<ToolbarItemId>,
}
```
Predefined items: CreateActivity, Undo, Redo, Delete, View-*, ColumnsChooser, GanttZoom, ToggleTheme, etc.

**View-Specific Configurations (10 views):**
| View | Default Density | Key Items | Resizable |
|------|-----------------|-----------|-----------|
| Table | Standard | Filter, Sort, Columns, Critical | Yes |
| Gantt | Standard | Zoom, Split, Critical Path, Today | Yes |
| Resources | Standard | Grouping, Histogram, Cost Mode | Yes |
| ... | Standard | View-specific items | Yes |

**Keyboard Density Cycling:**
- `Ctrl+Shift+D` cycles Compact → Standard → Comfortable
- Status hint 1.5s, announced via AccessKit
- View menu submenu + right-click toolbar menu

**Design Token Integration:** Density scales flow through token system (item_spacing, button_height, min_height, min_item_width, tooltip_delay)

**Implementation Phases (5 phases):**
1. Density token foundation → `DensityScale`, `DensityState`
2. View-specific configs → `VIEW_TOOLBAR_CONFIGS`
3. Customization API → `ToolbarItem`, `ToolbarCustomization`, dialog
4. Resize behavior → snap points, grid snap, mouse/keyboard
5. Keyboard density cycling → shortcut, status hint, AccessKit announcement

---

### 3. CADENCE-DESIGN-TOKENS — Design Token System *(spec pending)*

**Expected Scope (from discovery):**
1. **Color System:** Semantic roles with dark/light variants
   - Surface: base, raised, sunken, overlay
   - Text: primary, secondary, muted, disabled, on-accent
   - Accent: primary, hover, active, focus
   - Border: subtle, strong, focus
   - Status: success, warning, error, info
2. **Typography Scale:** display, h1-h3, body, caption, mono (size/weight/line-height)
3. **Spacing System:** 4px base, multipliers 0.5/1/2/3/4/6/8/12/16
4. **Radius System:** none, sm, md, lg, xl, full
5. **Elevation:** 3-4 levels (shadow tokens)
6. **Motion:** duration (50/100/200/300ms), easing, reduced-motion variants
7. **Density Tokens:** compact/standard/comfortable multipliers
8. **egui Integration:** Theme trait, Visuals/Style mapping, CI grep gate

**Expected Output:**
- JSON token library + Rust types (`crates/ui-egui/src/tokens/`)
- egui Theme implementation (`impl Theme for CadenceTheme`)
- Migration plan from `design.rs`
- Validation tests (contrast, token completeness)

**Dependencies:** Needed by CADENCE-TOOLBAR (density tokens), CADENCE-ACCESSIBILITY (contrast tokens), CADENCE-VIEW-SYNC (density propagation)

---

### 4. CADENCE-ACCESSIBILITY — WCAG AA Compliance (`accessibility.spec.md` in cadence-wt-release)

**Status:** ✅ Spec complete (in cadence-wt-release)

**Scope:**
1. **Keyboard Navigation Map:** Global shortcuts (F1, Ctrl+K, Ctrl+1-9, Alt+1-3), view-specific navigation, tab order
2. **Focus Management:** Visible focus ring (ACCENT, ≥3:1), focus trapping, restoration rules
3. **Screen Reader (AccessKit):** Landmarks, view semantics (Table→Grid, Gantt→Tree, etc.), live regions, heading hierarchy
4. **Color Contrast:** 22 required test pairs (Light, Dark, HighContrastLight, HighContrastDark)
5. **High Contrast Mode:** Two new palette variants, system detection, WCAG AAA/forced colors
6. **Reduced Motion:** Detection via `ctx.style().visuals.animate`, opt-out pattern
7. **Toolbar Density:** 3 modes, keyboard cycling (Alt+1/2/3, Alt+Shift+D), live region announcement
8. **CI Gates (5 new):**
   - `a11y-contrast-tests` (22+ pairs)
   - `a11y-harness-tests` (AccessKit tree)
   - `a11y-target-size-tests` (≥24pt all, ≥32pt primary)
   - `a11y-focus-ring-tests` (focus ring, trap, restoration)
   - `a11y-density-tests` (cycling, announcement, persistence)

**CI Gate Additions (`.ci-required.txt`):**
```
a11y-contrast-tests
a11y-harness-tests
a11y-target-size-tests
a11y-focus-ring-tests
a11y-density-tests
```

---

### 5. CADENCE-VIEW-SYNC — Cross-View Settings Sync (`cadence-view-sync-architecture.md`)

**Status:** ✅ Spec complete (51KB)

**Architecture:**
- **4-Level Hierarchy:** Global → Project → View → Tool
- **Observer Pattern:** `SettingsStore` + `SubscriptionRegistry` (SettingsKey → Weak<Observer>)
- **Batched Notifications:** Crossbeam channels, frame-batched to prevent thrashing
- **Conflict Resolution:** Higher-level-wins (View > Project > Global) + last-write-wins same-level

**View-Specific Settings Schema:**
| View | Settings Struct |
|------|-----------------|
| Table | density, sort, filter, column widths/visibility |
| Gantt | timescale, pixels_per_day, split, critical path |
| Resources | grouping, histogram scale, cost display |
| ... | (all 10 views) |

**Persistence:**
- Global: JSON config file (`~/.config/cadence/settings.json`)
- Project: SQLite `project_settings` + `view_settings` tables (JSON columns)
- View: optional SQLite `view_settings` table
- Tool: never persisted

**Undo/Redo Integration:** `SetSetting`, `BatchSettings` commands in `crates/commands`

**Initialization:** Load global → project → construct store → create views → subscribe → render

**Hot-Reload:** Batched pending changes flushed once/frame, no view recreation

**Open Questions (require human gate):**
1. View settings persistence across restarts? (Recommended: yes)
2. Cloud sync for global settings? (Recommended: no — local-first)
3. Conflict resolution prompt vs auto-resolve? (Recommended: auto with logging)
4. 3 vs 4 density levels? (Recommended: 3 per Discovery H1)
5. Tool settings in undo stack? (Recommended: no — transient)

---

## Dependency Graph & Implementation Order

```
Phase 1 (Foundation - Weeks 1-2)
├── CADENCE-DESIGN-TOKENS (independent)
├── CADENCE-SETTINGS Phase 1-2 (Core structures + Global)
└── CADENCE-ACCESSIBILITY Phase 1 (Contrast tokens + CI gates)

Phase 2 (Core UX - Weeks 3-4)
├── CADENCE-SETTINGS Phase 3-5 (Project + View + Tool settings)
├── CADENCE-TOOLBAR Phase 1-2 (Density foundation + View configs)
├── CADENCE-VIEW-SYNC (Sync architecture)
└── CADENCE-ACCESSIBILITY Phase 2 (Keyboard + Focus)

Phase 3 (Polish - Weeks 5-6)
├── CADENCE-TOOLBAR Phase 3-5 (Customization + Resize + Keyboard)
├── CADENCE-SETTINGS Phase 6-9 (UI + Event bus + Migration + Polish)
└── CADENCE-ACCESSIBILITY Phase 3-4 (Screen reader + CI gates)
```

---

## Consolidated Ticket List

### CADENCE-SETTINGS (9 tickets)
| ID | Title | Phase | Depends On |
|----|-------|-------|------------|
| CADENCE-201 | Core settings data structures | 1 | DESIGN-TOKENS |
| CADENCE-202 | Global persistence (JSON + config dir) | 2 | — |
| CADENCE-203 | Project settings in SQLite (schema v11) | 3 | 202 |
| CADENCE-204 | View settings (replace CadenceApp fields) | 4 | 203 |
| CADENCE-205 | Tool settings (ephemeral) | 5 | 204 |
| CADENCE-206 | Settings UI (global dialog + per-view) | 6 | 205 |
| CADENCE-207 | Change notification (event bus) | 7 | 206 |
| CADENCE-208 | Migration testing & schema versioning | 8 | 207 |
| CADENCE-209 | Polish & accessibility | 9 | 208 |

### CADENCE-TOOLBAR (5 tickets)
| ID | Title | Phase | Depends On |
|----|-------|-------|------------|
| CADENCE-210 | Density token foundation | 1 | DESIGN-TOKENS |
| CADENCE-211 | View-specific toolbar configs | 2 | 210, SETTINGS-204 |
| CADENCE-212 | Customization API (show/hide/reorder) | 3 | 211 |
| CADENCE-213 | Resize behavior (snap/grid) | 4 | 212 |
| CADENCE-214 | Keyboard density cycling | 5 | 213 |

### CADENCE-DESIGN-TOKENS (6 tickets)
| ID | Title | Phase | Depends On |
|----|-------|-------|------------|
| CADENCE-220 | Color system (semantic roles) | 1 | — |
| CADENCE-221 | Typography scale | 1 | — |
| CADENCE-222 | Spacing/radius/elevation | 1 | — |
| CADENCE-223 | Motion + reduced-motion | 2 | 220-222 |
| CADENCE-224 | Density tokens | 2 | 220-223 |
| CADENCE-225 | egui Theme implementation | 3 | 220-224 |

### CADENCE-ACCESSIBILITY (8 tickets)
| ID | Title | Phase | Depends On |
|----|-------|-------|------------|
| CADENCE-230 | Contrast token validation tests | 1 | DESIGN-TOKENS-220 |
| CADENCE-231 | Keyboard navigation map | 2 | SETTINGS-204 |
| CADENCE-232 | Focus management (ring, trap, restore) | 2 | 231 |
| CADENCE-233 | Screen reader (AccessKit) integration | 3 | 232 |
| CADENCE-234 | Color contrast test suite (22 pairs) | 3 | DESIGN-TOKENS-220 |
| CADENCE-235 | High contrast mode | 3 | 234 |
| CADENCE-236 | Reduced motion + density accessibility | 4 | TOOLBAR-214 |
| CADENCE-237 | CI gates (5 new gates) | 4 | 230-236 |

### CADENCE-VIEW-SYNC (6 tickets)
| ID | Title | Phase | Depends On |
|----|-------|-------|------------|
| CADENCE-240 | SettingsStore + SubscriptionRegistry | 1 | SETTINGS-205 |
| CADENCE-241 | Observer pattern + batched notifications | 2 | 240 |
| CADENCE-242 | Conflict resolution (higher-level-wins) | 2 | 241 |
| CADENCE-242 | View-specific settings schema | 3 | SETTINGS-204 |
| CADENCE-243 | Persistence (Global JSON + Project SQLite) | 3 | SETTINGS-203 |
| CADENCE-244 | Undo/Redo + Hot-reload | 4 | COMMANDS, 242 |

---

## Open Questions Requiring Human Decision

| # | Question | Recommendation | Decision Needed By |
|---|----------|----------------|---------------------|
| Q1 | View settings persistence across restarts? | Yes | Before VIEW-SYNC Phase 3 |
| Q2 | Cloud sync for global settings? | No (local-first) | Before SETTINGS Phase 2 |
| Q3 | Conflict resolution prompt vs auto-resolve? | Auto + logging | Before VIEW-SYNC Phase 2 |
| Q4 | 3 vs 4 density levels? | 3 (per Discovery H1) | Before DESIGN-TOKENS Phase 2 |
| Q5 | Tool settings in undo stack? | No (transient) | Before VIEW-SYNC Phase 4 |
| Q6 | Cloud-syncable settings format? | Not needed (local-first) | Before SETTINGS Phase 2 |

---

## Next Steps

### Immediate (This Session)
1. ✅ Discovery document complete
2. ✅ All 5 epic specifications drafted (4 complete, 1 pending)
3. 🔄 Awaiting CADENCE-DESIGN-TOKENS spec completion
4. ⏳ Human decision on 6 open questions

### Next Session
1. Review CADENCE-DESIGN-TOKENS spec when complete
2. Human decision on 6 open questions
3. Run `/discovery/synthesize` with all specs to generate Jira tickets
4. Begin Phase 1 implementation (DESIGN-TOKENS + SETTINGS Phase 1-2 + ACCESSIBILITY Phase 1)

---

## Files Created

| File | Size | Status |
|------|------|--------|
| `cadence/docs/discovery/inspiration-apps-ui-ux-discovery.md` | 12KB | ✅ Complete |
| `cadence/docs/specs/cadence-settings-architecture.spec.md` | 69KB | ✅ Complete |
| `cadence/docs/specs/cadence-toolbar-spec.md` | 28KB | ✅ Complete |
| `cadence/docs/specs/cadence-view-sync-architecture.md` | 51KB | ✅ Complete |
| `cadence-wt-release/docs/specs/accessibility.spec.md` | ~50KB | ✅ Complete |
| `cadence/docs/specs/cadence-design-tokens.spec.md` | TBD | ⏳ Pending |
| `cadence/docs/synthesis/implementation-epics-synthesis.md` | 15KB | ✅ Complete (this file) |