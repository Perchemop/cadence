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
| Accessibility | 100% (code-level audit) | WCAG AA contrast tests; AccessKit on through eframe; keyboard-only pass over every view; pointer-free link creation and Gantt zoom; limits recorded in `ACCESSIBILITY.md` |
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

- [x] Update check: query the GitHub Releases API, compare versions, and
      surface the result in the GUI; fail soft offline (guide §7.2). The check
      is opt-in (`cadence check-updates` and the GUI **Updates** window), and
      `cadence version` uses only the cached answer.

### Next slice (decided 2026-10-26, after the update check)

- [x] Accessibility audit: AccessKit integration for screen readers and a
      keyboard-only operation pass over every view, with the gaps fixed or
      recorded (v0.2.0 suite infrastructure; guide Part I §9). Added a
      keyboard-shortcuts window (`F1`), `Up`/`Down` row selection, keyboard
      Gantt zoom, a pointer-free **Add relationship** form, a scrollable
      inspector, and an AccessKit summary for the Gantt canvas.

### Next slice (decided 2026-10-27, owner-approved)

- [x] Programme audit tool (research note approved 2026-10-27): audit a
      construction programme inside Cadence for schedule quality and health,
      with a GUI **Audit** view and `cadence audit [--json]`. **Done so far:**
      the core check engine (12 checks, stable ids, configurable thresholds,
      unit tests), the `cadence audit` CLI with the machine-readable
      `kind: "audit"` document, the GUI **Audit** view (`Ctrl+6`, findings
      that jump to the activity, status date + re-run) and the critical-path
      checks (CPT nudge test and the length index against the baseline
      deadline) and the forensic comparison (`cadence audit-compare`,
      baseline comparison, `kind: "comparison"` JSON). Acceptance criteria in
      [`docs/research/schedule-audit.md`](docs/research/schedule-audit.md).

Progress-driven rescheduling remains queued in v0.3.x below.

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
- [x] Update check: query GitHub Releases API, compare versions, surface in
      the GUI; fail soft offline (guide §7.2). Implemented as an opt-in check
      (`cadence check-updates`, GUI **Updates**) with a six-hour cache.
- [x] Print the Gantt view: capture the rendered view and emit an image page
      through the existing PDF writer (GUI **Print Gantt…**, whole-window
      screenshot; timeline cropping/stitching is a later polish item).
- [x] Accessibility: AccessKit integration for screen readers; full
      keyboard-only operation audit. AccessKit is enabled by eframe's default
      features; every view was reviewed for pointer-only paths, with the
      remaining toolkit limits recorded in `ACCESSIBILITY.md`.

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

### Future expansion (approved 2026-10-27, queued)

The owner approved all three research notes on 2026-10-27, with this order:
audit tool first (now the next slice), then the Aphex-inspired lookahead /
field-planning work, then the optional AI integration. Approval lets the
acceptance criteria drive implementation; it does not promise a release date.

- [ ] **Programme audit tool** — an in-app tool that audits a construction
      programme for health and quality (open ends, leads/lags, hard
      constraints, high/negative float, invalid dates, missing resources,
      missed tasks, critical-path test, CPLI/BEI, forensic comparison), with a
      GUI Audit view and `cadence audit [--json]`. Inspired by public
      DCMA 14-point / GAO ten-practices behaviour and the commercial audit
      tools; research: [`docs/research/schedule-audit.md`](docs/research/schedule-audit.md).
- [ ] **Optional AI integration** — off-by-default assistant that reads
      user-supplied PDFs/emails/documents and proposes programme advice,
      skeleton WBS/activities, activity enrichment and status updates as
      reviewable, undoable command proposals; bring-your-own provider, no
      network without consent. Research:
      [`docs/research/ai-integration.md`](docs/research/ai-integration.md).
- [ ] **Lookahead / field planning (Aphex-inspired)** — rolling short-interval
      windows derived from the master schedule, weekly work plans with
      owner/promise tracking, a daily diary and huddle briefing, production
      tracking, light location clash detection and published-snapshot
      comparison. **Done so far:** the window extraction
      (`cadence-core::lookahead`), the GUI **Lookahead** view (`Ctrl+7`) and
      owner/promise/delay-reason tracking (schema v5). **Remaining:**
      published snapshots, the diary, production tracking, clash detection.
      Research:
      [`docs/research/aphex-integration.md`](docs/research/aphex-integration.md).

## Out of scope (recorded in `docs/scope.md`)

Mobile platforms; any type of telemetry; proprietary file formats.
