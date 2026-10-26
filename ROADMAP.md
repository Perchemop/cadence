# Roadmap

Cadence is a clean-room construction and capital-project scheduling
application: WBS, full dependency logic, CPM engine, working-day calendars,
resources with leveling, baselines, earned-value metrics, a Gantt/table GUI,
reports in Markdown and PDF, and open interchange (MSPDI, MPX, CSV, JSON).

Status is updated **only from evidence**: a capability is counted when it is
implemented and verified (CI run on the platforms, or hands-on local test
where CI cannot see it).

## Release milestones

### v0.1.0 — first functional test build (this tag)

The whole Phase 0–4 plan plus Phase 5 apart from the items below. Verified by
CI run on the release commit: 203+ tests per platform (Linux, Windows, macOS),
clippy with warnings denied, release binary packaged for all three.

| Area | Readiness | Evidence |
|---|---|---|
| CPM engine (passes, float, critical set, cycles, constraints) | 100% | `engine.rs` unit tests + property tests |
| Working-day calendars (multi-calendar, holidays) | 100% | unit + property tests |
| WBS hierarchy | 100% | unit + command tests |
| Command layer (typed, undo/redo) | 100% | command tests per family |
| SQLite project files (schema v4, migrations) | 100% | persistence round-trip/migration tests |
| Resources, histograms, leveling | 100% | engine + command + property tests |
| Baselines, variance, EVM | 100% | engine tests, GUI Baselines view |
| Progress updating | 100% | command + persistence tests |
| Gantt + table GUI | 100% of built views | Phase-2/3 QC checklists (interactive parts hands-on) |
| Reports (Markdown + PDF) and printing | 100% | report + PDF writer tests; dialog hands-on |
| Interchange MSPDI / MPX / CSV / JSON | 100% of documented subsets | round-trip + property tests |
| CI: multi-OS verify + release artefacts | 100% | runs on every push; release.yml on tags |
| Accessibility | ~60% | WCAG AA contrast verified; keyboard map added; screen-reader/keyboard-only audit outstanding |
| Crash reporting | stub | local panic hook + crash log (guide §6.1 allows a stub) |
| Update check | stub | version reported; network check deferred to 0.2.x |

### v0.1.x — polish the first release

- [ ] Fix anything the first release surfaces (bug reports via GitHub Issues).
- [x] Linux aarch64 release binary proven on a real ARM machine — the
      maintainer's own (v0.1.1: installed, checksum-verified, and exercised
      through the demo, save/load, report and PDF).
- [ ] Windows and macOS release binaries proven on real machines.

### v0.1.1 — open-source readiness (project health)

A review of the project against GitHub's opensource.guide found that the
*program* was complete but the *project* was not: missing community and security
documents, no issue templates, no static analysis, no dependency-licence or SBOM
step, and no written governance. This milestone closes that gap. None of it
touches the scheduling engine. See D18 for the decisions and
[`docs/qc/project-health.md`](docs/qc/project-health.md) for the evidence.

- [x] Required documents: `CODE_OF_CONDUCT.md`, `SECURITY.md` (with a threat
      model and an incident-response plan), `ACCESSIBILITY.md`, `VISION.md`,
      `GOVERNANCE.md`, `CONTRIBUTORS.md`.
- [x] `README.md` and `CONTRIBUTING.md` rewritten for a stranger: what it is,
      why it is useful, how to start, how to run the tests, what review expects,
      and where to get help.
- [x] Issue forms (bug, feature, accessibility) and a pull request template with
      an accessibility checklist for UI changes.
- [x] Security workflow: secret scanning over the full history, dependency
      licence/advisory/ban/source checks (`deny.toml`), dependency review on
      pull requests, CodeQL static analysis (Rust and the workflows), and a
      CycloneDX SBOM per run and per release.
- [x] Dependabot for cargo and GitHub Actions; every action pinned to a commit
      SHA.
- [x] Protected default branch with required status checks.
- [x] `ATTRIBUTION.md` extended to non-code assets, accepted licence classes,
      and the terms of the tooling and models used to build the project.
- [x] `Cargo.lock` refreshed by the CI lockfile workflow, and the freshness
      check switched on (D19). The first security run also flagged four
      unmaintained-crate advisories from the GUI stack, a yanked
      `clipboard-win` pin, and path-only internal workspace requirements —
      answered with recorded ignores, the regeneration, and explicit versions
      (D20).
- [ ] Maintainer-side items recorded in [`VISION.md`](VISION.md) and awaiting
      the human: name/trademark check, canonical home URL and handles,
      employer/IP question, second admin and MFA.

### Next slice (decided 2026-10-24)

- [x] Machine-readable schedule output (`--json`) from the CLI, so the tool can
      be driven by scripts and assistive technology without parsing the aligned
      plain-text table (guide section 21.5, accessibility of tooling).

### Next slice (decided 2026-10-26)

- [x] Print the Gantt view: capture the rendered view and emit an image page
      through the existing PDF writer (GUI **Print Gantt…**).

### Next slice (decided 2026-10-26, after Gantt print)

- [x] Crash reporting: GitHub Issue creation with user consent and a UI
      preview before send (v0.2.0 suite infrastructure, guide §7.1). The GUI
      opens a pre-filled issue URL in the browser; no token is embedded.

### Next slice (decided 2026-10-26, after crash reporting)

- [ ] Update check: query the GitHub Releases API, compare versions, and
      surface the result in the GUI; fail soft offline (guide §7.2).

### Human-gated (recorded, not decided)

- [ ] Name conflict check and trademark search for "Cadence"; reserve the
      canonical handles/domain (`VISION.md` N1, N2).
- [ ] Employer/IP policy check recorded in
      `docs/assumptions-and-questions.md` (`VISION.md` N3).
- [ ] A second admin with MFA, for continuity and least privilege
      (`VISION.md` N4, N5).

### v0.2.0 — suite infrastructure

- [x] Crash reporting: GitHub Issue creation with user consent, UI preview
      before send (guide §7.1); the local log and the pre-filled browser issue
      flow are in place, no token is embedded.
- [ ] Update check: query GitHub Releases API, compare versions, surface in
      the GUI; fail soft offline (guide §7.2).
- [x] Print the Gantt view: capture the rendered view and emit an image page
      through the existing PDF writer (GUI **Print Gantt…**, whole-window
      screenshot; timeline cropping/stitching is a later polish item).
- [ ] Accessibility: AccessKit integration for screen readers; full
      keyboard-only operation audit.

### v0.3.x — deeper scheduling

- [ ] Progress-driven rescheduling mode (actual dates feeding the engine —
      D15 records why this must be an explicit mode).
- [ ] Per-assignment resource units and cost rate tables (D11 records the
      current one-unit model).
- [x] Calendar editing in the GUI (Calendars view, calendar definition
      command, per-activity assignment).

### Later

- [x] Resource usage rendered in the GUI (per-day, per-resource, in the
      Resources view — as a table; a bar-chart form is still open polish).
- [ ] Suite shared crates (`crash-report`, `update-check`) extracted once a
      second application joins the suite (guide §7.3).

## Out of scope (recorded in `docs/scope.md`)

Mobile platforms; any type of telemetry; proprietary file formats.
