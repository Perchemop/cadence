# Ticket: GUI-006 — Toast Notifications for Undo/Actions

## Objective
Implement a toast notification system for undo operations and action status, matching the pattern from Linear, Planera, and Airtable. This is a **medium priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/design/src.rs` — Add `Toast` component and toast management system
- `crates/ui-egui/src/lib.rs` — Integration of toast system into UI views
- `crates/settings/src/lib.rs` — Toast preferences (optional)

## Toast Component Design (from gui-inspiration-audit.md)
- Top-right positioned toast stack
- Undo action on destructive operations (5s timeout)
- Auto-dismiss info/success, persist warnings/errors
- Distinct visual patterns for different types

## Toast Types (from research)
```rust
/// Type of toast notification
pub enum ToastType {
    Success,      // operation completed successfully
    Info,         // informational message
    Warning,      // cautionary message
    Error,        // error condition
    Undo,         // undo offer for destructive operation
}
```

## Toast Lifecycle (from research)
```rust
pub struct Toast {
    pub id: String,               // unique identifier
    pub r#type: ToastType,        // type of notification
    pub title: String,            // brief title
    pub message: String,          // detailed message
    pub timeout: Option<Duration>, // auto-dismiss timeout (None = persistent)
    pub show_undo: bool,          // whether undo offer is shown
    pub undo_action: Option<String>, // action to undo
}
```

## Toast Position and Stack
- Position: top-right corner of main application window
- Stack: toasts queue up vertically when multiple are shown
- Maximum display: 3-5 toasts before oldest is dismissed
- Auto-dismiss: info/success after appropriate duration; warnings/errors persist until user action

## Undo Integration (from research)
- Undo action on destructive operations after 5-second timeout
- Toast shows "Undo" button for 5 seconds after operation
- Clicking undo reverses the most recent action
- Undo history maintains the same as manual edits
- Command description names the operation origin

## Blocking Dependencies
- None (foundational UI component)

## Acceptance Criteria
- [ ] Toast notifications appear in top-right corner
- [ ] Toast types: Success, Info, Warning, Error, Undo
- [ ] Undo toast shown for destructive operations with 5s timeout
- [ ] Auto-dismiss: info/success after duration, warnings/errors persist
- [ ] Maximum 3-5 toasts displayed simultaneously
- [ ] Clicking an auto-dismissed toast navigates to it (optional)
- [ ] Toast stack handles multiple concurrent toasts
- [ ] Success and info toasts auto-dismiss after appropriate duration
- [ ] Warnings and errors persist until user dismisses

## Verification
- Headless test: Perform destructive action → undo toast appears → 5s timeout → auto-dismiss
- UI test: Multiple actions → toast stack renders correctly, each toast type styled appropriately
- Integration: Toast works with undo history, command stack, density modes
- Edge case: Rapid actions → toasts queue properly, none lost
- Edge case: Long-running operation → toast shows progress state