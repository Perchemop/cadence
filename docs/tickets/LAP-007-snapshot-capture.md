# Ticket: LAP-007 — Snapshot Capture for Audit

## Objective
Expose the existing `CaptureLookaheadSnapshot` command in the Lookahead view UI for audit/drift comparison.

## Target Files
- `crates/ui-egui/src/lib.rs` — Lookahead view: "Capture Snapshot" button in header → dialog for snapshot name → dispatches `CaptureLookaheadSnapshot`
- `crates/commands/src/lib.rs` — `CaptureLookaheadSnapshot` already exists (lines 1674-1707)
- `crates/core/src/model.rs` — `LookaheadSnapshot`, `LookaheadSnapshotEntry` already exist
- `crates/persistence/src/lib.rs` — persistence already handles snapshots

## Blocking Dependencies
- LAP-002

## Acceptance Criteria
- [ ] "Capture Snapshot" button in Lookahead view header
- [ ] Clicking opens dialog for snapshot name (required, non-empty)
- [ ] On confirm: dispatches `CaptureLookaheadSnapshot` with name, current date, window start, window weeks
- [ ] Snapshot appears in snapshots list (existing UI)
- [ ] Snapshot included in JSON export (existing behaviour)
- [ ] Snapshot is read-only — no edit/delete of individual entries within snapshot

## Verification
- Headless test: capture snapshot → verify `LookaheadSnapshot` created with correct entries
- UI test: button labelled, dialog accessible, name validation