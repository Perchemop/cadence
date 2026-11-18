# RP-4 — Request flow, review rendering and apply/discard through the CommandStack

**Index:** RP-1 → RP-2 → RP-3 → RP-4 (this) → RP-5 → RP-6. Spec:
[`docs/specs/review-panel.spec.md`](../specs/review-panel.spec.md).

## Objective

Make the Assistant view live: the request button builds the prompt, calls the
provider **off the UI thread** (the update-check thread + mpsc pattern), the
reply is parsed and validated, the review panel renders prose plus one card
per `ProposedChange` (rationale always shown; before/after for sets), and
Apply executes the mapped commands through the real `CommandStack`. Discard
clears the panel.

## Target file paths

- `crates/ui-egui/src/lib.rs` (request/poll/apply/discard logic inside the
  Assistant view render method and `CadenceApp` methods)

## Blocking dependencies

- RP-1 (prompt builder + parser).
- RP-3 (view shell + state seams).

## Contract (from the spec — implement exactly this)

- **Off-thread call.** On "Request proposal": build
  `build_proposal_prompt(&self.project, &request)` on the UI thread (pure,
  instant); construct `HttpProvider::from_config(&self.assistant_config)`
  (fails closed with `NotConfigured` — that error becomes the status
  sentence, no thread spawned); clone the provider; spawn
  `std::thread` → `provider.complete(&prompt)` → send
  `Result<String, AssistantError>` through an `mpsc::channel`; store the
  receiver in the field RP-3 added. Per frame: `take()` the receiver,
  `try_recv()` — `Ok` advances to parse/validate; `Empty` puts the receiver
  back and `ctx.request_repaint_after(Duration::from_millis(250))`;
  `Disconnected` becomes the "stopped unexpectedly" status sentence. While
  waiting, the request button is disabled.
- **Arrival.** `Ok(text)` → `parse_proposal_response` → on `Err`, the
  MalformedOutput status sentence; on `Ok(proposal)`,
  `proposal.validate(&self.project)` — errors are listed in the panel (all
  at once) *and* summarized in the status bar (`Error` severity); a clean
  proposal enters the review state (checkboxes ticked by default, per spec
  Q3 recommendation (a)).
- **Review rendering.** Prose block first; then one card per change: change
  kind, its fields, its `rationale` (always rendered), before/after values
  read from the live project for `SetActivityDuration` /
  `SetActivityProgress`, "New" text markers (not colour-only) for created
  activities/relationships, a selection checkbox per card. Buttons:
  "Apply selected changes" and "Discard proposal". A new request while a
  proposal is under review requires an inline confirm (ordinary
  confirm-flow pattern).
- **Apply.** Build the subset `Proposal` from the ticked changes (prose
  kept, rationales preserved); `map_proposal_to_commands(&self.project,
  &subset)` (re-validates, all-or-nothing); on error, the mapped
  `ProposalError` status sentence and nothing applied; on success, execute
  each command through `self.stack` (`stack.execute(&mut self.project,
  cmd)`), then `recompute()` once, then
  `set_status("N changes applied from the proposal.")`, and clear the review
  state. Undo/redo need **no new code** — the ordinary stack paths, already
  proven by the landed `command_map` tests.
- **Discard.** Clears the review state; a status sentence confirms.
- **Error-sentence mapping** exactly as the spec's table (NotConfigured /
  Refused / Network / MalformedOutput / the seven `ProposalError` variants).

## Verifiable acceptance criteria (CI-checkable)

- [ ] Headless flow test (no socket): with a configured config and
      `set_assistant_proposal` seeding a valid proposal, the review panel
      renders prose, per-change rationales and before/after values in both
      themes.
- [ ] Headless apply test: applying a seeded proposal's changes through the
      render path leaves the project with the expected activities /
      relationships / durations / progress, the status message set, and
      **undo restores the previous state** (driven through the real
      `CommandStack`).
- [ ] Headless partial-apply test: deselecting one change applies only the
      rest; a subset referencing an activity created by an unselected change
      is refused with the `UnknownActivity` sentence and nothing applied.
- [ ] Headless invalid-proposal test: a seeded proposal that fails
      validation renders every error and disables Apply.
- [ ] Error-mapping tests: each `AssistantError` and `ProposalError` variant
      produces its spec-table sentence (pure mapping function, unit-tested).
- [ ] No test opens a socket; the provider call path itself is exercised
      only through the offline seams (the thread+channel plumbing is covered
      by a test that sends a value through a channel and asserts the poll
      consumes it — no network).
- [ ] The raw-colour grep gate passes (tokens only); all new status
      sentences are classified by the existing `StatusSeverity` rules or the
      mapping test pins their severity.
- [ ] CI green: `Verify` on all four platforms, `Static analysis (rust)`,
      `Lockfile freshness`, `Secret scanning`, `Personal data scan`.

## Out of scope

Harness/keyboard assertions (RP-5); docs (RP-6); cancellation (spec Q8);
conversation history, streaming, new change variants (non-goals).
