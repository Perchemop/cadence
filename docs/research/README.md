# Research notes

Research for capabilities not yet implemented lives here as **original
behavioural summaries with public URLs**. The clean-room rules in `guide.md`
apply: no proprietary source, no decompilation, no pasted copyrighted text,
no pixel-copying of another product's UI — public documentation, public
specifications and observed behaviour only.

## Human gate

Per the guide (§2.3, §8.4): before a large implementation Task depends on a
note in this directory, a human reviews it and approves it. Rejected notes are
rewritten or deleted. Approved notes become the acceptance criteria for the
work.

## Current notes

Implemented (kept for the record of what was approved):

- [`printing-the-gantt-view.md`](printing-the-gantt-view.md) — capturing the
  rendered Gantt view and putting it into a PDF page (shipped as **Print
  Gantt…**).
- [`screen-reader-support.md`](screen-reader-support.md) — AccessKit
  integration through eframe (shipped; see `ACCESSIBILITY.md`).
- [`crash-report-submission.md`](crash-report-submission.md) — turning the
  local crash log into an optional GitHub Issue (shipped).
- [`update-check.md`](update-check.md) — checking GitHub Releases for a newer
  version, failing soft offline (shipped as `cadence check-updates`).

Approved by the owner (2026-10-27), in this queue:

1. [`schedule-audit.md`](schedule-audit.md) — an in-app tool that audits a
   construction programme (logic, float, constraints, dates, resources, baseline
   performance, critical path) against DCMA/GAO-style checks. **Next slice; in
   progress.**
2. [`aphex-integration.md`](aphex-integration.md) — short-interval/lookahead
   planning ideas (rolling windows, weekly work plans, diary, production
   tracking, location clashes) inspired by publicly documented behaviour.
3. [`ai-integration.md`](ai-integration.md) — optional, off-by-default AI
   assistance: document understanding, programme advice, skeleton schedules,
   activity enrichment and draft status updates, all via reviewable proposals.

Approval means the acceptance criteria in each note may drive implementation;
it does not commit a release date.

## Capability inventories (reference products)

Clean-room inventories of what the reference products do, for the parity
review (guide §II.27.3); summaries and URLs only:

- [`ms-project-capabilities.md`](ms-project-capabilities.md) — Microsoft
  Project: scheduling, baselines, resources, earned value, views and the
  Compare Projects diff, with what Cadence already ships and what is
  deliberately out of scope.
- [`primavera-p6-capabilities.md`](primavera-p6-capabilities.md) — Primavera
  P6: enterprise structure, activity types and codes, schedule log and health
  checks, Schedule Comparison, leveling, earned value periods, and the
  documented-vs-proprietary format split.
