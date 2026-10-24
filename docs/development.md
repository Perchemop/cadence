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

This file describes the **maintainer's** machine, where no build may run.
Contributors are not restricted this way: the commands to build and test
locally are in [`CONTRIBUTING.md`](../CONTRIBUTING.md#running-the-tests), and
running them before opening a pull request is expected.

## Security workflow (`.github/workflows/security.yml`)

Kept separate from the build workflow so its result is readable on its own:

- **Secret scanning** with `gitleaks` over the full history (a secret in an
  older commit is still a leaked secret).
- **Dependency licences, advisories, bans and sources** with `cargo deny`,
  configured by `deny.toml`. A disallowed licence fails the build.
- **Dependency review** on pull requests, failing at moderate severity.
- **Static analysis** with CodeQL: `rust` (build mode `none`, so no full
  compile is needed) and `actions`, which analyses the workflow files
  themselves.
- **SBOM**: a CycloneDX bill of materials per run, and one attached to each
  release by `.github/workflows/release.yml`.

`.github/dependabot.yml` opens grouped weekly update pull requests for cargo and
for GitHub Actions, and every action in every workflow is pinned to a commit
SHA — except `dtolnay/rust-toolchain`, which reads its own ref name to select
the toolchain. The pinned majors currently predate the Node 24 runner image and
produce a deprecation warning while GitHub forces them onto the newer runtime;
the major-version bumps are Dependabot's job, and each one arrives as a normal
CI-verified change rather than being applied by hand.

`.github/workflows/stale.yml` marks issues stale after four months and closes
them a month later, exempting `security`, `accessibility`, `pinned`,
`good first issue` and `help wanted`. Pull requests are never staled.

`.ci-required.txt` lists the check names that decide whether a `ci.sh` run is
a success. Without it the script cannot distinguish "every check that exists
has passed" from "only one check has registered yet", and it reported a false
success once while the security workflow was still running.

## Lockfile workflow (`.github/workflows/lockfile.yml`)

`Cargo.lock` cannot be regenerated locally, so this workflow resolves the
dependency graph on a runner — on every pull request, or on demand — and
uploads the resulting `Cargo.lock` as an artefact. Download it and commit it
like any other change; the file is deliberately not committed from inside the
workflow, so a lockfile change is always reviewed and CI-verified. The workflow
also carries the freshness check that fails a build whose lockfile no longer
matches the manifests; it is switched on together with the refreshed lockfile
(D19).

## Layout

- `crates/core` — domain model, CPM engine, resource analysis, reports (pure,
  headless; no I/O)
- `crates/commands` — typed command layer, undo/redo
- `crates/persistence` — SQLite project files (versioned schema, migrations)
- `crates/interchange` — MSPDI, MPX, CSV and JSON import/export
- `crates/ui-egui` — egui/eframe application shell (table, Gantt, resources,
  baselines, calendars)
- `crates/app` — the `cadence` binary (CLI and GUI entry point) and its example
- `.opencode/agents/` — project-local agent definitions (orchestrator, coder,
  explorer, reviewer, fixer, tester, docs-writer, ci-runner, designer)
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
secrets are committed, and the security workflow enforces it: `gitleaks` scans
the full history on every push, every pull request and weekly. A manual scan of
tracked files was performed for the v0.1.0 release
(`docs/qc/release-v0.1.0.md`); the automated scan replaces that step.

The default branch is protected: changes land through a pull request whose
required status checks pass. The maintainer does not push to `main` directly,
and the workspace tooling (`ci.sh`) always goes through a pull request
(D18, `docs/qc/project-health.md`).
