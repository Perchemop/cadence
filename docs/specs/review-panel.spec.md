# Specification — Assistant review panel (the last assistant stage)

Status: **drafted for owner review — no implementation has started.** This is a
planning document only (guide §5.6.3, `/to-spec`). Every decision the
codebase could not answer is listed under [Open questions for the owner](#open-questions-for-the-owner)
with a recommendation; the owner approves the spec before ticket RP-1 starts
(human gate, §5.6.3 / §2.3 discipline).

Sources this spec is grounded in (all read at branch tip `7755c92`):

- [`docs/research/ai-integration.md`](../research/ai-integration.md) —
  owner-approved research. The governing rule: **the assistant proposes, the
  planner disposes.** Proposals are shown as reviewable diffs and applied
  through the ordinary command layer; nothing is ever edited silently.
- `crates/assistant/src/proposal.rs` — `Proposal`, `ProposedChange`
  (create activity / create relationship / set duration / set progress, each
  with a `rationale`), `ProposalError`, and `Proposal::validate` which returns
  *all* problems at once against the live project.
- `crates/assistant/src/command_map.rs` — `map_proposal_to_commands`, the
  landed, headless-tested translation from a validated proposal to boxed
  `cadence-commands::Command` values. It re-validates and is all-or-nothing
  (never a partial command list). Its tests assert undo walks backwards
  **one change at a time**.
- `crates/assistant/src/http.rs` + `config.rs` — `HttpProvider::from_config`
  (fails closed with `NotConfigured`), `AssistantConfig::is_configured`,
  `AssistantError::{NotConfigured, Refused, MalformedOutput, Network}`,
  `SecretKey` which redacts itself in `Debug` and serialises as the literal
  `"<redacted>"` (a settings file therefore **cannot** round-trip a key).
- `crates/commands/src/lib.rs` — the `Command` trait and `CommandStack`
  (`execute`/`undo`/`redo`, validate-before-mutate).
- `crates/ui-egui/src/lib.rs` — the shell patterns described below (patterns,
  not line numbers: line references do not survive past the branch tip).
- `docs/design/gui-stack.md`, `ACCESSIBILITY.md`,
  `docs/audit/2026-11-07-full-visual-functional-audit.md` — keyboard,
  AccessKit and dialog/state expectations.
- `ROADMAP.md` AI bullet — "Next: the review panel that shows a proposal,
  cites its rationales and applies it on acceptance."

---

## System architecture

### View placement: a tenth centre view tab (recommended)

**Recommendation: the Assistant becomes the tenth view in the existing
centre-view strip, not a right-side panel.** The argument from the codebase's
own patterns:

- The shell is a fixed three-pane layout: a top bar (menu bar → view-tab strip
  → tool cluster), a bottom status bar, right-hand inspector panels, and one
  active centre view. Nine views exist today — Table (`Ctrl+1`), Gantt
  (`Ctrl+2`), Resources (`Ctrl+3`), Baselines (`Ctrl+4`), Calendars
  (`Ctrl+5`), Audit (`Ctrl+6`), Lookahead (`Ctrl+7`), Diary (`Ctrl+8`),
  Production (`Ctrl+9`) — each a variant of the shell's private view-mode
  enum, a labelled tab button, a View-menu radio item, and a render method.
- The right side is **already owned** by per-view inspectors (the Table,
  Gantt, Resources, Baselines and Calendars inspectors) plus the optional
  split-Gantt second right panel. A persistent assistant panel would compete
  with every inspector for width and break the "one inspector per view"
  reading of the shell.
- The closest existing analogue to the review workflow is the **Audit view**
  (`Ctrl+6`): a stage-based centre view with a run action, per-check blocks
  and jump-to-detail. The review panel has the same shape — a request stage,
  per-change blocks with rationales, apply/discard actions — and needs the
  full centre width to lay out change cards readably.
- A tenth view slot is free: `Ctrl+1..9` are all taken, and the enum, the tab
  list, the View menu and the shortcut map are all designed around an
  enumerable set of views.

Placement decision is [open question Q1](#open-questions-for-the-owner); the
tickets below are written against the tenth-tab recommendation and would be
re-shaped if the owner picks the panel.

### The request/review flow

```
user types a request and presses "Request proposal"   (explicit action only)
        │
        ▼
UI thread: build_proposal_prompt(&self.project, &request)   (pure, instant)
UI thread: HttpProvider::from_config(&self.assistant_config)  (fails closed)
        │
        ▼
std::thread::spawn:  provider.complete(&prompt)               (blocking call,
        │       │                                             5 s connect /
        │       │                                             120 s read cap)
        │       └── mpsc::Sender<Result<String, AssistantError>>
        ▼
UI thread, each frame: receiver.try_recv()          (same poll pattern as the
        │                                            update check; repaint
        │                                            requested after 250 ms
        │                                            while the thread runs)
        ▼
on Ok(text):  parse_proposal_response(&text)  →  Result<Proposal, AssistantError>
        │
        ▼
UI thread: proposal.validate(&self.project)  →  Vec<ProposalError>  (all at once)
        │
        ├─ errors → review panel lists every problem; nothing is appliable
        └─ clean  → review panel renders prose + one card per ProposedChange
                    (rationale, before/after for sets) with apply/discard
        │
        ▼
"Apply selected changes":
        build a subset Proposal from the ticked changes
        →  map_proposal_to_commands(&self.project, &subset)   (re-validates,
        │                                                      all-or-nothing)
        →  for each command: stack.execute(&mut self.project, cmd)
        →  recompute()  →  set_status("N changes applied from the proposal.")
        (every applied change is an ordinary undoable history entry;
         Ctrl+Z works exactly as for a manual edit)
```

Design rules the flow enforces:

- **The provider call never runs on the UI thread.** `AssistantProvider` is a
  synchronous trait (`complete` blocks; the HTTP layer caps the wait at a
  5-second connect and a 120-second read), and the egui shell repaints every
  frame. The pattern is the one the update check already uses: an
  `Option<mpsc::Receiver<…>>` field, a spawned `std::thread` that sends one
  result, and a per-frame `try_recv` poll that requests a 250 ms delayed
  repaint while the thread is running. A disconnected channel becomes a
  plain-language status message, exactly as the update poller treats a
  stopped checker.
- **The prompt is built on the UI thread before spawning.** Building it is a
  pure, instant function over `&Project`; only the network call is slow, so
  only the network call moves off-thread. The `Project` itself is never sent
  to another thread — the thread receives an owned `String` prompt and a
  cloned provider (`AssistantProvider: Send + Sync` is already the trait's
  contract).
- **Validation runs twice by design.** `parse → Proposal::validate` on
  arrival decides what the review panel may offer; `map_proposal_to_commands`
  re-validates at apply time against the *live* project (the project may have
  changed between the proposal arriving and the user pressing Apply — manual
  edits, undo). Identifiers for created activities are therefore always the
  next free ones at the moment of application, never identifiers guessed
  when the model answered.
- **One proposal at a time.** A new request replaces any proposal still under
  review after an inline confirm (a proposal that has not been applied or
  discarded is unsaved work in the panel's own terms). While a request is in
  flight the request button is disabled and the panel shows a "Waiting for
  the provider…" state.

### The prompt (minimal working prompt, no tuning)

`build_proposal_prompt` produces one deterministic message containing:

1. A compact project snapshot: project name and start date; per activity its
   id, name, duration and percent complete; per relationship
   predecessor → successor. A hard cap (500 activities) bounds the snapshot;
   past the cap the prompt says the list is truncated, so the model cannot
   invent ids it never saw.
2. The user's request, verbatim.
3. The change vocabulary: the four `ProposedChange` variants with their exact
   field names, the rule that every change carries a `rationale`, and the
   rules already enforced by validation (no zero durations — zero is a
   milestone; progress 0–100; relationships must reference existing
   activities and must not duplicate or self-link).
4. The output contract: **one JSON object** `{"prose": …, "changes": […]}`,
   no surrounding commentary.

`parse_proposal_response` accepts the JSON object, tolerating the common
markdown code-fence wrapper, and rejects everything else as
`AssistantError::MalformedOutput` with a plain-language detail. It uses the
existing serde derives on `Proposal`; it never evaluates code or interprets
prose as instructions — the typed-change schema *is* the injection defence,
together with the review gate and command validation.

### Off-by-default visibility

`AssistantConfig::is_configured` gates the feature. The recommendation
([open question Q7](#open-questions-for-the-owner)):

- The **Assistant tab is always present**, because research acceptance
  criterion 1 requires the UI to *show how to enable it*.
- When unconfigured, the view renders the **enabling state**: a short
  explanation (what the assistant is, that it is optional, that requests send
  project data to the configured provider) and the configuration form
  (provider name, model, optional server address, API key, enable checkbox).
  No other view changes; no network path exists in this state.
- When configured, the enabling state is replaced by the request + review
  surface. The configuration form stays reachable (collapsed section) so the
  provider can be changed or the feature switched off.

### Error surfaces (plain-language status sentences)

Every failure lands in the existing status bar via `set_status`, classified
`Error` severity (so it persists until replaced, matching the shell's
status-message behaviour). The mapping:

| Failure | Status sentence |
|---|---|
| `AssistantError::NotConfigured` | "The assistant is not configured. Set a provider and model in the Assistant view." |
| `AssistantError::Refused` | "The provider refused the request. Check the provider name, model and API key in the Assistant view." |
| `AssistantError::Network(detail)` | "Could not reach the provider: {detail}. Check the connection and try again." |
| `AssistantError::MalformedOutput(detail)` | "The provider's answer could not be read as a proposal: {detail}. Try rephrasing the request." |
| `ProposalError::UnknownActivity { id }` | "The proposal names activity {id}, which does not exist in the programme." |
| `ProposalError::PercentOutOfRange { percent }` | "The proposal sets progress to {percent}%, which is above 100." |
| `ProposalError::SelfRelationship { id }` | "The proposal links activity {id} to itself." |
| `ProposalError::DuplicateRelationship` | "The proposal repeats a relationship that already exists." |
| `ProposalError::EmptyName` | "The proposal contains a blank activity name." |
| `ProposalError::ZeroDuration` | "The proposal sets a duration of zero, which is reserved for milestones." |
| `ProposalError::NoChanges` | "The proposal contains no changes." |

Validation problems are additionally listed inside the review panel (all of
them at once — `Proposal::validate` already returns the full list), so the
user can read them beside the change cards, not only in the status bar.

---

## Interface contracts

### New seams in `crates/assistant` (new module `prompt.rs`)

```rust
/// Build the minimal working prompt for one proposal request, from the live
/// project and the user's request. Pure; deterministic for a given project
/// and request. Contains project data only — never the API key.
pub fn build_proposal_prompt(project: &Project, request: &str) -> String;

/// Parse a model completion into a `Proposal`. Accepts the bare JSON object
/// or the same object inside one markdown code fence; anything else is
/// `AssistantError::MalformedOutput` with a plain-language detail.
/// Never runs a command, never touches a project.
pub fn parse_proposal_response(text: &str) -> Result<Proposal, AssistantError>;
```

Both are pure functions over data, so they are verified headlessly (K.13):
prompt-shape tests, truncation-cap tests, parse tests for each wire's
realistic shapes (bare JSON, fenced JSON, prose-only answer, empty answer,
invalid JSON), and an injection-shaped test (a completion whose prose says
"delete everything" but whose typed changes are valid is still just a
proposal for review — prose is never executed).

### Settings storage (new module `crates/assistant/src/settings.rs`)

Verified fact: **no settings storage exists in the repository today.** The
persistence crate is the project-file (SQLite) layer only, and the research
note forbids the key in the project file anyway. The settings module
therefore lives in `cadence-assistant` and is the only new state this feature
adds outside the UI:

```rust
/// The user-level settings file path (platform config directory:
/// XDG_CONFIG_HOME / HOME on Linux, APPDATA on Windows,
/// HOME/Library/Application Support on macOS), or `None` when the
/// environment provides none. Hand-rolled platform lookup — no new
/// dependency.
pub fn settings_path() -> Option<PathBuf>;

/// Load the assistant settings; any read or parse failure yields the
/// default (off) config. Never fails the caller.
pub fn load_settings() -> AssistantConfig;

/// Store the assistant settings, **without the API key**. The key is
/// session-only: `SecretKey`'s serialiser deliberately writes
/// `"<redacted>"`, so a settings file cannot round-trip a key, and the
/// module honours that invariant rather than working around it.
/// Errors are returned as plain-language strings for the status bar.
pub fn store_settings(config: &AssistantConfig) -> Result<(), String>;
```

Acceptance includes a test proving a stored-then-loaded config has
`api_key == None` while `enabled`, `provider_name`, `model` and `base_url`
round-trip, and a legacy test proving a pre-existing file with a
`"<redacted>"` placeholder still loads (the existing config test already
fixes that serde shape).

The `cadence-assistant` workspace-dependency entry is re-added for
`cadence-ui-egui` in ticket RP-3 (it was removed under the dead-code rule
when nothing consumed the crate; it returns exactly when a consumer lands).

### UI seams in `crates/ui-egui`

- `ViewMode` gains an `Assistant` variant (or whatever shape Q1 settles on);
  the tab strip, the View-menu radio list and the view-switch match each gain
  the new entry. The tab label follows the existing pattern — icon glyph,
  space, name — and the name includes the shortcut only if one is adopted.
- `CadenceApp` gains private state: the loaded `AssistantConfig`, the request
  text buffer, the in-flight receiver
  (`Option<mpsc::Receiver<Result<String, AssistantError>>>`), the proposal
  under review with its per-change selection flags, and the current
  validation-error list (recomputed before apply and after project edits).
- Two public, documented offline-support seams, following the existing
  `set_update_checks_enabled` precedent (the harness must never touch the
  network):
  - `set_assistant_config(&mut self, config: AssistantConfig)` — installs a
    configuration without the settings file (tests, and a future CLI flag);
  - `set_assistant_proposal(&mut self, proposal: Proposal)` — loads a
    proposal into the review state so the review surface is drivable by the
    kittest harness and the headless render tests without any network.
- Apply path: `map_proposal_to_commands` (already public) is called at apply
  time; commands are executed one at a time through the real `CommandStack`;
  `recompute()` runs once after the batch; a `set_status` sentence reports
  the outcome. Undo/redo need no new code — they are the ordinary stack
  paths, which the landed `command_map` tests already exercise end-to-end.

### What is deliberately *not* a seam

- No assistant code in `cadence-core`, `cadence-commands` or
  `cadence-persistence`. The engine, the command layer and project files stay
  deterministic and assistant-unaware; the only touchpoint is that proposals
  become ordinary commands, which is the landed `command_map` module.
- No new `ProposedChange` variants. The four landed variants are the whole
  vocabulary for this slice; more (actual dates, owners, WBS) are future
  slices with their own validation.

---

## Non-goals

- **No automatic network calls.** The provider is contacted only when the
  user presses the request button. No background pre-fetch, no retry loop, no
  telemetry, no update-tied behaviour (the research note's hard rule).
- **No assistant-initiated edits.** The assistant can only fill the review
  panel. It cannot execute commands, schedule, save, or touch the project
  outside an explicit Apply.
- **No conversation history.** Each request is single-shot: prompt in,
  proposal out. Follow-up questions start a new request built from the live
  project.
- **No prompt-engineering tuning beyond the minimal working prompt.** The
  prompt is the deterministic snapshot-plus-contract text above; iteration on
  wording quality is a later, measured slice, not this one.
- **No document upload, retrieval or citations.** Research stages 1 (Q&A with
  citations), 2 (skeleton drafting from documents) and 5 (report drafting)
  are explicitly later; this slice is the *review panel* stage that all of
  them will render through.
- **No streaming and no mid-request cancellation.** The wait is bounded by
  the provider's own 120-second read timeout; a "stop waiting" control is a
  future nicety (Q8).
- **No changes to the nine existing views** beyond the tab strip, View menu
  and shortcut-map entries the new view requires.
- **No settings dialog for the rest of the app.** The configuration form
  covers only the assistant.

---

## Explicit constraints

1. **Keyboard-completable flow.** Every step — configuring the assistant,
   typing a request, submitting it, reading the review, applying or
   discarding — is reachable and operable from the keyboard alone, consistent
   with `ACCESSIBILITY.md` (Tab reaches every control; Space/Enter activates;
   the View menu lists the new view like the other nine). No step requires
   dragging or a pointer-only gesture.
2. **AccessKit labels for every control.** The prompt text field, the
   configure-form fields (including the password-style key field), the change
   cards' headings and buttons, and Apply/Discard all carry non-empty
   accessible names, tied to visible labels with `Response::labelled_by` the
   way the Filter field is. The `a11y_harness` hand-rolled named-input check
   (every visible input widget must carry a non-empty accessible name, both
   themes, two frames) must pass with the new view present.
3. **No secret is ever displayed, logged or stored.** The key field renders
   with password masking; the key never appears in the prompt (the prompt is
   project data only), in a status message, in `Debug` output, in a crash
   report, or in the settings file (RP-2's round-trip test proves the
   settings path drops it). Errors never echo request material — the HTTP
   layer already guarantees that; the status mapping above preserves it.
4. **The review panel shows each change's rationale.** Every
   `ProposedChange` card renders its `rationale` as text; a change is never
   shown without the reason the assistant gave. Before/after values are
   rendered for `SetActivityDuration` and `SetActivityProgress` (old and new
   values read from the live project at render time); created activities and
   relationships are marked as new rows.
5. **Apply goes through the real `CommandStack`.** Apply executes the mapped
   commands exactly as a manual edit does — `stack.execute` per command,
   `recompute()` after — so undo and redo work with no special case, and the
   undo history reads like manual edits. A failing command leaves the
   project unchanged (the command layer validates before mutating) and its
   error becomes a status message.
6. **Off by default, fail closed.** With `AssistantConfig::is_configured() ==
   false` there is no reachable network path: no provider is constructed, no
   thread is spawned, and the request surface does not exist. The unconfigured
   view is the enabling form. Every failure mode (unconfigured, refused,
   network, malformed, invalid proposal) is a plain-language message — never a
   crash, never a partial application (`map_proposal_to_commands` is
   all-or-nothing by construction and its tests pin that).
7. **Colour is never the only signal.** Error, warning and success states in
   the panel carry text (and the status bar carries the same sentence);
   new-row markers carry a text label ("New"), not a colour alone.
8. **The repo's rules hold.** No code in this slice runs `cargo` locally —
   every ticket verifies through `ci.sh` and CI artefacts; no ticket touches
   `.github/` or the shared CI definitions; every change lands as
   `Cadence Dev` via the ordinary branch → CI → PR → merge flow.

---

## Open questions for the owner

Each item below is a decision this spec had to assume to stay concrete. The
recommendation is what the tickets are written against; the owner's answer
before RP-1 starts rewrites the affected sections, not the whole spec.

**Q1 — View placement: tenth centre view tab or right-side panel?**
Recommendation: **tenth centre view tab.** Argued in System architecture: the
right side is owned by per-view inspectors (plus the optional split-Gantt
panel), and the Audit view is the established pattern for a stage-based
centre workflow. A panel would crowd every inspector and cannot show change
cards comfortably.

**Q2 — Keyboard shortcut for the Assistant view.**
`Ctrl+1..9` are taken; every other global chord took letters (`S`, `O`, `N`,
`D`, `W`, `Z`, `Y`). Options:
(a) **No new global chord (recommended for v1):** the view is reached from
the View menu and the tab strip, both fully keyboard-operable (Tab, then
Space/Enter). No key is overloaded; the flow stays keyboard-complete.
(b) `Ctrl+0` — natural next slot, but `Ctrl+0` already resets the Gantt zoom
while the Gantt view is active; the same chord would mean two things
depending on the active view.
(c) Another chord (e.g. `Ctrl+M`) — arbitrary and harder to document.
The spec and tickets are written for (a); adopting (b) or (c) adds one
match-arm and one usage.md row.

**Q3 — Apply granularity: per-change, whole-proposal, or one undo step?**
Research asks for "applies all, some or none" *and* says one undo restores
the previous programme; the landed `command_map` tests pin undo walking back
one change at a time, so those two readings differ. Options:
(a) **Per-change selection with "Apply selected changes" (recommended):**
each change card has a checkbox (ticked by default); Apply maps and executes
the selected subset through the stack. Honours "all, some or none" verbatim;
reuses `map_proposal_to_commands` unchanged; undo removes one change per
press, exactly as the landed tests assert. A subset that references an
activity created by an unselected change fails validation with
`UnknownActivity` — correct, and surfaced as a review error.
(b) Whole-proposal only: simpler UI, but drops "some".
(c) A composite command in `cadence-commands` (apply the batch as one history
entry, one undo restores everything): matches the "one undo" sentence
literally, but adds new command-layer semantics and duplicates the
`DeleteActivities`-style atomic-inverse plumbing for arbitrary command
lists. Larger slice, deferred unless the owner wants single-undo batches.

**Q4 — Settings storage and the API key.**
Options:
(a) **User-level settings file, key session-only (recommended):** the file
(assistant `settings.rs`, platform config directory, no new dependency)
stores enabled/provider/model/base_url; the key lives in memory for the
session and is re-entered after a restart. This honours the codebase's own
documented invariant — `SecretKey` serialises as `"<redacted>"` precisely so
"a settings file, a log line or a crash report cannot carry it" — at the cost
of re-entering the key per session.
(b) OS keystore (keyring crate): comfortable, but adds a platform dependency
and platform-specific failure modes to a slice that is otherwise
dependency-light.
(c) Store the key in the file by bypassing `SecretKey`'s redacting
serialiser: **rejected** — it contradicts the crate's stated design and the
research note's "never in the project file or the binary" spirit.

**Q5 — Where is the assistant configured?**
(a) **Inside the Assistant view's unconfigured state (recommended):** the
enabling form lives where the research criterion 1 says the UI must show how
to enable it; no new global dialog; the feature's settings stay beside the
feature.
(b) A global Settings dialog (new chrome, wider scope, later).

**Q6 — Prompt wording ownership.**
The spec fixes the *structure* (snapshot + request + change vocabulary + JSON
contract) and the tickets implement exactly that; the owner approves the
final wording as part of reviewing this spec (or in RP-1's PR), since the
prompt shapes output quality. No tuning beyond the approved minimal prompt
happens in this slice.

**Q7 — Feature visibility when unconfigured.**
(a) **Assistant tab always present; unconfigured state is the enabling form
(recommended)** — satisfies research criterion 1 ("the UI shows how to enable
it") and costs one always-visible tab.
(b) Tab fully hidden until configured — stricter reading of "off by default",
but then nothing in the UI explains the feature exists, and the a11y
harness's nine-tab fixture stays unchanged at the price of discoverability.

**Q8 — Cancellation while a request is in flight.**
(a) **None in v1 (recommended):** the wait is bounded by the provider's
120-second read timeout; the request button is disabled while waiting.
(b) A "stop waiting" button that drops the receiver (the thread finishes and
its result is discarded) — trivial but adds a state to test; deferred.

---

## Acceptance criteria (slice level, from the research note)

Mapped to this slice — the full research matrix stays in the research note:

1. With nothing configured, no network code path runs and the Assistant view
   shows how to enable the feature. *(criterion 1)*
2. A request produces a proposal rendered for review — prose, per-change
   rationale, before/after for sets — and nothing is written until the user
   applies; every applied change is an ordinary undoable command entry.
   *(criteria 4, 6)*
3. An invalid proposal (unknown activity, progress > 100, self/duplicate
   relationship, blank name, zero duration) is shown as review errors — all
   at once — and nothing is applied. *(criterion 5)*
4. The provider failure modes (not configured, refused, network, malformed)
   surface as plain-language status sentences and never crash, never partially
   apply, never echo request material or key material. *(edge cases)*
5. Every control is labelled for AccessKit; the whole flow completes from the
   keyboard; the a11y harness passes with the new view present.
6. No secret is displayed, logged or persisted; the settings round-trip test
   proves the file cannot carry a key. *(criterion 7)*
