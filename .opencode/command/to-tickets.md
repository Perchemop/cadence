---
description: Split a spec into atomic, CI-checkable tickets in docs/tickets/ (guide §5.6.3).
---

# /to-tickets — atomic work units sized for one session each

**Purpose.** Parse `docs/specs/<feature>.md` and split it into atomic,
sequential work units, each small enough to implement inside one
Smart-Zone session (< 150,000 tokens, §5.6.1).

**Invocation.** `/to-tickets <feature>` in a fresh session (`/new` first —
never carry specification context into ticketing).

**Binding rules (guide §5.6.3):**

- One file per ticket: `docs/tickets/<id>.md`.
- Ticket schema: **Objective**, **target file paths**, **blocking
  dependencies**, **verifiable acceptance criteria**.
- A ticket is *ready for work* only when its dependencies are merged to
  `main` and its acceptance criteria are objectively checkable by CI —
  headless tests, the fmt/clippy gates, the platform build (D4, D17).
- Keep tickets atomic: one vertical behaviour each. No speculative
  scaffolding "for later", no dead code — every ticket ends integrated,
  tested and reachable from the entry points it claims to change (§20).
- Ticket IDs are stable and referenced downstream: `/implement <id>`,
  `PROGRESS.md` entries, and commit messages cite them.

**Reconciliation.** Guide §5.6.5 governs conflicts. Ticket sizing never
justifies running builds locally: however small the ticket, verification is
`ci.sh` only, and CI evidence (not the diff's appearance) is what makes a
ticket done.
