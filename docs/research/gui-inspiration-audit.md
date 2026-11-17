# GUI Inspiration Audit — Cadence vs Industry Standards

**Date**: 2026-11-17  
**Scope**: Public research on scheduling/project management GUIs for Cadence desktop app overhaul  
**Method**: Public sources only — official docs, help pages, public videos, marketing screenshots, demos  
**Constraint**: Clean-room — no decompilation, no proprietary code/assets copied

---

## Products Researched

| Product | Category | Key Public Sources |
|---------|----------|-------------------|
| Microsoft Project (Desktop) | Industry standard | MS Support docs, Learn.microsoft.com |
| Oracle Primavera P6 | Enterprise construction | Oracle Cloud help, Tensix walkthroughs |
| nPlan Schedule Studio | AI-assisted scheduling | nPlan public demos, blog |
| ALICE Technologies | Generative optimization | ALICE public webinars, docs |
| Planera | Collaborative scheduling | Planera public demo, blog |
| Microsoft Project for Web | Cloud scheduling | MS Learn, public demo |
| Smartsheet | Grid + Gantt hybrid | Smartsheet public demo, help |
| Airtable | Flexible grid views | Airtable public templates, help |
| Linear | Polished issue tracking | Linear blog, public demo |
| Notion | Flexible views | Notion templates, help center |

---

## Comparative Feature Matrix

| Feature | MS Project | Primavera P6 | nPlan | ALICE | Planera | MS Project Web | Cadence (Current) | Gap |
|---------|-----------|-------------|-------|-------|---------|---------------|-------------------|-----|
| **Gantt Chart** | | | | | | | | |
| Interactive bar drag | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| Dependency creation on bars | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | Add drag-to-link |
| Critical path highlight | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Partial | Enhance |
| Longest path (driving chain) | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | Add longest path |
| Zoom/pan smooth | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Basic | Improve |
| Baseline overlay | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| Progress fill on bars | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| **Table/Grid** | | | | | | | | |
| Inline editing | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | Add |
| Multi-select + range | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Partial | Complete |
| Column customization | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Basic | Add |
| Filtering/sorting | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Basic | Enhance |
| Column pinning | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Add |
| Row height density | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | Add |
| **Navigation** | | | | | | | | |
| Command palette (⌘K) | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | Add |
| Tabbed views | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| Sidebar navigation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| Keyboard shortcuts | Extensive | Extensive | Good | Good | Good | Good | Good | Document |
| **Resources** | | | | | | | | |
| Histogram view | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| Leveling visualization | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Partial | Enhance |
| Over-allocation indicators | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Basic | Enhance |
| **Themes/Accessibility** | | | | | | | | |
| Light/Dark | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| High contrast | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Planned | Add |
| Density controls | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | Add |
| **States** | | | | | | | | |
| Skeleton loaders | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | Add |
| Empty states | Basic | Basic | Excellent | Excellent | Excellent | Excellent | Basic | Rewrite |
| Error states | Dialog | Dialog | Toast+Inline | Toast+Inline | Toast+Inline | Toast+Inline | Status bar | Add toasts |
| Loading states | Spinner | Spinner | Skeleton | Skeleton | Skeleton | Skeleton | Spinner | Add skeletons |

---

## Top 10 Priority Improvements for Cadence

### 1. Gantt: Drag-to-Create Dependencies (Critical)
**Inspiration**: MS Project, P6, Planera, nPlan all support click-drag between bars
**Implementation**: 
- Click bar end → drag to another bar start → preview link → release to create FS link
- Shift+drag for SS/FF/SF types
- Visual preview during drag (dashed line, target highlight)
**Files**: `crates/ui-egui/src/lib.rs` Gantt rendering + interaction handling

### 2. Command Palette (⌘K) (Critical)
**Inspiration**: Linear, Planera, nPlan, MS Project Web all have ⌘K
**Implementation**:
- Single searchable palette for all actions (views, commands, navigation)
- Fuzzy search, recent items at top, keyboard shortcuts shown
- Replace F1 shortcuts dialog with unified palette
**Files**: New module in `crates/ui-egui/src/command_palette.rs`

### 3. Skeleton Loaders for Async Operations (High)
**Inspiration**: Linear, Planera, Airtable all use skeleton screens
**Implementation**:
- Update check: skeleton for status area
- Project load: skeleton for table/Gantt
- Report generation: skeleton for report preview
**Files**: Add `Skeleton` component in `design.rs`, use in `lib.rs` views

### 4. Inline Table Editing (High)
**Inspiration**: MS Project, Smartsheet, Airtable, Planera all support inline edit
**Implementation**:
- Click cell → inline editor (text, number, date picker, combo)
- Enter to commit, Esc to cancel, Tab/Shift+Tab to navigate
- Validation on commit, status message on error
**Files**: Table view in `lib.rs` — add `EditableCell` component

### 5. Command-to-Create Dependencies in Table (High)
**Inspiration**: MS Project predecessor/successor columns editable inline
**Implementation**:
- Predecessor/Successor columns with activity picker
- Type activity ID or name → autocomplete
- Validation: no self-links, no duplicates, no cycles
**Files**: Table view column definitions + inspector

### 6. Density/Compact Mode (Medium)
**Inspiration**: Linear, Notion, Airtable, MS Project Web all have density control
**Implementation**:
- Three densities: Comfortable (default), Compact, Condensed
- Affects: row height, padding, font size, spacing
- Persisted in user preferences
**Files**: `design.rs` spacing scale + `CadenceApp` preference field

### 7. High Contrast Theme (Medium)
**Inspiration**: MS Project, Linear, MS Project Web all have high contrast
**Implementation**: 
- Extend `ThemeVariant` with `HighContrastLight`/`HighContrastDark`
- WCAG AA verified for all semantic pairs
- System preference detection + manual override
**Files**: `design.rs` + `apply` function

### 8. Toast Notifications for Undo/Actions (Medium)
**Inspiration**: Linear, Planera, Airtable use toast for undo + status
**Implementation**:
- Replace status bar messages with toast stack (top-right)
- Undo action on destructive operations (5s timeout)
- Auto-dismiss info/success, persist warnings/errors
**Files**: New `Toast` component in `design.rs` + integration in `lib.rs`

### 9. Longest Path / Driving Chain Visualization (Medium)
**Inspiration**: P6 "longest path", nPlan "driving chain", ALICE "critical drivers"
**Implementation**:
- Distinct from critical path (float-based)
- Show driving chain to project finish in different style
- Toggle in Gantt toolbar
**Files**: Gantt rendering + schedule analysis

### 10. Polished Empty/Loading/Error States (Medium)
**Inspiration**: Linear, Planera, Airtable — empty states guide action
**Implementation**:
- Empty project: "Create your first activity" + CTA button
- Empty filtered table: "No activities match filter" + clear filter button
- Loading: skeleton for Gantt/table rows
- Error: inline + toast, never modal-only
**Files**: View components in `lib.rs` + `design.rs` empty state constants

---

## Design System Gaps to Close

### Color System
- Current: Single accent, semantic palette, variance colors
- Need: Full semantic scale (50-900) for each semantic role per II.30.6
- Action: Extend `Palette` with scale, add `ColorScale` helper

### Typography Scale
- Current: Inter, fixed sizes
- Need: Fluid scale (12-13-14-16-18-24-32) with `TextStyle` mapping
- Action: Extend `palette()` to return full `FontDefinitions`

### Spacing System
- Current: 4px base
- Need: 4-8-12-16-24-32-48 scale with named tokens
- Action: Add `Spacing` struct to `Tokens`

### Component Library
- Need: `Button` (Primary/Secondary/Ghost/Danger), `Input`, `Select`, `DatePicker`, `Toast`, `Skeleton`, `Card`, `Divider`, `Tooltip`, `Dropdown`, `Tabs`, `CommandPalette`
- Action: Build in `design.rs` as reusable `ui` functions

---

## Implementation Phases

### Phase 1 (Immediate — Week 1-2)
1. Fix crash #168 ✅ DONE
2. Fix CI flaky evidence ✅ DONE
3. Command palette (⌘K)
3. Gantt drag-to-link
4. Skeleton loaders

### Phase 2 (Week 3-4)
1. Inline table editing
2. Density/compact mode
4. High contrast theme
5. Toast system

### Phase 3 (Week 5-6)
1. Inline dependency editing in table
2. Toast/undo system
2. Longest path visualization
3. Polish empty/loading/error states

### Phase 4 (Ongoing)
1. Component library consolidation
2. Keyboard shortcut coverage audit
3. AccessKit integration verification
4. Performance profiling (10k+ activities)

---

## Sources (All Public)

1. Microsoft Project Critical Path: https://support.microsoft.com/en-us/project/show-the-critical-path-of-your-project-in-project
2. Microsoft Project Compare: https://support.microsoft.com/en-us/project/compare-two-versions-of-a-project
3. Primavera P6 Check Schedule: https://tensix.com/using-the-p6-check-schedule-tool
4. Primavera P6 Schedule Comparison: https://tensix.com/primavera-p6s-schedule-comparison-feature-explained
5. nPlan Schedule Studio: https://nplan.io/schedule-studio (public demo)
6. ALICE Technologies: https://alicetechnologies.com (public webinars)
7. Planera: https://planera.com (public demo)
8. Linear: https://linear.app (public demo, blog)
9. Smartsheet: https://www.smartsheet.com (public demo)
10. Airtable: https://airtable.com (templates, help)
11. Linear Blog: https://linear.app/blog
12. Notion Help: https://www.notion.so/help

---

## Next Steps

1. **Immediate**: Create feature branch `feat/gui-overhaul-phase1` with command palette + Gantt drag-to-link + skeletons
2. **Review**: UI/UX designer review of Phase 1 implementation against this audit
3. **Test**: Add headless tests for new components (command palette, skeleton, drag-to-link)
4. **CI**: Verify all 18 checks pass before merging

---

*This audit follows clean-room rules (§2.1-2.3): only public sources consulted, no proprietary code/assets copied, all analysis original.*