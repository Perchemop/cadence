# Ticket: LAP-008 — Diary Integration

## Objective
Allow creating daily notes from the Lookahead view pre-linked to the lookahead window start date.

## Target Files
- `crates/ui-egui/src/lib.rs` — Lookahead view: "Add Note" button → opens diary note dialog with `lookahead_window_start` pre-filled to current window anchor
- `crates/commands/src/lib.rs` — `CreateDailyNote` already exists (lines 1759-1795) with `lookahead_window_start: Option<Date>`
- `crates/core/src/model.rs` — `DailyNote.lookahead_window_start` already exists

## Blocking Dependencies
- LAP-001 (window anchor date)

## Acceptance Criteria
- [ ] "Add Note" button in Lookahead view
- [ ] Dialog opens with date = today, `lookahead_window_start` = current window anchor date
- [ ] Saving dispatches `CreateDailyNote` with the window start date
- [ ] Note appears in Diary view (Ctrl+8) and can be filtered by window
- [ ] If user changes window anchor later, existing notes retain their original `lookahead_window_start`

## Verification
- Headless test: create note from lookahead → verify `lookahead_window_start` set correctly
- UI test: button accessible, dialog pre-filled correctly