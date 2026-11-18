# RP-6 — Usage docs, shortcuts and tracking for the Assistant review panel

**Index:** RP-1 → RP-2 → RP-3 → RP-4 → RP-5 → RP-6 (this). Spec:
[`docs/specs/review-panel.spec.md`](../specs/review-panel.spec.md).

## Objective

Document the shipped feature for a non-technical reader and close the
slice's tracking records. The README privacy note for the opt-in provider
calls already landed with the HTTP slice — this ticket verifies it still
matches the shipped behaviour rather than rewriting it.

## Target file paths

- `docs/usage.md` (new "Assistant proposals" section + shortcuts table row
  if Q2 adopted a chord)
- `README.md` (verify the existing privacy note matches the shipped flow;
  adjust only if it drifts)
- `ROADMAP.md` (AI bullet: review-panel stage marked landed with evidence)
- `PROGRESS.md` (dated entry: what landed, CI run IDs)
- `CHANGELOG.md` (Unreleased/Added: the Assistant review panel)
- `REQUIREMENTS_TRACEABILITY.md` (the review-panel requirement IDs → DONE
  with CI evidence)
- `docs/parity.md` / `docs/capability-inventory.md` (the review-panel
  capability rows updated with evidence)

## Blocking dependencies

- RP-4 and RP-5 (documents describe shipped, verified behaviour only —
  never intent).

## Contract

- **usage.md section, plain language:** what the assistant review panel is;
  that it is optional and off until configured; how to enable it (the
  enabling form); what a request sends (project snapshot + the request, to
  the configured provider — no other data, no telemetry); that nothing
  changes until Apply; that undo works exactly as for manual edits; the
  error sentences a user may see; that the API key is never stored on disk
  and must be re-entered after a restart (spec Q4 recommendation (a)).
- **Shortcuts table:** gains the Assistant view row only if Q2 adopted a
  global chord; otherwise the View-menu/tab-strip path is documented.
- **No claims beyond evidence:** the section describes only what the
  landed tickets verify; future stages (citations, document upload,
  skeletons) are named as "not yet", or omitted entirely.

## Verifiable acceptance criteria (CI-checkable)

- [ ] Every user-facing claim in the new docs is traceable to a landed test
      or an existing invariant; the PR lists claim → evidence pairs.
- [ ] `docs/usage.md` renders sensibly (tables intact; links resolve).
- [ ] ROADMAP / PROGRESS / CHANGELOG / traceability updated with the green
      CI run IDs of RP-1..RP-5.
- [ ] `Personal data scan` and `Secret scanning` green (docs contain no
      keys, no personal data — example values are obviously synthetic).
- [ ] No code file modified in this ticket (docs + tracking only).

## Out of scope

Release cutting (the maintainer tags the next semver once the slice is
merged); README screenshots (v1.0 / major-release gate per suite standards).
