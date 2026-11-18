# RP-5 — Keyboard and AccessKit pass for the Assistant view (harness assertions)

**Index:** RP-1 → RP-2 → RP-3 → RP-4 → RP-5 (this) → RP-6. Spec:
[`docs/specs/review-panel.spec.md`](../specs/review-panel.spec.md).

## Objective

Prove the Assistant view meets the shell's a11y bar with real assertions,
not prose: the tenth tab is reachable by name in the AccessKit tree, every
new control carries a non-empty accessible name in both themes across
frames, and the keyboard-only path through the flow (configure → request →
review → apply/discard) is complete. Update `ACCESSIBILITY.md` afterwards.

## Target file paths

- `crates/ui-egui/tests/a11y_harness.rs` (extend the existing harness: the
  view-tab fixture grows to ten; new tests for the Assistant states)
- `crates/ui-egui/src/lib.rs` (only focus/label fixes the assertions expose)
- `ACCESSIBILITY.md` (keyboard list gains the Assistant flow; the
  screen-reader notes gain the view)

## Blocking dependencies

- RP-4 (the review surface must exist to be asserted).

## Contract (from the spec — implement exactly this)

- The `VIEW_TABS` fixture becomes ten entries; the existing
  "exactly one tab per name" assertion covers Assistant.
- New assertions (all offline, via the established public seams):
  - **Enabling state:** with no config, the harness tree exposes the
    explanation text and every configuration control (provider, model,
    server address, key field, enable checkbox) as named inputs with their
    visible labels; the key field is password-masked in the render.
  - **Request state:** with `set_assistant_config`, the request field
    ("Describe the change you want") and the "Request proposal" button are
    labelled; the button is exposed as disabled while a request is in
    flight (seeded via a channel the test controls — no network).
  - **Review state:** with `set_assistant_proposal`, each change card
    exposes its heading, rationale text, selection checkbox (ticked state
    visible in the tree) and the "Apply selected changes" / "Discard
    proposal" buttons; with a validation-failing proposal, the error list is
    exposed as text and Apply is disabled.
- The hand-rolled named-input check (every visible input widget must carry a
  non-empty accessible name, both themes, two frames) passes with the new
  view present in all three states.
- Keyboard walk: documented in the PR (Tab order through the enabling form
  and the review surface; Space/Enter activates; no trapped focus) and any
  gap found is fixed in this ticket, not deferred.

## Verifiable acceptance criteria (CI-checkable)

- [ ] The extended harness passes on all four CI platforms (kittest 0.35.0,
      CPU renderer, `wgpu`/`snapshot` features still off).
- [ ] Every new assertion names its control via the visible label; no
      assertion depends on colour.
- [ ] `ACCESSIBILITY.md` updated: the keyboard list covers the Assistant
      flow; limits that egui cannot meet are recorded, not glossed over.
- [ ] No test touches the network (the update-check fixture pattern —
      `set_update_checks_enabled(false)` — is preserved).
- [ ] CI green: `Verify` on all four platforms, `Static analysis (rust)`,
      `Secret scanning`, `Personal data scan`.

## Out of scope

Rendered PNG artefacts (phase 2b, queued separately); contrast unit tests
beyond what the existing suite asserts; docs beyond `ACCESSIBILITY.md`
(RP-6).
