---
description: Parallel ticket execution in isolated worktrees — one sub-agent per worktree, no file collisions (guide §5.6.3, §5.7).
---

# /implement-spec — parallel work without file collisions

**Purpose.** Execute independent, non-blocking tickets of one specification
in parallel sub-agents, isolated so two agents never modify the same files
(Case 4, §5.6.2). Cases 1–3 and 5 never spawn a team.

**Invocation.** `/implement-spec <feature>` after `/to-tickets`, when the
dependency graph actually contains independent branches.

**Binding sequence (guide §5.6.3):**

1. Parse the ticket dependency graph from `docs/tickets/`.
2. Create isolated temporary git worktrees (`git worktree add`) for the
   non-blocking tickets. This is a git operation, not a build command, so
   it is allowed locally.
3. Dispatch one OpenCode `task` sub-agent per worktree. Each sub-agent obeys
   the HARD BUILD RULE — it may not run `cargo` or any build/test command
   locally; it verifies through `ci.sh` like any other session (§5.7.5).
4. Merge completed branches sequentially into the feature branch **only
   after** `ci.sh` reports green for that worktree, and **only** via the
   Dev account (`Cadence Dev <dev@example.com>`, II.26.7). No direct local
   merges into `main` — the feature branch itself lands through the normal
   PR + §9.3 path.

**Team rules (§5.7, when a coordinated team is used):** one Orchestrator;
2–5 specialised teammates; strict file ownership; the shared task list and
the team definition live on disk (`docs/tickets/`, the spec appendix);
teammates propose plans and code is written only after plan approval;
inter-agent exchanges are logged to `docs/team-log/<ticket-id>.md`; at
completion every teammate flushes state to the log and the Orchestrator
records the shutdown in `PROGRESS.md` with the final green CI run ID.

**Reconciliation.** Guide §5.7.5 governs: "bypass permissions" is refused,
build/test commands are never pre-approved, and a deliverable is a CI
artefact downloaded and run — never a local build.
