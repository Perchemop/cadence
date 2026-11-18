---
description: Read-only four-axis audit of all changes before commit or merge (guide §5.6.3).
agent: plan
subtask: true
---

# /code-review — the Reviewer role as an isolated read-only sub-task

**Purpose.** Audit all code changes before committing or merging. OpenCode
runs this as a read-only audit in an isolated sub-agent context
(`subtask: true`, `agent: plan` — no edits possible): the Reviewer role
(§5.3) in OpenCode terms.

**Invocation.** `/code-review` on the finished diff — step 4 of
`/implement`, or on any PR branch before merge.

**Four axes (guide §5.6.3):**

1. **Architectural alignment** — compliance with the specification
   document, `AGENTS.md` conventions and separation of concerns. For
   Cadence: the core / commands / persistence / app layering (D3), every
   mutation through `cadence-commands`, headless logic outside the paint
   loop, II.30 token discipline for UI changes.
2. **Tactical correctness** — off-by-one errors, unhandled null/None
   states, race conditions, missing or weak test coverage, panics outside
   tests, fake success states.
3. **Clean-room axis** — no proprietary source, assets or reverse-
   engineering material; public research notes are original summaries with
   URLs only (§2.2); no copied trade dress.
4. **Secrets axis** — no tokens, keys, credentials or real personal data in
   the diff, the PR text or the commit metadata (II.29).

Report findings by file and line with severity, in priority order; do not
fix anything. A review that accepts dead code, placeholder-only data or an
unintegrated module is a failed review (§20) — reject it.

**Reconciliation.** Guide §5.6.5 governs. The clean-room and secrets axes
are additions the base protocol does not carry; they are never waived, and
a red axis blocks the commit or merge regardless of the other axes.
