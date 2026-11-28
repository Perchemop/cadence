# Ticket: GUI-003 — Inline Table Editing

## Objective
Implement inline table editing for the activity table, matching the pattern from MS Project, Smartsheet, Airtable, and Planera. This is a **high priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/ui-egui/src/lib.rs` — Table view integration
- `crates/design/src/lib.rs` — EditableCell component and design tokens
- `crates/core/src/model.rs` — Any model changes needed for inline edits

## Inline Editing Features (from gui-inspiration-audit.md)
- Click cell → inline editor (text, number, date picker, combo)
- Enter to commit, Esc to cancel, Tab/Shift+Tab to navigate
- Validation on commit, status message on error
- Support for: text, number, date, combo/dropdown fields

## Cell Editor Types
```rust
/// Inline editor for table cells
pub enum InlineEditor {
    Text(String),               // plain text editor
    Number(f32),               // numeric editor
    Date(ChronologicalDate),   // date picker popup
    Combo(Vec<String>),        // dropdown with options
}
```

## Editing Behavior
- **Click to edit**: User clicks on a cell → inline editor appears
- **Enter to commit**: Commit the edited value
- **Esc to cancel**: Return cell to original value, close editor
- **Tab/Shift+Tab**: Navigate to next/previous cell in edit mode
- **Validation on commit**: 
  - Text: non-empty validation
  - Number: range validation, positive values
  - Date: valid date, within project range
  - Combo: selects from listed options
- **Status message on error**: Temporary error toast/inline message

## Implementation Details
```rust
// Editable cell component
pub struct EditableCell {
    pub row: usize,
    pub column: usize,
    pub original_value: String,
    pub current_value: String,
    pub editor_type: InlineEditor,
    pub is_editing: bool,
    pub committed: bool,
    pub error_message: Option<String>,
}
```

## Supported Column Types (from research)
- **Activity name**: Text editor, non-empty validation
- **Duration**: Number editor, positive whole days validation
- **Percent complete**: Number editor, 0-100 range
- **Start/finish dates**: Date picker popup
- **Resource names**: Combo/dropdown from resource list
- **Constraint type**: Combo from: ASAP, ALAP, SNET, SNLT, FNET, FNLT

## Blocking Dependencies
- GUI-002 (skeleton loaders - can be parallel)

## Acceptance Criteria
- [ ] Click on table cell → inline editor appears
- [ ] Enter commits the edited value
- [ ] Esc cancels editing and restores original value
- [ ] Tab/Shift+Tab navigates to next/previous cell
- [ ] Validation on commit with status message on error
- [ ] Supported editors: text, number, date, combo/dropdown
- [ ] Error messages displayed on invalid commit
- [ ] Works in both light and dark themes

## Verification
- Headless test: Click cell → edit → Enter commits → value updated; Esc cancels
- UI test: Cell editing renders, all editor types work, validation messages appear
- Integration: Edits are reflected in the project model and undo history
- Edge case: Invalid value → error message, value not committed
- Edge case: Programmatic focus → editor opens correctly