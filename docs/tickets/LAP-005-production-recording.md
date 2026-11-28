# Ticket: LAP-005 — Per-Activity Daily Production Recording

## Objective
Allow recording per-activity daily production % complete in the Lookahead view, reusing existing `ProductionEntry` and `RecordProduction` command.

## Target Files
- `crates/ui-egui/src/lib.rs` — Lookahead view: production % numeric input per activity per day (or per activity with date context)
- `crates/commands/src/lib.rs` — reuse `RecordProduction` / `UndoRecordProduction` (already exist)
- `crates/core/src/model.rs` — `ProductionEntry` and `ProductionEntryId` already exist

## Blocking Dependencies
- LAP-003

## Acceptance Criteria
- [ ] Production % input per activity in Lookahead view (context: today's date or selected date)
- [ ] Entering value dispatches `RecordProduction` → creates/updates `ProductionEntry` for (activity, date)
- [ ] Value 0–100 (basis points), clamped
- [ ] Existing entry for same (activity, date) is updated (not duplicated) — existing behaviour
- [ ] Entry immediately visible in Production view (Ctrl+9)
- [ ] Undo works via `UndoRecordProduction`

## Verification
- Headless test: record production via command → verify `ProductionEntry` created/updated
- UI test: input field labelled "Production %", keyboard accessible