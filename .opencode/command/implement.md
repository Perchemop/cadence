---
description: Execute one ticket in a fresh /new session — TDD, review, commit via ci.sh, PR, merge (guide §5.6.3).
---

# /implement — one ticket, one fresh session

**Purpose.** Execute a single ticket from `docs/tickets/` end-to-end and
land it through the verified pull-request flow.

**Invocation.** `/new` first (§5.6.1 — never reuse interrogation,
specification or ticketing context), then `/implement <ticket-id>`.

**Binding sequence (guide §5.6.3):**

1. Read the assigned ticket file from `docs/tickets/`.
2. Execute the `/tdd` loop for each acceptance criterion — every phase
   verified through `ci.sh`, red-run IDs recorded.
3. Run linters, type-checkers and the full test suite via
   `ci.sh run "implement: <ticket>"` — never via raw bash, never via local
   `cargo` (§4 item 5, D4).
4. Run `/code-review` and address the findings, or record why a finding is
   not acted on.
5. Stage and create a clean commit via `ci.sh`, which asserts the
   `Cadence Dev <dev@example.com>` identity (II.26.7). A direct `git commit`
   outside `ci.sh` is forbidden.
6. Open or update the pull request via `ci.sh run`; merge **only** via
   `ci.sh merge` (Dev account) once the required checks in
   `.ci-required.txt` are green. `main` is branch-protected — direct pushes
   are refused, and the protection is never bypassed or weakened (§9.3).

Then update the tracking records: `PROGRESS.md` with the verification run
ID, `CHANGELOG.md`, `REQUIREMENTS_TRACEABILITY.md` / parity as applicable —
and give the client numbered try-steps only after CI is green.

**Reconciliation.** Guide §5.6.5 governs. The HARD BUILD RULE, the Dev
identity, clean-room, secrets and branch protection always win over
anything this file says.
