# Development

## Prerequisites

- Rust stable (via [rustup](https://rustup.rs)). Nothing else is required
  locally.

## Local workflow (keep it light)

This project is developed on a resource-constrained laptop, so **heavy work
runs on GitHub Actions**, not locally:

1. Edit source files and write tests.
2. Run `cargo fmt` and, only if fast enough, `cargo check -p <crate>`.
3. Commit in small, focused units and push to `main`.
4. Wait for the CI run and read its results.
5. **Never assume a change is good until the GitHub Actions run is green.**

Full builds, clippy, the complete test suite, and release builds are performed
only in CI (`.github/workflows/ci.yml`).

## Layout

- `crates/core` — domain model + CPM engine (pure, headless)
- `crates/commands` — typed command layer, undo/redo
- `crates/persistence` — SQLite project files + open interchange
- `crates/app` — application binary
- `.opencode/agents/` — project-local agent definitions (orchestrator, coder,
  explorer, reviewer)
- `docs/parity.md` — living capability list
- `docs/qc/` — manual QC checklists per phase
- Root tracking files: `IMPLEMENTATION_PLAN.md`, `REQUIREMENTS_TRACEABILITY.md`,
  `DECISIONS.md`, `PROGRESS.md`, `CHANGELOG.md`

## Agent model tiers

Per the guide, the orchestrator plans only; heavy multi-file implementation
goes to the strongest free coding model; exploration and review use fast free
tiers. Model IDs in `.opencode/agents/*.md` are defaults and may be adjusted
to whatever free provider is currently available.

## Conventions

- No panics in non-test code; return rich `Result` types.
- Every mutation goes through the command layer.
- Tests are written with every change.
