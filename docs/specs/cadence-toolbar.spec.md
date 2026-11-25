# CADENCE-TOOLBAR Feature Specification

## Overview

**Epic**: CADENCE-TOOLBAR — Toolbar System with 3 Density Levels, Resize, Customization
**User Value**: Enable professional planners to optimize toolbar density for their workflow, customize visible actions, and resize toolbar areas
**Target Users**: Professional planners (primary), site supervisors, power users
**Status**: Draft for review

---

## Functional Requirements (EARS Format)

### FR-01: Density Levels
- **FR-01.1**: The system shall provide three density levels: Compact (0.875×), Standard (1.0×), Comfortable (1.15×).
- **FR-01.2**: The system shall provide a `DensityScale` struct containing scaling factors for: item spacing, button height, minimum height, minimum item width, tooltip delay.
- **FR-01.3**: The system shall provide a `DensityState` struct storing current level and scale, persisted through settings hierarchy (Global → Project → View → Toolbar).
- **FR-01.4**: The system shall apply density scaling to all toolbar elements: item spacing, button heights, minimum dimensions, tooltip delays.

### FR-02: Toolbar Height & Resize
- **FR-02.1**: The toolbar height shall snap to density-aware snap points: Compact (24/32px), Standard (32/40px), Comfortable (40/48px).
- **FR-02.2**: The user shall resize toolbar height via mouse drag on the toolbar bottom edge (with Alt modifier).
- **FR-02.3**: The user shall resize toolbar height via keyboard: Alt+Up/Down to cycle snap points.
- **FR-02.4**: On resize release, the toolbar shall snap to the nearest valid density-aware snap point.
- **FR-02.5**: Minimum toolbar height shall be enforced per density level.

### FR-03: Customization API
- **FR-03.1**: The system shall define a `ToolbarItem` enum with 20+ predefined items: CreateActivity, Undo, Redo, Delete, View-Table/Gantt/Resources/Baselines/Calendars/Lookahead/Diary/Production/Audit/Assistant, ColumnsChooser, GanttZoom, ToggleTheme, ToggleSplitGantt, PrintReport, ExportXML, ImportXML.
- **FR-03.2**: The system shall provide `ToolbarItemId` type for stable item identification.
- **FR-03.3**: The system shall provide `ToolbarCustomization` struct with `visible_items` (Vec<ToolbarItemId>) and `item_order` (Vec<ToolbarItemId>).
- **FR-03.4**: The system shall provide a "Customize Toolbar..." dialog for show/hide/reorder operations.
- **FR-03.5**: Per-view customization shall be stored in the settings hierarchy (Global → Project → View → Toolbar).

### FR-04: View-Specific Configurations
- **FR-04.1**: The system shall provide predefined toolbar configurations for all 10 view modes with appropriate default density, visible items, item order, minimum height, and resizability.
- **FR-04.2**: Table view: Standard density, Filter/Sort/Columns/Critical, resizable.
- **FR-04.3**: Gantt view: Standard density, Zoom/Split/Critical Path/Today, resizable.
- **FR-04.4**: Resources view: Standard density, Grouping/Histogram/Cost Mode, resizable.
- **FR-04.5**: Other views: Baselines, Calendars, Lookahead, Diary, Production, Audit, Assistant with appropriate defaults.

### FR-05: Keyboard Density Cycling
- **FR-05.1**: The system shall provide `Ctrl+Shift+D` to cycle density: Compact → Standard → Comfortable → Compact.
- **FR-05.2**: A status hint shall appear for 1.5s when density changes.
- **FR-05.3**: Density changes shall be announced via AccessKit for screen readers.
- **FR-05.4**: View menu submenu and right-click toolbar menu shall provide density selection.

### FR-06: Design Token Integration
- **FR-06.1**: Density scales shall flow through the design token system: `item_spacing`, `button_height`, `min_height`, `min_item_width`, `tooltip_delay` all derive from `DensityScale`.
- **FR-06.2**: CI grep gate shall enforce no raw `Color32::from_*` outside the `theme/` module (per II.30.6).

### FR-07: Accessibility
- **FR-07.1**: All toolbar buttons shall have AccessKit accessible names.
- **FR-07.2**: Keyboard focus shall be visible on all toolbar elements (≥3:1 contrast).
- **FR-07.3**: Density cycling shall be announced via AccessKit live region.
- **FR-07.4**: Minimum target size: 24pt for all items, 32pt for primary actions.

---

## Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | Toolbar render time | < 5ms |
| NFR-02 | Density change propagation | < 16ms (one frame) |
| NFR-03 | Resize snap latency | < 50ms |
| NFR-04 | Customization dialog open | < 100ms |
| NFR-04 | Density cycling | < 50ms |
| NFR-05 | Minimum target size | 24pt all, 32pt primary |
| NFR-06 | Focus ring contrast | ≥ 3:1 on all themes |

---

## Acceptance Criteria (Given/When/Then)

### AC-01: Density Levels
**Given** the application is running
**When** the user cycles density with `Ctrl+Shift+D`
**Then** the toolbar cycles through Compact → Standard → Comfortable → Compact
**And** the toolbar height snaps to appropriate density snap points
**And** a status hint appears for 1.5s
**And** the change is announced via AccessKit

### AC-02: Toolbar Resize
**Given** the toolbar is visible
**When** the user drags the toolbar bottom edge with Alt held
**Then** the toolbar height changes smoothly
**And** on release, the height snaps to the nearest valid density snap point

### AC-03: Keyboard Resize
**Given** the toolbar has focus
**When** the user presses Alt+Up/Down
**Then** the toolbar height cycles to the next/previous snap point

### AC-04: Toolbar Customization
**Given** the user opens "Customize Toolbar..." dialog
**When** the user shows/hides/reorders items
**Then** the toolbar updates immediately
**And** the customization persists per view via settings hierarchy

### AC-05: View-Specific Configuration
**Given** the user switches from Table to Gantt view
**Then** the toolbar shows Gantt-specific items (Zoom, Split, Critical Path, Today)
**And** the toolbar density matches the Gantt view's configured density

### AC-05: Density Persistence
**Given** the user sets toolbar density to Compact in Table view
**When** the user restarts the application
**Then** the Table view toolbar density is still Compact

---

## Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | Toolbar render time | < 5ms |
| NFR-02 | Density change propagation | < 16ms (one frame) |
| NFR-03 | Resize snap latency | < 50ms |
| NFR-04 | Customization dialog open | < 100ms |
| NFR-05 | Density cycling | < 50ms |
| NFR-06 | Minimum target size | 24pt all, 32pt primary |
| NFR-06 | Focus ring contrast | ≥ 3:1 on all themes |

---

## Error Handling

| Error Condition | Detection | Response |
|----------------|-----------|----------|
| Invalid density state | Settings load | Fall back to Standard, log warning |
| Customization dialog error | Dialog open fails | Show error toast, log error |
| Resize below minimum | Drag below minimum | Clamp to minimum, show visual feedback |
| Customization corruption | Settings load | Reset to view defaults, log warning |

---

## Implementation TODO Checklist

### Phase 1: Density Token Foundation (Week 1)
- [ ] Add `DensityScale` and `DensityState` to `design_tokens.rs`
- [ ] Implement `DensityScale` with presets for Compact/Standard/Comfortable
- [ ] Add `density_scale()` to `DesignTokens` trait
- [ ] Wire density scaling through `SpacingToken`, `TypographyToken`, `RadiusToken`
- [ ] Add unit tests for density scaling

### Phase 2: View-Specific Configurations (Week 1-2)
- [ ] Define `VIEW_TOOLBAR_CONFIGS` constant array for 10 views
- [ ] Implement `ToolbarConfig` struct with default density, items, order, min height
- [ ] Wire configurations into toolbar rendering
- [ ] Add view-specific density persistence

### Phase 3: Customization API (Week 2-3)
- [ ] Define `ToolbarItem` enum with 20+ items
- [ ] Implement `ToolbarItemId` type
- [ ] Implement `ToolbarCustomization` struct with visible/order vectors
- [ ] Build "Customize Toolbar..." dialog UI
- [ ] Wire customization to settings hierarchy

### Phase 4: Resize Behavior (Week 3-4)
- [ ] Implement mouse drag resize on toolbar bottom edge (Alt modifier)
- [ ] Implement keyboard resize (Alt+Up/Down)
- [ ] Add density-aware snap points (Compact: 24/32, Standard: 32/40, Comfortable: 40/48)
- [ ] Add grid snap on release
- [ ] Add minimum height enforcement per density

### Phase 5: Keyboard Density Cycling (Week 4)
- [ ] Add `Ctrl+Shift+D` shortcut handler
- [ ] Implement density cycling logic (Compact → Standard → Comfortable)
- [ ] Add 1.5s status hint
- [ ] Add AccessKit live region announcement
- [ ] Add View menu submenu and right-click toolbar menu

### Phase 6: Accessibility & Polish (Week 5)
- [ ] Add AccessKit labels to all toolbar buttons
- [ ] Implement focus ring visibility (≥3:1 contrast)
- [ ] Add AccessKit live region for density announcements
- [ ] Verify minimum target sizes (24pt/32pt)
- [ ] Add CI gates for accessibility

### Phase 6: Accessibility & Polish (Week 5)
- [ ] Add CI gates for toolbar accessibility
- [ ] Verify design token integration
- [ ] Verify density scaling works with all token categories
- [ ] Verify no raw colour literals in toolbar code

---

## Validation

Run CI verification:
```bash
ci.sh run "CADENCE-TOOLBAR Phase 1-5"
```

Expected: All tests pass, clippy clean, formatting clean, accessibility gates pass.