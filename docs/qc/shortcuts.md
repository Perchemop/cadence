# Keyboard Shortcuts (verified from lib.rs handlers)

Global shortcuts — active from any view, unless a modal dialog is open or a text field has focus.

| Key | Action | Where it works |
|---|---|---|
| `Ctrl+S` | Save the project | Everywhere (acts even while a text field has focus; save never touches focused text) |
| `Ctrl+O` | Open a project file | Everywhere (not behind a modal dialog) |
| `Ctrl+N` | Create a new activity | Everywhere |
| `Ctrl+W` | Close the window | Routed through the unsaved-changes guard (same as the window's close button) |
| `Ctrl+Z` | Undo | Any view, skipped while a text field is focused (where the field's own undo applies) |
| `Ctrl+Shift+Z` / `Ctrl+Y` | Redo | Any view |
| `Ctrl+D` | Toggle light/dark theme | Everywhere |
| `Ctrl+`+` / `Ctrl+=` | Zoom Gantt timeline in | Only in Gantt view, when no text field focused and no dialog open |
| `Ctrl+`-` | Zoom Gantt timeline out | Only in Gantt view, when no text field focused and no dialog open |
| `Ctrl+0` | Reset Gantt zoom to default | Only in Gantt view |
| `F1` | Open the keyboard-shortcuts window | From every view; toggles `show_shortcuts_dialog` |

View switching — held `Ctrl` + number key.

| Key | Action | View |
|---|---|---|
| `Ctrl+1` | Activity table | Table |
| `Ctrl+2` | Gantt | Gantt |
| `Ctrl+3` | Resources | Resources |
| `Ctrl+4` | Baselines | Baselines |
| `Ctrl+5` | Calendars | Calendars |
| `Ctrl+6` | Audit | Audit |
| `Ctrl+7` | Lookahead | Lookahead |
| `Ctrl+8` | Diary | Diary |
| `Ctrl+9` | Production | Production |

Text-editing / table navigation — active in the Table view.

| Key | Action | Where |
|---|---|---|
| `Up` / `Down` | Select the previous/next activity in the activity table | Table view, when no text field focused, no popup open, dialog not open |
| `Tab` / `Shift+Tab` | Move focus between controls | Everywhere |
| `Space` / `Enter` | Activate the focused control or selectable row | Everywhere |
| `Escape` | Drop the extra multi-selection rows (anchor row stays selected) | Table view, when multi-selected rows exist |
| `Ctrl`/`⌘`-click | Add one row to the table selection | Table view |
| `Shift`-click | Add a range of rows from the anchor row to the clicked row | Table view |

F1 — keyboard reference window:

- `F1` opens the keyboard-shortcuts dialog (PR #92, `show_shortcuts_dialog` flag). This window lists all shortcuts and their actions.

RP-3 note:

- The assistant review panel (RP-3..RP-6, tickets `docs/tickets/RP-3.md` … `RP-6.md`) proposes adding an **Assistant view** as a tenth centre view tab, mirroring the nine-view pattern. The shortcut for this view is **proposed — pending RP-3 merge** (the view-shell enabling state depends on RP-3 merging first). Once RP-3 lands, a `Ctrl+0` alternative or a new key will be assigned and this note updated.

Source: all handlers verified in `crates/ui-egui/src/lib.rs` (lines 2182–2297, 2322–2358). Cross-referenced with `docs/usage.md` keyboard map (lines 190–217).