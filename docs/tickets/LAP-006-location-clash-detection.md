# Ticket: LAP-006 — Location Clash Detection

## Objective
Detect and display location clashes within the lookahead window: same non-empty location + overlapping scheduled dates.

## Target Files
- `crates/core/src/lookahead.rs` — add `detect_location_clashes(entries: &[LookaheadEntry]) -> Vec<Clash>`; `Clash` struct with `activity_a_id`, `activity_b_id`, `location: String`, `overlap_start`, `overlap_finish`
- `crates/core/src/model.rs` — `Clash` struct definition (or in `lookahead.rs`)
- `crates/ui-egui/src/lib.rs` — Lookahead view: clash indicator column (icon + tooltip), clash detail dialog on click

## Blocking Dependencies
- LAP-002 (live entries)

## Acceptance Criteria
- [ ] Clash detection runs on current lookahead entries each render
- [ ] Pair flagged iff: both have non-empty `location` AND `location` strings equal (case-sensitive) AND scheduled date ranges overlap (inclusive)
- [ ] Clash indicator shown on both activities in the pair
- [ ] Clicking indicator opens dialog: activity A name, activity B name, location, overlap start–finish
- [ ] No false positives: different locations, non-overlapping dates, or empty location → no clash
- [ ] Performance: < 50 ms for 500 entries

## Verification
- Unit test: `detect_location_clashes` with overlapping/non-overlapping/same-different/empty locations
- Property test: symmetry (A-B = B-A), no self-clash, no duplicate pairs
- UI test: indicator has accessible name "Location clash with [activity]"