---
description: Compile /grill-me decisions into docs/specs/<feature>.md (guide §5.6.3).
---

# /to-spec — the specification lives on disk, not in memory

**Purpose.** Compile the aligned decisions from `/grill-me` into a single
Markdown specification that survives `/new` (§5.6.1 — state is
externalised to disk; session history is never a source of truth).

**Invocation.** `/to-spec <feature>` in a fresh session, after a completed
`/grill-me`.

**Binding rules (guide §5.6.3):**

- Write the specification to `docs/specs/<feature>.md`.
- Required sections: **System architecture**, **Interface contracts**,
  **Non-goals**, **Explicit constraints**.
- State Cadence's standing constraints explicitly where they apply: no
  `cargo` or build/test command ever runs locally (D4) — every phase is
  verified through `~/.config/opencode/scripts/ci.sh`; every state mutation
  goes through `cadence-commands`; core tests are headless (no display);
  UI work follows the II.30 token discipline; every git action is authored
  `Cadence Dev <dev@example.com>` and lands on `main` via a PR.
- Clean-room check: a specification may cite public URLs only — no
  proprietary source, no pixel-perfect UI descriptions of a commercial
  product (§2.2). Behavioural summaries are original wording.
- For parallel multi-session work (Case 4), append the **Team Definition**
  appendix (§5.7.3): teammates, roles, scopes, and the global goal quoted
  verbatim.

**Reconciliation.** Guide §5.6.5 governs any conflict; where this command
could be read to allow unverified claims, the §5.6.4 verification mandate
wins — a spec is not "done" by being written, but by driving tickets whose
acceptance criteria CI can check.
