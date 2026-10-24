# Development

## Prerequisites

- Nothing is required locally. Do **not** run any `cargo` command on this
  machine — that includes `cargo fmt`, `cargo check`, `cargo metadata` and
  `cargo tree`. Formatting is matched by hand from the `Diff in ...` lines in
  the CI log; CI is the only verification that exists.

## Local workflow (keep it light)

This project is developed on a resource-constrained laptop, so **all heavy work
runs on GitHub Actions**, never locally. A local build or test run is not an
allowed step of this workflow, no matter which model or tool is driving it.

1. Edit source files and write tests.
2. Verify with the project script, which commits the working tree, pushes it,
   updates the pull request, waits for CI, and prints the outcome — including
   the failing job's logs:

   ```bash
   ~/.config/opencode/scripts/ci.sh run "short description of the change"
   ```

   Related: `ci.sh status` (current checks), `ci.sh logs` (failing logs),
   `ci.sh diff` (what would be committed).
3. Read the result, fix what CI reported, and run the script again.
4. **Never assume a change is good until the CI run is green.** Formatting in
   particular is checked by CI only: match `rustfmt` output from the
   `Diff in ...` lines in the log rather than guessing.

Allowed locally: editing and reading files, `git status`/`diff`/`log`/`show`,
`gh` commands, and the `ci.sh` script itself. Everything else — builds, tests,
clippy, release packaging, benchmarks — belongs to CI.

## CI (`.github/workflows/ci.yml`)

Two jobs:

- **Verify**, a matrix over `ubuntu-latest`, `windows-latest` and
  `macos-latest`. The test suite runs on all three; `cargo fmt --check --all`
  and `cargo clippy --workspace --all-targets -- -D warnings` run on Linux
  only, because they do not depend on the platform. `fail-fast` is off so one
  red platform cannot hide another.
- **Package**, which builds the release binary on Linux and uploads it as the
  `cadence-linux-x86_64` artefact.

Each job writes a summary to the run page. Dependencies are cached by
`Swatinem/rust-cache`.

## Layout

- `crates/core` — domain model, CPM engine, resource analysis, reports (pure,
  headless; no I/O)
- `crates/commands` — typed command layer, undo/redo
- `crates/persistence` — SQLite project files (versioned schema, migrations)
- `crates/interchange` — MSPDI, MPX, CSV and JSON import/export
- `crates/ui-egui` — egui/eframe application shell (table, Gantt, resources,
  baselines)
- `crates/app` — the `cadence` binary (CLI and GUI entry point) and its example
- `.opencode/agents/` — project-local agent definitions (orchestrator, coder,
  explorer, reviewer, fixer, tester, docs-writer, ci-runner)
- `docs/parity.md` — living capability list
- `docs/usage.md` — commands, formats, keyboard map, reports
- `docs/qc/` — manual QC checklists per phase
- Root tracking files: `IMPLEMENTATION_PLAN.md`, `REQUIREMENTS_TRACEABILITY.md`,
  `DECISIONS.md`, `PROGRESS.md`, `CHANGELOG.md`, `ATTRIBUTION.md`

## Agent model tiers

Per the guide, the orchestrator plans only; heavy multi-file implementation
goes to the strongest free coding model; exploration and review use fast free
tiers. Model IDs in `.opencode/agents/*.md` are defaults; the live mapping is
recorded in `docs/current-model-map.md`.

## Conventions

- No panics in non-test code; return rich `Result` types.
- Every mutation goes through the command layer.
- Tests are written with every change.
- Money is `i64` minor units, percentages are basis points (D13).
- Derived schedule data is never persisted; it is recomputed (D9).
- Dependency-light: a new dependency has to earn its place, which is why the
  XML and JSON readers live in-tree (D14).

## Secret scanning

The pull-request template and the release checklist both require that no
secrets are committed. For automated enforcement, run
[`gitleaks`](https://github.com/gitleaks/gitleaks) locally or enable GitHub's
built-in secret scanning on the repository — a manual scan of tracked files
was performed for the v0.1.0 release (`docs/qc/release-v0.1.0.md`).
