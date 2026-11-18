# Documentation reference index

Pre-loading index for agent teams (guide §5.7.2): one line per file in the
repository's `docs/` tree, so teammates query this locally instead of
re-reading raw source. **Regenerated when docs change (guide §5.7.2).**

## Product & scope

- `docs/project-brief.md` — what Cadence is: mission, primary user, and the problem it solves.
- `docs/scope.md` — in-scope and explicitly deferred capabilities for the current phase.
- `docs/product-name.md` — the naming decision record for "Cadence".
- `docs/inspiration.md` — clean-room record of the inspiration products and what may (and may not) be borrowed.
- `docs/architecture.md` — workspace layering (core / commands / persistence / app) and the scheduling engine's conventions.
- `docs/usage.md` — end-user guide: how to drive the app day to day.
- `docs/development.md` — contributor build/test instructions; CI is authoritative, nothing runs locally.
- `docs/implementation-report.md` — evidence-based summary of what is implemented, how it is verified, and what is deliberately open.
- `docs/assumptions-and-questions.md` — facts, assumptions, and open questions awaiting owner decisions.

## Research (clean-room: original summaries + URLs only)

- `docs/research/README.md` — the rules every research note follows.
- `docs/research/ai-integration.md` — owner-approved research behind the optional AI assistant (providers, privacy, scope).
- `docs/research/aphex-integration.md` — what Aphex's short-interval / lookahead planning does publicly and what Cadence may borrow.
- `docs/research/crash-report-submission.md` — how crash-report submission works and exactly what is sent.
- `docs/research/gui-inspiration-audit.md` — GUI inspiration audit of Cadence against industry standards.
- `docs/research/ms-project-capabilities.md` — public capability inventory of Microsoft Project.
- `docs/research/primavera-p6-capabilities.md` — public capability inventory of Primavera P6.
- `docs/research/printing-the-gantt-view.md` — research behind Gantt printing / PDF export.
- `docs/research/schedule-audit.md` — research for the programme audit tool (schedule quality and health).
- `docs/research/screen-reader-support.md` — AccessKit / screen-reader support research and its recorded limits.
- `docs/research/update-check.md` — research behind the opt-in update check.

## Design

- `docs/design/brand.md` — Cadence's original brand: colours, type, iconography, voice.
- `docs/design/gui-stack.md` — the II.30.2 stack record: resolved egui/eframe versions, shell shape, layout model.
- `docs/design/elite-ui-ux-uplift.md` — investigation behind the paid-tool-grade UI/UX uplift.
- `docs/design/paid-tool-ui-uplift.md` — focused, concrete UI uplift proposals.
- `docs/design/ui-polish-audit.md` — UI polish audit and its remediation plan.
- `docs/ui-patterns.md` — clean-room UI pattern matrix from publicly documented desktop schedulers.

## Audits & quality gates

- `docs/audit/2026-11-01-design-uplift-research.md` — free/open-source design resources surveyed for the uplift.
- `docs/audit/2026-11-01-gantt-gap-analysis.md` — Cadence-vs-MS-Project/P6 Gantt gap analysis.
- `docs/audit/2026-11-01-remediation-plan.md` — verdict and remediation plan from the end-to-end audit.
- `docs/audit/2026-11-01-ui-surface-inventory.md` — inventory of every UI surface in the egui shell.
- `docs/audit/2026-11-01-ux-critique.md` — UX critique of the desktop app.
- `docs/audit/2026-11-07-full-visual-functional-audit.md` — full visual and functional audit at v0.12.1.
- `docs/audit/2026-11-08-elite-tool-uplift-plan.md` — the plan to move from "engineer-built" to "studio-polished".
- `docs/ELITE_TOOL_UPLIFT_SUMMARY.md` — summary of the elite-tool uplift investigation (2026-11-08).
- `docs/qc/phase-1.md` — Phase 1 quality gate results.
- `docs/qc/phase-2.md` — Phase 2 quality gate results.
- `docs/qc/phase-3.md` — Phase 3 quality gate results.
- `docs/qc/phase-5.md` — Phase 5 quality gate results.
- `docs/qc/project-health.md` — project-health quality gate (the opensource.guide items).
- `docs/qc/release-v0.1.0.md` — v0.1.0 release legal and quality checklist.
- `docs/qc/app-audit.md` — whole-app end-to-end audit.
- `docs/qc/capability-audit.md` — capability audit against the reference products.
- `docs/qc/gui-audit.md` — GUI audit of the egui desktop shell.
- `docs/security-audit-2026-11.md` — security and quality audit, November 2026.

## Process & records

- `docs/current-model-map.md` — which free model fills which agent role right now (guide §5.2), plus the guide-sync record.
- `docs/parity.md` — capability parity tracker (`todo` / `in progress` / `done`) with evidence.
- `docs/process-feedback.md` — append-only agent friction log (addendum N).
- `docs/repo-radar.md` — periodic build-vs-adopt pass over public GitHub projects (guide §4A).
- `docs/tokens.json` — CI-managed token-usage ledger (guide §12.1); release notes cite it rather than estimates.
