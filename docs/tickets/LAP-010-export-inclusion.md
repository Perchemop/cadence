# Ticket: LAP-010 — Export Inclusion

## Objective
Ensure lookahead window config and captured snapshots are included in JSON export; MSPDI/MPX export does not error.

## Target Files
- `crates/interchange/src/json.rs` — extend export to include `lookahead_config` (if new struct) and `lookahead_snapshots` (already exported)
- `crates/interchange/src/mspdi.rs` — verify no error when lookahead data present (format limitation: lookahead not in MSPDI)
- `crates/interchange/src/mpx.rs` — same

## Blocking Dependencies
- LAP-001, LAP-007

## Acceptance Criteria
- [ ] JSON export includes `lookahead_window_weeks`, `lookahead_anchor_date`, and all `lookahead_snapshots`
- [ ] JSON import restores config and snapshots correctly
- [ ] MSPDI export completes without error (lookahead data silently omitted)
- [ ] MPX export completes without error
- [ ] Round-trip: export → import → verify config and snapshots restored

## Verification
- Round-trip test: export → import → assert config + snapshots identical
- MSPDI/MPX test: export → parse with Microsoft Project (if available) or validate XML schema