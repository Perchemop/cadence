# Vision

## In one paragraph

Cadence is a free, offline scheduling application for construction and
capital-project work. It does what professional scheduling tools do — work
breakdown structures, dependency logic, critical-path calculations, resources,
calendars, baselines and earned value — without a subscription, without an
account, and without sending your project anywhere. It exists so that a
planner whose employer will not buy or host a commercial tool can still produce
a defensible schedule, on a laptop, with a file they own.

## Who it is for

- Planners and project-controls engineers on construction and capital projects.
- Small consultancies and site teams with no enterprise scheduling licence.
- Anyone who must keep project data on their own machine.

## What makes it different

- **Genuinely offline.** No telemetry, no account, no network call in the
  scheduling, persistence, reporting or interchange paths.
- **Open file formats.** Files are documented and readable: a versioned SQLite
  project file, MSPDI (Microsoft Project XML) and MPX import/export, plus
  Markdown and PDF reports. Nothing is locked behind a proprietary reader.
- **Original and open source.** A clean-room implementation under MIT OR
  Apache-2.0, so it can be audited, extended and embedded.

## Goals

1. A complete, correct CPM engine: full precedence logic (FS/SS/FF/SF with
   lag), constraints, calendars, float and critical path.
2. A usable desktop UI for that engine: activity table, Gantt, resources,
   baselines, calendars.
3. Interchange and reporting that a project controls team can actually send to
   someone else.
4. Honest behaviour: the schedule never invents data, and every limitation is
   documented.

## Non-goals

Explicitly *not* planned, so that "no" stays impersonal (see
[`GOVERNANCE.md`](GOVERNANCE.md)):

- Cloud collaboration, multi-user editing, or a hosted service.
- Mobile or web clients.
- Enterprise resource planning, cost accounting or procurement.
- Oracles, solvers, or optimising (rather than greedy) resource leveling.
- Macros, scripting or a plugin SDK.
- Reading or writing proprietary/project-file formats that require reverse
  engineering — see the clean-room rule in [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Roadmap

[`ROADMAP.md`](ROADMAP.md) is the living plan: stage labels, feature breadth,
ready-for-work markers and semver milestones. This document states intent;
the roadmap states current readiness, and the roadmap wins on detail.

## How to grow into leadership, and how decisions are made

See [`GOVERNANCE.md`](GOVERNANCE.md).

## Open questions recorded for the maintainer

These are deliberately unresolved, because only the human owner can settle
them:

| # | Question | Why it matters |
|---|---|---|
| N1 | **Name conflict.** "Cadence" is also the name of a large EDA company (Cadence Design Systems), and the word appears in other software. Has the name been checked against the [WIPO Global Brand Database](http://www.wipo.int/branddb/en/) and package registries? | A later takedown request is expensive; a rename later is worse |
| N2 | **Canonical home URL and handles.** Which domain/handle is the single canonical home for the project (GitHub Pages or otherwise)? | Marketing and update-check endpoints need one stable URL |
| N3 | **Employer / IP policy.** Has the maintainer's employer policy been checked for rights over this work (third-party material, trade secrets, patents, trademarks, data, AI-model terms, SBOM obligations)? | The answer decides whether the work can be licensed and published as it stands; recorded in [`docs/assumptions-and-questions.md`](docs/assumptions-and-questions.md) |
| N4 | **Admin bus factor.** At least two admins with multi-factor authentication are wanted, for least privilege and continuity. Is there a second trusted admin? | Losing sole access loses the project |
| N5 | **MFA and account security.** Is multi-factor authentication enabled on the owner account and any account with write access? | Account takeover of a maintainer is the highest-impact attack on a project; it can put malware into a release |
