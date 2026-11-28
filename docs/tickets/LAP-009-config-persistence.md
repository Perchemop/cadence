# Ticket: LAP-009 — Window Configuration Persistence

## Objective
Persist lookahead window configuration (weeks, anchor date) in ViewSettings so it restores on project open.

## Target Files
- `crates/core/src/settings.rs` — add `lookahead_window_weeks: u8` (1–6, default 2), `lookahead_anchor_date: Option<Date>` to `ViewSettings`
- `crates/persistence/src/lib.rs` — schema migration v11: add columns to `view_settings` table or new `lookahead_config` table
- `crates/ui-egui/src/lib.rs` — on app/project load, read config and apply to Lookahead view state

## Blocking Dependencies
- LAP-001 (schema migration)

## Acceptance Criteria
- [ ] User sets window to 4 weeks, anchor to next Monday → closes project → reopens → Lookahead view shows 4 weeks from that Monday
- [ ] Schema migration v11 applies cleanly to v10 databases (no data loss)
- [ ] Default: 2 weeks, anchor = today (if not set)
- [ ] Config change immediately updates Lookahead view

## Verification
- Migration test: open v10 project → migrate → verify config readable
- Integration test: change config → save project → reload → verify restored