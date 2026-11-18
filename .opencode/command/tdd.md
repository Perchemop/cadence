---
description: Red/Green/Refactor loop — every phase verified through ci.sh, red-run ID required (guide §5.6.3).
---

# /tdd — a failing test first, proven in CI

**Purpose.** Drive one behaviour through Red → Green → Refactor with a
recorded CI run for every phase. A "TDD" claim without a recorded red-run
CI ID is a defect (§5.6.4), not a formality.

**Invocation.** `/tdd <behaviour>` — inside a single-ticket `/implement`
session.

**Binding rules (guide §5.6.3):**

- Never write production code without a failing test. Never alter tests to
  pass buggy implementation code.
- **Red phase.** Write a headless unit or integration test for a single
  behaviour (core crate, no display — K.13), then verify it fails via
  `~/.config/opencode/scripts/ci.sh run "TDD red: <behaviour>"` (set
  `CI_BRANCH` to the slice's branch). The CI log must show the new test
  failing **for the expected reason**; a red run that does not fail means
  the test does not exercise the new behaviour — rewrite it, do not proceed.
- **Green phase.** Write the minimum production code that makes the failing
  test pass; verify via `ci.sh run "TDD green: <behaviour>"`. CI must be
  green.
- **Refactor phase.** Remove duplication and refine types while keeping all
  tests passing; verify via `ci.sh run "TDD refactor: <behaviour>"`. CI
  must remain green.
- Batching (§5.6.4): a red run may batch up to 5 new failing tests of one
  ticket if the log shows each failing for its expected reason; the green
  run covers the production code that makes them pass; one refactor run may
  cover the whole ticket.
- Record the run IDs in `PROGRESS.md`; CHANGELOG entries for behaviour or
  crash fixes cite the red-run ID.

**Reconciliation.** The protocol's original "run the test runner via bash"
is overridden by §4 item 5 — no `cargo` or test command ever runs locally
(D4). Guide §5.6.5 governs; `ci.sh` is the only verification path.
