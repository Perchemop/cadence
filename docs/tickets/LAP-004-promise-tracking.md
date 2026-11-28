# Ticket: LAP-004 — Promise Flag Tracking

## Objective
Display and edit the `promised` boolean flag per activity in the Lookahead view.

## Target Files
- `crates/ui-egui/src/lib.rs` — Lookahead view: promised checkbox per activity row
- `crates/commands/src/lib.rs` — `UpdateActivityPromised` command (or reuse existing pattern)
- `crates/core/src/model.rs` — `Activity.promised: bool` already exists

## Blocking Dependencies
- LAP-003

## Acceptance Criteria
- [ ] Checkbox per activity in Lookahead view reflects `Activity.promised`
- [ ] Toggling checkbox dispatches command → updates master `Activity.promised` → undoable
- [ ] Change immediately visible in all views
- [ ] Accessible: checkbox has label "Promised" via `labelled_by`

## Verification
- Headless test: toggle promised → verify `Activity.promised` toggled
- a11y test: checkbox labelled, keyboard toggleable