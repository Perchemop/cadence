# Ticket: LAP-002 — Live Master Schedule Derivation

## Objective
Ensure the Lookahead view always reflects the current master schedule with no separate stored state. Extend `lookahead()` to accept configurable window.

## Target Files
- `crates/core/src/lookahead.rs` — modify `lookahead()` signature: `fn lookahead(project: &Project, window_weeks: u8, anchor: Date) -> Vec<LookaheadEntry>`
- `crates/core/src/model.rs` — ensure `LookaheadEntry` has all fields needed by UI (already has: scheduled dates, critical, in_progress, owner, promised, percent_complete)
- `crates/ui-egui/src/lib.rs` — Lookahead view calls `lookahead()` each frame/render with current config

## Blocking Dependencies
- LAP-001 (config in ViewSettings)

## Acceptance Criteria
- [ ] Changing an activity's dates in Table view immediately updates Lookahead view (no refresh needed)
- [ ] Adding/removing activities in master schedule immediately reflects in Lookahead
- [ ] Calendar changes (non-working days) immediately affect lookahead window inclusion
- [ ] No "stale" lookahead state possible — always derived from current `Project`
- [ ] Performance: < 100 ms derivation for 10k activities

## Verification
- Headless test: mutate project → call `lookahead()` → assert results match expectation
- Integration test: UI - change dates in one view → verify other view updates