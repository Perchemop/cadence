# Ticket: LAP-001 — Lookahead Window Configuration

## Objective
Add configurable rolling window (1–6 weeks, default 2) with anchor date to the Lookahead view. Persist in ViewSettings.

## Target Files
- `crates/core/src/model.rs` — add `window_weeks: u8`, `anchor_date: Option<Date>` to `ViewSettings` or new `LookaheadConfig`
- `crates/core/src/settings.rs` — `Default` impl, validation (clamp 1–6)
- `crates/persistence/src/lib.rs` — schema migration to v11, read/write new fields
- `crates/commands/src/lib.rs` — `UpdateLookaheadConfig` command
- `crates/ui-egui/src/lib.rs` — dropdown (1–6) + date picker in Lookahead view header

## Blocking Dependencies
None (foundational)

## Acceptance Criteria
- [ ] User can set window 1–6 weeks via dropdown in Lookahead view header
- [ ] User can set anchor date via date picker (defaults to today)
- [ ] Settings persist across project close/reopen
- [ ] Schema migration v11 applies cleanly to v10 databases
- [ ] `clippy -D warnings`, `fmt`, all tests pass

## Verification
- Headless test: create project → set config → save → reload → verify config restored
- Unit test: `UpdateLookaheadConfig` clamps values 1–6