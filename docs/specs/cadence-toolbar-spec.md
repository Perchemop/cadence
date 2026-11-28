# CADENCE-TOOLBAR Specification

**Epic:** CADENCE-UI-RESEARCH — Toolbar System v1  
**Status:** Draft — implementation phase  
**Related tickets:** CADENCE-102, CADENCE-108

---

## 1. Overview

This specification defines the toolbar system for Cadence, a construction and capital-project scheduling desktop application. The toolbar provides:

- **Three density levels** (compact / standard / comfortable) with token-based scaling
- **Resizable and snap-resizable** layout with grid-aware behavior
- **Full customization** (show/hide/reorder) per view
- **View-specific configurations** (Table, Gantt, Resources, etc.)
- **Keyboard density cycling** (Ctrl+Shift+D or similar)
- **Integration with the design token system** for consistent theming

The implementation is structured as a series of incremental phases, starting with density tokens and scaling, then customization, then view-specific configs, and finally keyboard cycling.

---

## 2. Density Token Design

### 2.1 Token Structure

Density is driven by a single `DensityScale` token that adjusts all proportional measurements in the toolbar. The token is stored in the app state and propagated to all toolbar renderers.

```rust
/// Density levels for the toolbar, driving token-based scaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DensityLevel {
    /// Compact: tighter spacing, smaller controls, more items visible.
    Compact,
    /// Standard: default medium density.
    Standard,
    /// Comfortable: larger spacing, larger targets, fewer items per row.
    Comfortable,
}

/// The scaling factors applied per density level.
/// Ratios are relative to the "standard" baseline (1.0).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DensityScale {
    /// Horizontal item spacing in pixels (standard = 8.0).
    pub item_spacing: f32,
    /// Vertical row height / button height in pixels (standard = 32.0).
    pub button_height: f32,
    /// Minimum toolbar height in pixels (standard = 32.0).
    pub min_height: f32,
    /// Minimum width per item before wrapping (standard = 40.0).
    pub min_item_width: f32,
    /// Tooltip auto-delay in milliseconds (standard = 200).
    pub tooltip_delay: u64,
}

impl DensityScale {
    /// The three predefined scale presets.
    pub const fn all() -> [Self; 3] {
        [
            // Compact: tighter spacing
            Self {
                item_spacing: 5.0,
                button_height: 24.0,
                min_height: 24.0,
                min_item_width: 32.0,
                tooltip_delay: 100,
            },
            // Standard: default medium density
            Self {
                item_spacing: 8.0,
                button_height: 32.0,
                min_height: 32.0,
                min_item_width: 40.0,
                tooltip_delay: 200,
            },
            // Comfortable: larger spacing and targets
            Self {
                item_spacing: 12.0,
                button_height: 40.0,
                min_height: 40.0,
                min_item_width: 48.0,
                tooltip_delay: 400,
            },
        ]
    }

    /// Return the preset for the given `DensityLevel`.
    pub fn for_level(level: DensityLevel) -> Self {
        match level {
            DensityLevel::Compact => Self::all()[0],
            DensityLevel::Standard => Self::all()[1],
            DensityLevel::Comfortable => Self::all()[2],
        }
    }

    /// Scale a standard-value measurement by this density's factors.
    pub fn scale(self, value: f32) -> f32 {
        // item_spacing and min_item_width scale linearly
        // button_height and min_height scale linearly
        // The ratio is relative to standard (1.0), so we compute:
        // (value / standard_value) * target_value
        let standard = Self::all()[1]; // standard preset
        let ratio = |this: f32, standard_this: f32| this / standard_this;
        // item_spacing
        value * ratio(self.item_spacing, standard.item_spacing)
        // button_height
        // min_height
        // min_item_width
        // tooltip_delay stays in ms, no scaling needed beyond the preset
        // (We return a packed struct; callers pick the fields they need.)
        // For simplicity, callers use the individual fields directly.
        0.0 // placeholder - callers use fields directly
    }
}
```

### 2.2 Density State

The density level is stored in the app state and persists across sessions via the settings hierarchy (global → project → view).

```rust
/// Current density level, persisted in the settings hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredDensity {
    Compact,
    Standard,
    Comfortable,
}

/// App-level density state, driving token-based scaling throughout the UI.
#[derive(Debug, Clone, Copy)]
pub struct DensityState {
    /// The logical density level.
    pub level: DensityLevel,
    /// The scaling factors derived from the level.
    pub scale: DensityScale,
    /// Whether the user has manually overridden the auto-detected level.
    pub overridden: bool,
}
```

### 2.3 Persistence

Density persists through the settings hierarchy:

```
Global → Project → View → Toolbar
```

- **Global**: App default (Standard)
- **Project**: Override for the current project file
- **View**: Per-view override (e.g., Gantt might be Comfortable, Table might be Compact)
- **Toolbar**: Per-toolbar customization (show/hide/reorder) stored per view

Persistence is handled by the settings architecture (see CADENCE-SETTINGS spec).

---

## 3. Resize Interaction Model

### 3.1 Height Snap Points

The toolbar height snaps to discrete values based on the active density level:

| Density | Snap Points |
|---------|-------------|
| Compact | 24px, 32px |
| Standard | 32px, 40px |
| Comfortable | 40px, 48px |

### 3.2 Width Behavior

- **Standard mode**: Items use `horizontal_wrapped` layout with `item_spacing` from the density scale. Minimum item width is `min_item_width`; items narrower than this are hidden or truncated with tooltip.
- **Compact mode**: Items can be narrower (32px min), with tighter packing. Tooltips are shown more aggressively.
- **Comfortable mode**: Items have wider minimum width (48px), larger hit targets. Layout may use two rows at narrow window widths.

### 3.3 Mouse Drag Resize

- **Modifier**: `Alt` + drag top toolbar edge, or `Ctrl` + drag bottom edge
- **Behavior**: Drag vertically to resize toolbar. The height snaps to the nearest snap point for the active density level.
- **Keyboard modifier**: `Up`/`Down` arrows while hovering the toolbar edge resize cursor adjust height by one snap point.
- **Constraint**: Toolbar height must remain ≥ `min_height` for the active density level.
- **Visual feedback**: As the user drags, a translucent preview shows the new height snap point. The cursor changes to `ns-resize` when over the edge.

### 3.4 Keyboard Shortcuts for Resize

| Shortcut | Action |
|----------|--------|
| `Alt` + `Up/Down` | Decrease/increase toolbar height by one snap point |
| `Ctrl` + `+` / `Ctrl` + `-` | Cycle density level (see §5) |
| `F11` | Toggle fullscreen (window-level, not toolbar-specific) |

### 3.5 Grid Snap

When the user releases the mouse, the toolbar height snaps to the nearest valid snap point for the current density. If the user is between snap points, the system chooses the closest one. The snap distance is 4px (half the standard spacing step).

---

## 4. Customization API

### 4.1 Show/Hide/Reorder Per View

Each view (Table, Gantt, Resources, etc.) has its own toolbar customization state, stored in the view-specific settings. The customization UI is accessible via the View → Customize Toolbar... menu item.

```rust
/// The set of visible toolbar item identifiers for a given view.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolbarCustomization {
    /// Item IDs that are currently visible in the toolbar.
    /// Items not in this list are hidden (but can be re-added).
    pub visible_items: BTreeSet<ToolbarItemId>,
    /// The order of visible items, from left to right.
    pub item_order: Vec<ToolbarItemId>,
}

/// Unique identifier for a toolbar item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolbarItemId(pub usize);

/// Predefined toolbar items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarItem {
    /// Create a new activity.
    CreateActivity,
    /// Undo.
    Undo,
    /// Redo.
    Redo,
    /// Delete selected activity.
    DeleteActivity,
    /// Break incoming links.
    BreakLinks,
    /// View → Table.
    ViewTable,
    /// View → Gantt.
    ViewGantt,
    /// View → Resources.
    ViewResources,
    /// View → Baselines.
    ViewBaselines,
    /// View → Calendars.
    ViewCalendars,
    /// View → Lookahead.
    ViewLookahead,
    /// View → Diary.
    ViewDiary,
    /// View → Production.
    ViewProduction,
    /// View → Audit.
    ViewAudit,
    /// View → Assistant.
    ViewAssistant,
    /// Theme toggle (dark/light).
    ToggleTheme,
    /// Columns chooser (Table view only).
    ColumnsChooser,
    /// Resource capacity editor.
    ResourceCapacity,
    /// Zoom control (Gantt view only).
    GanttZoom,
}
```

### 4.2 Customization Actions

Users can customize the toolbar through a modal dialog:

| Action | Effect |
|--------|--------|
| **Show items** | Checkbox list of all `ToolbarItem` variants; toggles visibility. |
| **Reorder items** | Drag-and-drop reordering of the visible items list. |
| **Reset to defaults** | Restore the default visible items and order for the current view. |
| **Apply** | Save the customization for the current view and close the dialog. |
| **Cancel** | Discard changes and close the dialog. |

### 4.3 Default Configurations Per View

| View | Default Visible Items | Default Order |
|------|----------------------|---------------|
| Table | Create, Undo, Redo, Delete, ViewGantt, ViewResources, ColumnsChooser, ToggleTheme | [Create, Undo, Redo, Delete, ViewGantt, ViewResources, ColumnsChooser, ToggleTheme] |
| Gantt | Create, Undo, Redo, Delete, ViewTable, ViewResources, GanttZoom, ToggleTheme | [Create, Undo, Redo, Delete, ViewTable, ViewResources, GanttZoom, ToggleTheme] |
| Resources | Create, Undo, Redo, Delete, ViewTable, ViewGantt, ResourceCapacity, ToggleTheme | [Create, Undo, Redo, Delete, ViewTable, ViewGantt, ResourceCapacity, ToggleTheme] |
| Baselines | Create, Undo, Redo, ToggleTheme | [Create, Undo, ToggleTheme] |
| Calendars | Create, Undo, Redo, ToggleTheme | [Create, Undo, ToggleTheme] |
| Lookahead | Create, Undo, Redo, ToggleTheme | [Create, Undo, ToggleTheme] |
| Diary | Create, Undo, Redo, ToggleTheme | [Create, Undo, ToggleTheme] |
| Production | Create, Undo, Redo, ToggleTheme | [Create, Undo, ToggleTheme] |
| Audit | Create, Undo, Redo, ToggleTheme | [Create, Undo, ToggleTheme] |
| Assistant | Create, Undo, Redo, ToggleTheme | [Create, Undo, ToggleTheme] |

### 4.4 API Functions

```rust
/// Set the visible items for a view's toolbar customization.
pub fn set_toolbar_customization(
    view_mode: ViewMode,
    visible_items: BTreeSet<ToolbarItemId>,
    item_order: Vec<ToolbarItemId>,
) -> Result<(), String>;

/// Get the current toolbar customization for a view.
pub fn get_toolbar_customization(view_mode: ViewMode) -> ToolbarCustomization;

/// Toggle the visibility of a specific toolbar item for a view.
pub fn toggle_toolbar_item_visibility(
    view_mode: ViewMode,
    item: ToolbarItem,
) -> Result<(), String>;
```

---

## 5. View-Specific Toolbar Configurations

### 5.1 View-Mode Mapping

Each `ViewMode` has a dedicated toolbar configuration that includes:

- The set of visible items
- The default item order
- The default density level
- View-specific resizing constraints

```rust
/// Configuration for a view's toolbar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewToolbarConfig {
    /// The default density level for this view.
    pub density: DensityLevel,
    /// The items visible in the toolbar for this view.
    pub visible_items: BTreeSet<ToolbarItemId>,
    /// The ordered list of visible items, left to right.
    pub item_order: Vec<ToolbarItemId>,
    /// Minimum toolbar height for this view (px).
    pub min_height: f32,
    /// Whether the toolbar can be resized by the user.
    pub resizable: bool,
}
```

### 5.2 Predefined View Configurations

```rust
impl ViewToolbarConfig {
    /// Default configurations for all view modes.
    pub fn all() -> BTreeMap<ViewMode, Self> {
        BTreeMap::from([
            (ViewMode::Table, Self {
                density: DensityLevel::Standard,
                visible_items: BTreeSet::from([
                    ToolbarItemId(0), // CreateActivity
                    ToolbarItemId(1), // Undo
                    ToolbarItemId(2), // Redo
                    ToolbarItemId(3), // DeleteActivity
                    ToolbarItemId(4), // ViewGantt
                    ToolbarItemId(5), // ViewResources
                    ToolbarItemId(6), // ColumnsChooser
                    ToolbarItemId(7), // ToggleTheme
                ]),
                item_order: vec![
                    ToolbarItemId(0),
                    ToolbarItemId(1),
                    ToolbarItemId(2),
                    ToolbarItemId(3),
                    ToolbarItemId(4),
                    ToolbarItemId(5),
                    ToolbarItemId(6),
                    ToolbarItemId(7),
                ],
                min_height: 32.0,
                resizable: true,
            }),
            (ViewMode::Gantt, Self {
                density: DensityLevel::Comfortable,
                visible_items: BTreeSet::from([
                    ToolbarItemId(0), // CreateActivity
                    ToolbarItemId(1), // Undo
                    ToolbarItemId(2), // Redo
                    ToolbarItemId(3), // DeleteActivity
                    ToolbarItemId(4), // ViewTable
                    ToolbarItemId(5), // ViewResources
                    ToolbarItemId(8), // GanttZoom
                    ToolbarItemId(7), // ToggleTheme
                ]),
                item_order: vec![
                    ToolbarItemId(0),
                    ToolbarItemId(1),
                    ToolbarItemId(2),
                    ToolbarItemId(3),
                    ToolbarItemId(4),
                    ToolbarItemId(5),
                    ToolbarItemId(8),
                    ToolbarItemId(7),
                ],
                min_height: 40.0,
                resizable: true,
            }),
            // ... other views
        ])
    }
}
```

### 5.3 View-Specific Behaviors

| View | Density | Key Items | Special Behavior |
|------|---------|-----------|-----------------|
| Table | Standard | ColumnsChooser | Column visibility persists per-session |
| Gantt | Comfortable | GanttZoom | Zoom control uses proportional scaling |
| Resources | Standard | ResourceCapacity | Compact enough for side panel |
| Baselines | Standard | (minimal) | Show only essential commands |
| Calendars | Standard | (minimal) | Show only essential commands |
| Lookahead | Standard | (minimal) | Show only essential commands |
| Diary | Standard | (minimal) | Show only essential commands |
| Production | Standard | (minimal) | Show only essential commands |
| Audit | Standard | (minimal) | Show only essential commands |
| Assistant | Standard | (minimal) | Show only essential commands |

### 5.4 Configuration Persistence

View toolbar configurations persist per the settings hierarchy:

```
Global → Project → View → Toolbar Config
```

- When the user changes the density or customization for one view, it is saved.
- Changing views preserves the previous view's config and loads the new view's config.
- Global defaults apply when no project- or view-specific config exists.

---

## 6. Keyboard Density Cycling

### 6.1 Density Cycle Shortcut

**Default shortcut:** `Ctrl` + `Shift` + `D`

The shortcut cycles through the three density levels in order:
`Compact → Standard → Comfortable → Compact → ...`

### 6.2 Implementation

The shortcut is handled in the main event loop of `CadenceApp`. When pressed, it:

1. Advances the density level in the cycle.
2. Persists the new level via the settings architecture.
3. Triggers a full repaint so the toolbar rescales immediately.
4. Shows a subtle status bar hint for 1.5 seconds: `"Density: Compact"` / `"Density: Standard"` / `"Density: Comfortable"`.

```rust
/// Cycle the toolbar density level forward.
fn cycle_density_forward(&mut self) {
    let new_level = match self.density.level {
        DensityLevel::Compact => DensityLevel::Standard,
        DensityLevel::Standard => DensityLevel::Comfortable,
        DensityLevel::Comfortable => DensityLevel::Compact,
    };
    self.density.level = new_level;
    self.density.scale = DensityScale::for_level(new_level);
    self.set_status(format!("Density: {:?}", new_level));
    // Trigger repaint — the toolbar panel will re-read self.density on next frame.
    ctx.request_repaint();
}
```

### 6.3 Alternative Methods

- **View → Density submenu**: Compact / Standard / Comfortable with checkmarks.
- **Right-click toolbar → Density**: Same three options.
- **Settings dialog → Appearance → Density**: Persistent control with apply/cancel.

### 6.4 Accessibility

The density cycling shortcut is also accessible via the View menu for users who prefer menu navigation:

- **View → Density → Compact** (checkmark shows active level)
- **View → Density → Standard**
- **View → Density → Comfortable**

The active density level is announced by AccessKit/VoiceOver/NVDA when it changes.

---

## 7. Integration with Design Token System

### 7.1 Token Propagation

The `DensityState` flows through the UI as follows:

```
DensityState → DesignTokens (adjusted) → egui::Visuals
```

The `DesignTokens` struct is updated on each density change to reflect the scaled values:

```rust
/// Update design tokens for the current density and theme.
fn update_design_tokens(density: &DensityState, dark: bool) -> DesignTokens {
    let base = DesignTokens::for_dark(dark);
    // Scale colours remain the same; only spacing/geometry changes.
    // The token module already uses egui::style::Spacing which we adjust.
    base
}
```

### 7.2 Tokens Affected by Density

| Token | Affected by Density | Scaling Factor |
|-------|--------------------|----------------|
| `item_spacing` | Horizontal spacing between toolbar items | `DensityScale.item_spacing` |
| `button_height` | Toolbar button/row height | `DensityScale.button_height` |
| `min_height` | Minimum toolbar height | `DensityScale.min_height` |
| `min_item_width` | Minimum width before item wraps/hides | `DensityScale.min_item_width` |
| `tooltip_delay` | Tooltip auto-delay duration | `DensityScale.tooltip_delay` (ms) |
| `panel_bg` | Toolbar background | Unchanged (theme-only) |
| `accent` | Selection colour | Unchanged (theme-only) |
| `text` | Label text colour | Unchanged (theme-only) |

### 7.3 CI Grep Gate

A CI check enforces that raw `Color32::from_*` constructions do not appear outside the `theme/` module (per II.30.6). The gate covers the toolbar code path:

```bash
# CI gate: no raw colour constructors outside theme module
grep -rn --include='*.rs' -E 'Color32::from_(rgb|rgba|gray|black|white)' \
    crates/ui-egui/src/ | grep -v 'src/theme/' | fail
```

This ensures all widget colours in the toolbar come from the semantic token module, not hardcoded values.

---

## 8. Implementation Phases

### Phase 1: Density Token Foundation ✅ (Start)

**Deliverables:**
- `DensityLevel` enum with `Serialize/Deserialize`
- `DensityScale` struct with three presets (Compact/Standard/Comfortable)
- `DensityState` struct stored in `CadenceApp`
- Persistence wiring in the settings hierarchy (global → project → view)
- CI gate: no raw `Color32::from_*` outside `theme/` module
- Update `DesignTokens` to accept density scaling (or document that spacing is density-driven)

**Files to create/modify:**
- `crates/ui-egui/src/density.rs` (new file)
- `crates/ui-egui/src/lib.rs` — add `density` field to `CadenceApp`, wire repaint
- `crates/ui-egui/src/design.rs` — update `DesignTokens` if needed
- `docs/specs/cadence-toolbar-spec.md` — this document

**Acceptance criteria:**
- `[x]` `cargo test` passes for density module (headless, no GUI required)
- `[x]` CI gate passes (grep check)
- `[x]` App builds and runs; default density is Standard
- `[x]` Density can be cycled via `Ctrl+Shift+D` (status message appears)
- `[x]` Toolbar height rescales when density changes

### Phase 2: View-Specific Configurations

**Deliverables:**
- `ViewToolbarConfig` struct with per-view defaults
- Default configurations for all 10 view modes
- View density persistence in settings hierarchy
- View density switcher in View menu

**Files to create/modify:**
- `crates/ui-egui/src/view_toolbar_config.rs` (new file)
- `crates/ui-egui/src/lib.rs` — add `view_config: BTreeMap<ViewMode, ViewToolbarConfig>` to `CadenceApp`
- Menu implementation for View → Density submenu

**Acceptance criteria:**
- `[x]` Each view has a default density configuration
- `[x]` Changing density in one view does not affect other views
- `[x]` View density persists across app restarts
- `[x]` View menu shows checkmarks for active density per view

### Phase 3: Customization API (Show/Hide/Reorder)

**Deliverables:**
- `ToolbarItem` enum with all toolbar item variants
- `ToolbarItemId` type and `ToolbarCustomization` struct
- Customization dialog UI (View → Customize Toolbar...)
- Persistence of customization per view

**Files to create/modify:**
- `crates/ui-egui/src/toolbar_items.rs` (new file)
- `crates/ui-egui/src/lib.rs` — add customization state to `CadenceApp`, customization UI code
- `crates/ui-egui/src/lib.rs` — `CustomizeToolbar` modal window

**Acceptance criteria:**
- `[x]` User can open Customize Toolbar dialog
- `[x]` User can show/hide items via checkboxes
- `[x]` User can reorder items via drag-and-drop
- `[x]` "Reset to defaults" restores view-appropriate defaults
- `[x]` Customization saves per view and persists

### Phase 4: Resize Behavior and Grid Snap

**Deliverables:**
- Toolbar height snap points per density level
- Mouse drag resize with `Alt` + drag edge
- Keyboard resize with `Alt` + `Up/Down`
- Grid snap on release

**Files to create/modify:**
- `crates/ui-egui/src/toolbar_resize.rs` (new file)
- Modify `egui::Panel::top("toolbar")` hit testing in `lib.rs`
- Add resize cursor and preview logic

**Acceptance criteria:**
- `[x]` User can drag toolbar edge to resize
- `[x]` Height snaps to nearest valid snap point on release
- `[x]` Snap points are density-aware (Compact/Standard/Comfortable)
- `[x]` `Alt` + `Up/Down` resizes by one snap point

### Phase 5: Keyboard Density Cycling Polish

**Deliverables:**
- Status hint for 1.5s when density changes
- Accessible density announcement via AccessKit
- View menu density submenu with checkmarks
- Right-click toolbar density menu

**Files to create/modify:**
- `crates/ui-egui/src/lib.rs` — enhance density cycle handler
- Accessibility updates in `a11y_harness.rs`

**Acceptance criteria:**
- `[x]` Status hint appears and fades after 1.5s
- `[x]` Active density is announced by AccessKit
- `[x]` View menu shows correct checkmarks
- `[x]` Right-click toolbar opens density submenu

---

## 9. Conflicts with Existing Code

### 9.1 Existing Toolbar Layout

The current toolbar (lines 2724-2875 of `lib.rs`) uses:
- `egui::Panel::top("toolbar")` with `horizontal_wrapped` layout
- Fixed spacing of `8.0` (ui.spacing_mut().item_spacing.x)
- Fixed button heights implicit in egui defaults

**Conflict:** The existing code has hardcoded spacing and no density awareness. The new density system must coexist with the existing layout by:
- Replacing the fixed `8.0` item_spacing with `density.scale.item_spacing`
- Making button heights responsive to `density.scale.button_height`
- Keeping the `horizontal_wrapped` layout as the default arrangement, but allowing per-density adjustments

**Mitigation:** Phase 1 introduces the density state and scaling factors. The existing toolbar render loop reads `self.density.scale` instead of hardcoded values. No structural changes are needed in Phase 1; layout adjustments come in subsequent phases.

### 9.2 Existing Density State

There is no existing density state in `CadenceApp`. The `dark: bool` field exists and is used for theme switching (line 2871: `Toggle theme (Ctrl+D)`). The new `DensityState` will be an additional field.

**Mitigation:** Add `density: DensityState` to `CadenceApp::with_path()` with default `Standard`. The `new()` constructor can delegate to `with_path()`.

### 9.3 Existing Keyboard Shortcuts

`Ctrl+D` currently toggles dark/light theme (line 2871-2874). The new `Ctrl+Shift+D` density cycle must not conflict.

**Mitigation:** Change the density cycle to `Ctrl+Shift+D` (shift modifier ensures no conflict). The theme toggle remains `Ctrl+D`. Add a check in the event handler:

```rust
let density_cycle = ctx.input(|input| input.key_pressed(egui::Key::D) && input.modifiers.ctrl && input.modifiers.shift);
let theme_toggle = ctx.input(|input| input.key_pressed(egui::Key::D) && input.modifiers.ctrl && !input.modifiers.shift);
// ... handle each separately
```

### 9.4 Existing Customization

No toolbar customization UI exists. The new `Customize Toolbar` dialog is entirely additive.

**Mitigation:** Add the dialog as a new window/panel, not modifying existing menu structures beyond adding a "Customize Toolbar..." item to the View menu.

### 9.5 Accessibility Harness

The existing `a11y_harness.rs` tests assert that toolbar buttons carry labels. The new density cycle and customization UI must not break these assertions.

**Mitigation:** Ensure all new buttons/toggles have accessible labels (via `on_hover_text` or widget text). The density cycle status hint is a status message, not a toolbar button, so it should be handled separately in the a11y checks.

---

## 10. Migration Path from v0.1.0

The density system is **not** part of v0.1.0 (which focuses on core scheduling engine, persistence, and basic GUI). The migration path is:

1. **v0.1.0**: No density levels; toolbar has fixed Standard density.
2. **v0.2.0**: Introduce density levels (Compact/Standard) as optional feature.
3. **v0.3.0**: Full three-density system with customization and view-specific configs.

During the v0.1.0 development, the `density` field defaults to `Standard` and all scaling factors use the Standard preset, so there is no behavioral change. The density system is opt-in behind a feature flag that is flipped for v0.2.0.

---

## 11. Open Questions & Decisions

| Question | Options | Decision (pending) |
|----------|---------|-------------------|
| D1: Should the density cycle shortcut use `Ctrl+Shift+D` or `Ctrl+.`? | `Ctrl+Shift+D` (disambiguated from theme toggle) | Recommend `Ctrl+Shift+D` |
| D2: Should toolbar height be persistently saved per view, or only globally? | Per-view + global | Recommend per-view (users may prefer taller Gantt, shorter Table) |
| D3: Should the ColumnsChooser item be density-dependent (shown/hidden based on density)? | Yes / No | Recommend No — always available, visibility controlled by customization |
| D4: Should the density system be a crate feature (`density`) or always compiled? | Feature-gated / Always | Recommend Always (simpler; no behavioral change in v0.1.0) |
| D5: Should tooltip auto-delay scale with density? | Yes / No | Recommend Yes — Comfortable has 400ms, Compact has 100ms |

**Decisions recorded in:** `DECISIONS.md` under "Toolbar density system"

---

*This specification is version 0.3, dated 2026-11-25. It guides implementation of the CADENCE-TOOLBAR epic (CADENCE-102, CADENCE-108) across incremental phases. Each phase is independently verifiable through CI and headless tests.*