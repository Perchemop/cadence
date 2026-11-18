# RP-1 — Assistant prompt builder, response parser and off-thread call seam

**Index:** RP-1 (this) → RP-2 → RP-3 → RP-4 → RP-5 → RP-6. Spec:
[`docs/specs/review-panel.spec.md`](../specs/review-panel.spec.md).

## Objective

Add the pure assistant-side half of the review-panel flow: build the minimal
working prompt from a live `Project`, parse a model completion into a
validated-shape `Proposal`, and define the off-thread call contract the UI
will follow. **No UI code, no network change, no settings change** — this
ticket touches `crates/assistant` only and ships headless tests.

## Target file paths

- `crates/assistant/src/prompt.rs` (new — `build_proposal_prompt`,
  `parse_proposal_response`, the activity-snapshot cap constant)
- `crates/assistant/src/lib.rs` (module declaration + `pub use` only)
- `crates/assistant/Cargo.toml` (no new dependencies expected; add one only
  if a test needs it, with a `DECISIONS.md` note)

## Blocking dependencies

None. First ticket of the series.

## Contract (from the spec — implement exactly this)

- `pub fn build_proposal_prompt(project: &Project, request: &str) -> String`
  — deterministic, pure; contains: project name + start date; per activity
  id/name/duration/percent; per relationship predecessor → successor; a hard
  cap of 500 activities with an explicit truncation notice past the cap; the
  user's request verbatim; the four `ProposedChange` variants with exact
  field names; the validation rules (no zero durations, progress 0–100,
  relationships reference existing activities, no self/duplicate links);
  every change carries a `rationale`; output is **one JSON object**
  `{"prose": …, "changes": […]}` with no surrounding commentary. Never
  contains the API key (it has no access to one — type-level guarantee).
- `pub fn parse_proposal_response(text: &str) -> Result<Proposal,
  AssistantError>` — accepts the bare JSON object or the same object inside
  one markdown code fence; uses the existing serde derives; rejects
  everything else as `AssistantError::MalformedOutput` with a plain-language
  detail (never a raw dump of the body).

## Verifiable acceptance criteria (CI-checkable)

- [ ] Prompt tests: contains the project snapshot fields for a fixture
      project; contains the request verbatim; names all four change variants
      and the `rationale` requirement; states the JSON contract.
- [ ] Truncation: a project with > 500 activities produces a prompt that
      says the activity list was truncated and contains at most the capped
      number of activity entries.
- [ ] Parse tests: bare JSON parses; fenced JSON parses; prose-only,
      empty, and non-JSON completions yield `MalformedOutput` (each with a
      non-empty plain-language detail); a JSON object missing `changes`
      yields `MalformedOutput`.
- [ ] Injection-shape test: a completion whose prose says "delete everything"
      but whose typed changes are valid parses into a normal `Proposal`
      (prose is never executed or interpreted).
- [ ] All tests are headless (no socket, no display) and green in CI
      (`Verify` on all four platforms); `Static analysis (rust)` clean;
      `Lockfile freshness` clean if the lockfile changes.
- [ ] No file outside `crates/assistant/**` and this ticket's tracking
      entries is modified.

## Out of scope

Settings storage (RP-2), any UI (RP-3+), prompt wording tuning beyond the
spec's fixed structure (owner-approved in the spec review).
