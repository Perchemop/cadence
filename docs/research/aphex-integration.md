# Research — Aphex (short-interval / lookahead planning) and what Cadence can borrow

Status: draft, not human-approved. Large implementation must not depend on this
note until the owner approves it (guide §2.3, §8.4).

## What Aphex is (public behaviour only)

Aphex is a construction delivery-planning platform for **short-interval
planning**: it takes the high-level master schedule and turns it into live
weekly and daily site work, then feeds what actually happened back to the plan.
Its own framing is "one live plan" that removes the drift between disconnected
spreadsheets. Summaries below are original wording from its public site/help;
URLs are in Sources.

## Features observed (public)

- **Field scheduling**
  - *Lookaheads* — plan upcoming work over a rolling window (commonly 3–12
    weeks).
  - *Weekly work plans* — keep the near-term plan live as work unfolds.
  - *Resource planning* — forecast and allocate labour, plant and materials.
  - *Staging diagrams* — map work with auto-built diagrams.
- **Daily operations**
  - *Daily meetings/huddles* — structured daily coordination from the live plan.
  - *Briefings* — brief crews directly from the plan.
  - *Production tracking* — track production against the plan.
  - *Daily diary* — dated site records linked to planned work.
- **Views and distribution**: map, board, list and Gantt views; save and share
  custom views by secure link or print to PDF; group/sort/filter printouts.
- **Collaboration**: multi-user editing on one plan without overwriting;
  task owners and subcontractors per task; comments and notifications.
- **Planning detail**: calendars, shifts, subtasks, blockers, mandatory fields
  before publishing, and "promises" on imminent tasks (with a delay reason
  required to move a promised task, and variation tracking).
- **Clash detection**: flags when two activities are planned on the same day in
  the same location, with in-app mention/notification.
- **Master-schedule bridge**: imports tasks, structure and metadata (WBS,
  calendar, relationships, UDFs and owner/subcontractor/location attributes)
  from Primavera P6, Asta Powerproject or Microsoft Project, keeps the lookahead
  aligned with the master, and can feed published reality back.
- **Baseline/intra-plan comparison**: compare the latest published lookahead
  with previous versions and with the master schedule.
- **Integrations**: ArcGIS for spatial data, Power BI for dashboards, and an
  API on the higher tier.
- **AI**: placed inside the Diary product to make sense of site records, rather
  than as a standalone chatbot; the vendor states a deliberate governance and
  risk stance.

## What Cadence could borrow (original design direction)

None of these copy Aphex's UI or cloud; they are original features inspired by
the observable behaviour, and they stay inside Cadence's offline-first model.

1. **Lookahead view derived from the master schedule.** A rolling window
   (configurable 2–12 weeks) that filters the programme to near-term activities,
   with the ability to freeze/publish a snapshot. It is purely a view over the
   existing schedule plus a saved window — no new scheduling engine.
2. **Weekly work plan.** Assign an owner and a promise flag to each activity in
   the window; require a delay reason to move a promised activity; keep the
   previous published version to compare.
3. **Daily diary / huddle view.** Dated notes linked to activities or to the
   lookahead window (with an optional app-level view and printable briefing
   sheet). Reuses the crash-report lesson: capture locally, export deliberately.
4. **Production tracking.** Record daily actual progress (percent or quantity)
   against the lookahead and chart plan-versus-actual per activity, feeding the
   existing progress/percent-complete model and the audit tool's
   out-of-sequence and missed-task checks.
5. **Location/area tags and a simple clash view.** Optional free-text location
   (or area) on activities, with a check that flags two activities planned on
   the same day at the same location. This is a light, dependency-free version
   of Aphex's clash detection; a real map/GIS layer is out of scope for v1.
6. **Baseline comparison of lookaheads.** Compare a published lookahead against
   the previous one and against the baseline, reusing the existing Baselines/
   variance machinery.
7. **Publish and share.** Export a lookahead/report to PDF or a self-contained
   shareable file (Cadence already prints PDF); a hosted read-only link is a
   cloud feature and stays out of scope.
8. **Master-schedule bridge.** Cadence already imports/exports MSPDI and MPX;
   the Aphex idea confirms that importing WBS, calendar and relationship data
   (not just a flat task list) is what makes a master→lookahead workflow real.
   Use the existing interchange rather than a proprietary Aphex connector.

Possible later, consent-gated surface (recorded, not planned): an optional API
or sync so a team on a master-scheduling tool and a field-planning tool can
exchange published reality. This would be a separate cloud/network decision and
must not become a dependency of the offline core.

## Acceptance criteria (for the first Cadence slice, if approved)

1. A **Lookahead** view shows only activities starting (or in progress) in a
   configurable rolling window, driven from the master schedule with no
   duplicate data.
2. A window can be published as a named, read-only snapshot; a later window can
   be compared with it.
3. An activity in the window can carry an owner and a promise; moving a promised
   activity requires a delay reason that is stored and visible.
4. Daily notes can be attached to an activity or a window and exported with the
   lookahead.
5. Everything is offline and undoable; no account, no cloud.

## Edge cases to design for

- Activities longer than the window (start before, finish after) and milestones.
- Multiple calendars and shifts; a "day" in a lookahead is a working day on the
  activity's calendar.
- A promise moved repeatedly, or a delay reason edited later; keep the history
  honest.
- Location free text becomes inconsistent; keep it optional and unvalidated in
  the light version.
- Published snapshots must be immutable so comparisons stay meaningful.
- Do not let the lookahead become a second, divergent schedule: it is always
  derived from the master unless the user explicitly promotes a change through
  the command layer.

## Clean-room notes

- Aphex is a commercial product; only its public website, help pages and public
  blog posts were read. Nothing is copied: no UI, strings, diagrams, code or
  data model. Cadence's lookahead/diary/clash design is our own, drawn from
  general short-interval-planning practice (also a long-standing construction
  method) and the observable Aphex workflow.
- Do not build an Aphex connector or claim compatibility.

## Sources

- Aphex home and product pages: https://www.aphex.co/ , https://www.aphex.co/for/planners , https://www.aphex.co/product/plan-together , https://www.aphex.co/product/build-routine , https://www.aphex.co/product/simplify-reporting
- Aphex lookahead explainer: https://www.aphex.co/updates/what-is-a-lookahead-plan
- Aphex master→lookahead import approach: https://blog.aphex.co/post/importingplans
- Aphex Australia overview (views, mapping, clash detection, distribution): https://blog.aphex.co/construction-scheduling-software-australia
- Aphex integrations, pricing tiers, API/ArcGIS/Power BI: https://blog.aphex.co/excel-vs.-aphex
- Aphex on how it uses AI: https://help.aphex.co/docs/policies/how-aphex-uses-ai
