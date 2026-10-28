# Inspiration (Clean-Room Record)

Cadence is inspired only by **publicly documented concepts and observable
behaviour** of existing scheduling products. No proprietary code, resources,
icons, layouts, strings, or internal algorithms have been or will be
consulted.

## Reference products

| Product | What inspires us (public concepts only) |
|---|---|
| Microsoft Project | The idea of a combined activity table + Gantt view; baseline and variance workflow; earned-value reporting basics. Capability inventory: [`docs/research/ms-project-capabilities.md`](research/ms-project-capabilities.md) |
| Primavera P6 | Enterprise scheduling concepts: WBS hierarchy, activity codes, multiple calendars, resource leveling at scale. Capability inventory: [`docs/research/primavera-p6-capabilities.md`](research/primavera-p6-capabilities.md) |
| PMBOK (public standard) | CPM, precedence diagramming, float terminology, EVM formulas (SPI, CPI, SV, CV) |

## Future-expansion references (candidates — human gate pending)

These products are studied for **capabilities Cadence does not have yet**. Each
has a research note in `docs/research/` with original behaviour summaries and
public URLs, and none may be implemented until the owner approves the note
(guide §2.3). They are references for behaviour, never for code, UI, strings or
assets.

| Product | Capability the reference explains | Research note |
|---|---|---|
| Aphex | Short-interval/lookahead planning: rolling windows, weekly work plans, owner/promise tracking, daily diary and briefings, production tracking, location clash detection, master→lookahead bridge | [`docs/research/aphex-integration.md`](research/aphex-integration.md) |
| Oracle Primavera P6 (Check Schedule), Deltek Acumen Fuse, SSI Schedule Analyzer, Steelray Analyzer, Barbecana Schedule Inspector, ScheduleLens/SmartPM/Onplana | Schedule-quality and health auditing of a construction programme (DCMA 14-point, GAO ten best practices, AACE RPs): open ends, leads/lags, hard constraints, high/negative float, invalid dates, missed tasks, critical-path test, CPLI/BEI, forensic comparison | [`docs/research/schedule-audit.md`](research/schedule-audit.md) |
| nPlan, Microsoft Copilot for Project, Planera, ALICE, Trunk Tools, Document Crunch | Optional AI assistance: reading PDFs/emails/documents, programme advice, skeleton schedules, activity enrichment, draft status updates | [`docs/research/ai-integration.md`](research/ai-integration.md) |

DCMA (14-point assessment), GAO (Schedule Assessment Guide), AACE International
(recommended practices 49R-06, 92R-17, 53R-06, 29R-03, 57R-09) and PMBOK are
public standards/bodies; their concepts may be implemented from the public
standard, restated in Cadence's own words.

## Clean-room protocol

1. Feature inspiration is derived from public documentation, publicly known
   algorithms (CPM, precedence diagramming, EVM), and black-box observation
   of external behaviour only.
2. All data models, commands, UI layouts, interaction patterns, and visual
   design are original.
3. Every agent working on this repository must follow these rules; any
   proprietary material in a contribution invalidates it.
