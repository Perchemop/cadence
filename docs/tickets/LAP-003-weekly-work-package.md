# Ticket: LAP-003 — Weekly Work Package (MVP)

## Objective
Display week-grouped activity list in Lookahead view with inline owner editing. MVP: activity list + owner per week.

## Target Files
- `crates/ui-egui/src/lib.rs` — Lookahead view: group entries by week (ISO week from scheduled_start), expandable/collapsible sections per week, owner inline text edit per activity
- `crates/commands/src/lib.rs` — `UpdateActivityOwner` command (or reuse existing if present) dispatched from inline edit
- `crates/core/src/model.rs` — `Activity.owner: Option<String>` already exists

## Blocking Dependencies
- LAP-001, LAP-002

## Acceptance Criteria
- [ ] Lookahead view shows collapsible sections per week (Week 1, Week 2, …)
- [ ] Each activity row shows: name, scheduled dates, owner (editable text field), promised checkbox, production %, clash indicator
- [ ] Editing owner field dispatches command → updates `Activity.owner` in master → undoable
- [ ] Owner changes immediately visible in all views (Table, Gantt, Resources)
- [ ] Keyboard navigation: Tab through owner fields, Enter to commit, Esc to cancel

## Verification
- Headless test: set owner via command → verify `Activity.owner` updated
- UI test: `egui_kittest` — owner field has accessible label, Tab order correct