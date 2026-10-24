# Project Health Quality Gate

Checklist for the open-source readiness work (D18, D19): the documents, the
contribution surface, the security pipeline and the governance that a project
needs before it can honestly be called open source. Nothing here changes the
scheduling engine, so the engine-side gates remain in
[`phase-1.md`](phase-1.md), [`phase-2.md`](phase-2.md), [`phase-3.md`](phase-3.md),
[`phase-5.md`](phase-5.md) and [`release-v0.1.0.md`](release-v0.1.0.md).

## Required documents

- [x] `LICENSE-MIT` and `LICENSE-APACHE`, and the "MIT OR Apache-2.0" choice
      stated deliberately in the README (not by accident).
- [x] `README.md` written for a stranger: what it does, why it is useful, how to
      start, where to get help — plus a runnable command example.
- [x] `CONTRIBUTING.md` covering bug reports, feature proposals, environment
      setup, the exact local test commands, review expectations, the clean-room
      rules and the licence terms.
- [x] `CODE_OF_CONDUCT.md` with scope, reporting path and graduated enforcement.
- [x] `SECURITY.md` with a private reporting path, scope, a threat model and an
      incident-response plan.
- [x] `ACCESSIBILITY.md` with the target standard, priorities, known
      limitations, supported environments and the dedicated report path.
- [x] `VISION.md` with the one-paragraph message, goals, non-goals and the
      questions left for the maintainer.
- [x] `GOVERNANCE.md` with roles, decision rule, tiebreaker, commit access,
      conflict resolution, the path to maintainership, tracked metrics and
      sustainability boundaries.
- [x] `CONTRIBUTORS.md`, including triage, documentation and testing as
      contributions, and how to be added.
- [x] `ATTRIBUTION.md` extended to non-code assets, accepted licence classes,
      build-tooling and model terms, and the SBOM location.
- [x] No third-party document text was copied into any of the above; each was
      written for this project.

## Contribution surface

- [x] Issue forms for a bug report, a feature request and (separately) an
      accessibility report, with an issue-template chooser that links to the
      private security path and to the vision's non-goals.
- [x] Pull request template with the existing engineering checklist plus an
      accessibility section for UI changes, a dependency-licence check and a
      secret-scanning line.
- [x] Contribution labels: `good first issue`, `help wanted`, `documentation`
      (existing), plus `security`, `accessibility`, and area labels.
- [x] Expectations published where contributors will see them: seven-day
      response, a few hours a week of maintainer time, tests required, one
      bounded change per pull request.
- [x] A written path to maintainership, and the areas that can be owned
      independently.
- [x] Stale-issue housekeeping: issues are marked stale after four months and
      closed a month later, with `security`, `accessibility`, `pinned`,
      `good first issue` and `help wanted` exempt; pull requests are never
      staled.
- [x] `.ci-required.txt` names the checks that decide a verification run's
      verdict, so a run cannot report success while part of CI is still
      running.

## Security pipeline

- [x] Secret scanning over the full history (a secret in an older commit is
      still a leaked secret).
- [x] Dependency licence, advisory, ban and source checks (`deny.toml`), so a
      disallowed licence fails the build instead of being discovered later.
- [x] Dependency review on every pull request, failing at moderate severity.
- [x] Static analysis (CodeQL) over the Rust sources and the workflow files.
- [x] A CycloneDX SBOM per security run, and one attached to each release.
- [x] Dependabot for cargo and GitHub Actions, with a weekly schedule and a
      bounded open-pull-request limit.
- [x] Every workflow action pinned to a commit SHA, except
      `dtolnay/rust-toolchain`, which reads its own ref name to select the
      toolchain.
- [x] Protected default branch with required status checks, so a CI result is
      meaningful. Configured on `main`: the required checks are the ones
      listed in `.ci-required.txt` (build and package on three platforms, the
      six security checks, and lockfile freshness); branches must be up to
      date; force pushes and branch deletion disabled; the maintainer account
      is exempt from the checks so a broken pipeline cannot lock the project
      out (see `GOVERNANCE.md`).
- [x] Private vulnerability reporting enabled, so a report creates a private
      advisory instead of a public issue (`SECURITY.md` is the public guide to
      it).
- [x] Repository secret scanning and push protection enabled, on top of the
      `gitleaks` scan in the security workflow.
- [x] Dependabot alerts and security updates enabled, alongside the weekly
      grouped update pull requests in `.github/dependabot.yml`.
- [x] Repository description written as the one-paragraph message, and topics
      set so the project is findable (`scheduling`, `construction`,
      `critical-path`, `cpm`, `gantt`, `earned-value`, `rust`, `egui`,
      `offline`, `desktop`, `sqlite`).
- [ ] Required-review rule, once a second maintainer exists (impossible to
      satisfy with one account, so it is recorded rather than enabled).

## Engineering

- [x] `cargo fmt --check --all` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo test --workspace` passes on Linux, Windows and macOS.
- [x] The security workflow passes on `main`.
- [x] `Cargo.lock` refreshed from the lockfile workflow and the freshness check
      enabled (D19): the committed lockfile no longer lags the manifests, the
      yanked `clipboard-win` pin is gone, and any future drift fails the build.

## Human-gated items (recorded, not decided)

These need the maintainer, and are listed in `VISION.md` so they stay visible:

- [ ] Name-conflict and trademark check for "Cadence"; reserve the canonical
      handles/domain.
- [ ] Employer/IP policy check recorded in `docs/assumptions-and-questions.md`.
- [ ] A second admin with MFA.
- [ ] Real-machine testing of the Windows and macOS release binaries.
- [ ] The human-gated research notes in `docs/research/`.

## Evidence

- The checks named in `.ci-required.txt` on PR #15's final commit: all green,
  including the security workflow's licence, advisory, secret-scanning, SAST
  and SBOM jobs.
- The same set re-runs on `main` immediately after the merge; those runs are
  the durable record (the repository's Actions tab keeps them).
