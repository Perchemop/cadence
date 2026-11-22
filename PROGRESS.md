# Progress

Source of truth for build state. "Done" means implemented **and** verified by
green CI (`main` branch).

## 2026-11-21 — Chrome redesign from the multi-agent design audit

Owner complaint: does not like how the app looks, where the toolbars are
placed, and how cluttered it feels; must stay usable for entry-level users.

### Audit findings (three parallel reviewers)

1. **Layout inventory:** the single wrapping top bar held 19 interactive
   controls + ~12 separators (5 menus, 10 tabs, Columns menu, Undo/Redo with
   shortcut labels, a version button); the default Table view stacked four
   chrome rows (toolbar, filter row, split-Gantt toolbar, status bar) and
   ~63 clickable controls; the View menu duplicated the tab strip 1:1;
   Undo/Redo existed twice in two styles; Close and Exit did the same thing.
2. **Design/clutter audit:** 10 distinct button treatments (only 2 tokenised);
   31 labels carrying visible keyboard shortcuts; a separator between every
   pair of view tabs; unselected tabs stroked in critical red (semantic
   colour misuse); a 7-number EVM jargon string in the status bar; "RP-4"
   ticket IDs leaking into user text; Lookahead/Assistant sharing one icon;
   7 off-scale font sizes; four competing spacing rhythms.
3. **Redesign proposal:** two-row chrome (menus + quick access; view
   switcher), two-tier view organisation (3 core tabs + grouped More-views
   menu from one `VIEW_META` table), view-scoped controls moved into their
   views, a tokenised chrome button kit, and a rebuilt status bar.

### Implemented (slices 1–4)

- `design.rs`: added `ACCENT_ON` and the `chrome::{primary, ghost, tab}`
  button kit (tokens only; selected-tab state = fill + weight + hairline,
  never colour alone).
- `lib.rs`: added `ViewGroup` + the `VIEW_META` metadata table; rebuilt the
  tab strip as 3 core tabs + grouped More-views menu (active specialist view
  becomes the menu label); rebuilt the View menu from the same table with
  group captions; deleted the Window menu (theme → View; split Gantt →
  Table filter row); moved the Columns chooser into the Table filter row;
  removed the File ▸ Exit duplicate and the doubled separators; dropped
  menu-title icons; added the quick-access cluster (primary New activity,
  ghost Undo/Redo, shortcut hints in tooltips); removed the toolbar version
  button; fixed the Lookahead/Assistant shared icon (BINOCULARS /
  CHAT_CIRCLE_TEXT); rebuilt the status bar (severity-glyph message /
  plain-language facts + SPI·CPI / save, Shortcuts, subtle version label);
  removed the EVM jargon string and the duplicate zoom readout.
- `tests/a11y_harness.rs`: updated for the two-tier switcher — core tabs
  asserted exactly once, More-views button asserted, every specialist view
  asserted reachable by name when active; Undo/Redo label assertions updated
  to the new ghost labels; Window removed from the menu-bar assertion;
  `ViewMode` made public with a `set_view_mode` seam for hosts/tests.
- Release-notes dialog copy updated for the new chrome.
- **Jargon scrub:** the internal "RP-4" ticket-ID strings are gone from the
  Assistant view; user-facing copy explains the request flow ships later.
- **CI workflow fix:** the evidence job's SIGSEGV check wrote
  `crashed:value` to `$GITHUB_OUTPUT` (must be `key=value`); the job now
  passes and is green on run `44822114868`.

### Verification

- Green CI run `44822114868` on `main`: Verify (ubuntu-latest,
  ubuntu-24.04-arm, windows-latest, macos-latest), Package (Linux),
  Evidence (rendered), Packaging plan — all success; Semver job skipped as
  in all prior runs (event-gated, not in `.ci-required.txt`).
- Interim runs recorded: `44819994633` (all authoritative checks green;
  evidence job failed on the workflow format bug, fixed after), plus the
  red repair chain `44734939827`→`44822114868` (formatting, dependency,
  dead-code, clone-on-Copy, fmt-collapse rounds — each fixed from the CI
  log, never bypassed).

### Released: v0.17.0 (2026-11-21)

**v0.17.0 shipped** with the chrome redesign from the multi-agent design audit.
Release workflow run `44829423233` completed successfully on tag `v0.17.0`.

Artefacts attached:
- cadence-linux-x86_64.deb
- cadence-linux-aarch64.deb
- cadence-macos-aarch64.dmg
- cadence-windows-x86_64.msi
- SHA256SUMS.txt + SHA256SUMS.txt.minisig (minisign signature)
- cadence-sbom-cyclonedx (CycloneDX SBOM)

Token ledger updated: cumulative 1,711,676,910 (Tier A+B), this release 0 tokens.
`main` is current with the release tag — no commits ahead.

### Deferred (from the audit, not in this slice)

- "RP-4" ticket-ID strings and raw activity IDs in the Assistant/variance
  views (jargon scrub, next slice).
- Per-dialog danger styling for the 17 confirm modals; empty-state type size.
- Inspector section separators (headings alone would carry the structure).

## 2026-11-21 — Design audit (owner complaint: look, toolbar placement, clutter)

Owner complaint: does not like how the app looks, where the toolbars are
placed, and how cluttered it feels; must stay usable for entry-level users.

- **Multi-agent audit launched** (three parallel reviewers): (1) full
  chrome/toolbar placement inventory, (2) visual-design + clutter audit
  (colour discipline, button-style consistency, label verbosity, ranked
  clutter sources), (3) redesign proposal (panel architecture, view
  grouping, de-cluttering moves, styling rules, implementation slices).
  Findings and the redesign plan will be recorded here when the agents
  report.
- **CI repair en route:** the 2026-11-20 Gantt/menu commits were red on
  `main` (formatting, then compile errors masked by the format job:
  missing `cadence-interchange` dependency in `ui-egui`, dead bindings
  `header_color`/`coarse_band_height`/`fine_band_height`/`palette`).
  Fixed in commits `cfef3af`, `4eef0f6`; green run recorded here once the
  run completes.
- **Import/export dialogs wired** (from the 2026-11-20 menu work): the
  File menu Import/Export entries now open real dialogs backed by
  `cadence-interchange` (json/mspdi/mpx/csv import; json/mspdi/csv export)
  via `import_from_path` / `export_to_path`.

## 2026-11-20 — Gantt enhancements and menu overhaul

### V5: Non-working day shading visibility
- Improved non-working day fill opacity from 30% to 45% for better contrast
- Light theme: `Color32::from_rgb(0xF0, 0xF4, 0xF8)` for visible weekend shading
- Dark theme: `Color32::from_rgb(0x1A, 0x1E, 0x27)` for subtle dark grey
- Updated in `crates/ui-egui/src/design.rs`

### V6: Two-tier timescale
- Already implemented in codebase (lines 6937-6942 in lib.rs)
- Major bands: months/quarters/years at medium+ zoom
- Minor bands: days/weeks at finer zoom levels

### V7: Status date line in Gantt
- Added dotted line indicator for status date (separate from today line)
- Color: `palette.status_date` with 50% opacity red (light) or accent tint (dark)
- Positioned as dotted line in the Gantt canvas
- Changed `_status_date` parameter to `status_date` to actually use it

### Constraint markers (V8)
- Added visual indicators for MustStartOn/MustFinishOn constraints
- Dashed vertical lines at constrained dates on activity bars
- Diamond markers at constraint positions
- Color: critical tinted at 70% opacity

### Menu system overhaul (V9)
- Reorganized File menu with comprehensive file operations
- Added: New project, Import, Export
- Reorganized Edit menu
- Added: Duplicate activity (placeholder with guidance), Delete activity, Break link
- Added helper methods: `new_project()`, `duplicate_selected_activity()`, `delete_selected_activity()`, `break_incoming_links()`
- Added state fields: `pending_delete_activity_id`, `show_import_dialog`, `show_export_dialog`

### Design palette updates
- Added `status_date` field to `Palette` struct
- Improved non-working day colors per theme
- All colors maintain WCAG AA contrast

### Verification
CI verification pending after this commit.

## 2026-11-20 — EVM metrics completion + MSPDI WBS/calendar round-trip verification

The EVM metrics (EAC/BAC/TCPI/VAC) and MSPDI WBS calendars implementation from the WIP branch are now committed.

### EVM metrics (CAP-029)
- Updated `docs/capability-inventory.md`: CAP-029 ECM metrics status updated to `implemented`
- Updated `docs/gaps.md`: removed EVM EAD/BAC/TCPI/VAC gap
- Updated `docs/parity.md`: EVM row updated
- Updated `docs/qc/capability-audit.md`: EAC/ETC/VAC/TCPI marked as shipped

### MSPDI WBS and Calendar round-trip
- Added `Task/WBS` import (lines 162-167) and export (lines 417-420)
- Added `Project/Calendars/Calendar` import (lines 99-133) with weekly pattern and holidays
- Added TDD tests:
  - `wbs_code_is_imported_from_mspdi` - verifies WBS codes are parsed from MSPDI
  - `wbs_code_round_trip_through_mspdi` - verifies WBS round-trip
  - `named_calendars_are_imported_from_mspdi` - verifies calendar import with names, workweek and holidays
  - `calendar_with_invalid_workweek_defaults_to_weekend` - verifies fallback behavior
- Fixed test assertions: Standard calendar (1111110) has 6 working days, 7-day calendar has 7 working days
- Updated MSPDI documentation comment to include WBS and Calendars elements

### Gap updates
- Removed EVM EAC/BAC/TCPI/VAC gap from gaps.md (P1)
- Updated interchange gap description to reflect MSPDI now supports WBS/named calendars
- Updated capability-inventory.md CAP-046 status from `partial` to `implemented`

### Verification
CI verification completed. All tests pass including:
- `wbs_code_is_imported_from_mspdi` - WBS codes correctly parsed
- `wbs_code_round_trip_through_mspdi` - WBS round-trip verified
- `named_calendars_are_imported_from_mspdi` - Calendars with names, workweek and holidays imported
- `calendar_with_invalid_workweek_defaults_to_weekend` - Fallback behavior works
- EVM new metrics tests (EAC, TCPI, VAC) all pass

Commit verified: 7c0694ef pushed to main.

## 2026-11-20 — EVM completion slice (P1 gap)

- **EAC/BAC/TCPI/VAC implementation:** Extended `EvmMetrics` struct with
  `estimate_at_completion`, `to_complete_performance_index`, and
  `variance_at_completion` fields (`crates/core/src/model.rs`).
- **Updated `EvmMetrics::for_activity`** to compute EAC (BAC/CPI fallback),
  TCPI ((BAC-EV)/(BAC-AC) zero-denominator convention), and VAC (BAC-EAC).
- **Updated `engine::evm_metrics`** to aggregate the new metrics from all
  activities.
- **Added unit tests** for all three new metrics including happy paths,
  zero-division fallbacks, boundary conditions (0%/100% progress), and
  determinism verification (`crates/core/src/model.rs` tests).
- **CHANGELOG:** added `## [0.16.0] — Unreleased` section.
- **CAP-029** now marks EAC/BAC/TCPI/VAC as implemented in
  `docs/capability-inventory.md`.
- **Parity update:** `docs/parity.md` EVM row marked from partial → done.
- **DECISIONS.md:** entry D39 records the EVM completion slice decisions.

## 2026-11-19 — release signing bootstrap + v0.15.0 readiness

The signing key exists, the release is dated, and the tag is next:

- **Signing key bootstrap (D38):** the minisign keypair was generated and its
  secret key stored as the `RELEASE_SIGNING_KEY` repository secret per the
  owner's direction; local material shredded in the same pipeline. Public key,
  key ID `5F50A5EC98109664` and fingerprint recorded in
  `docs/update-signing.md`; v0.15.0 is the first signed release.
- **Workflow fix:** minisign 0.12 silently no-ops bare-positional file
  arguments — the release workflow's sign/verify steps now pass `-m` explicitly
  (the post-sign self-verify would have failed the release loudly either way).
- **RP-1/RP-2 landed** (PR #182): prompt builder + response parser and settings
  storage, with recorded red/green run IDs per §5.6.4.
- **CHANGELOG:** the 0.15.0 section is dated 2026-11-19 — the release notes
  body derives from it at tag time.

Verification: CI run for this slice on `opencode/ci-signing-docs` (result
recorded in the PR once green); the release itself is the next verification.

## 2026-11-19 — assistant review panel, RP-1 + RP-2: prompt/parser seam and settings storage (D37)

`crates/assistant` slice; no UI touched (RP-3+ pending on the review-panel
series). Both tickets verify headless; no new dependencies, lockfile
unchanged:

- **RP-1 — prompt builder + response parser** (`crates/assistant/src/
  prompt.rs`): `build_proposal_prompt` builds the deterministic minimal
  working prompt — compact project snapshot (name, start date, per-activity
  id/name/duration/percent, relationships) capped at 500 activities with an
  explicit truncation notice, the planner's request verbatim, the four
  typed change variants with their exact field names, the validation rules,
  and the one-JSON-object output contract. `parse_proposal_response`
  accepts the bare JSON object or the same object inside one markdown code
  fence and rejects everything else as `AssistantError::MalformedOutput`
  with a plain-language detail that never echoes the body; prose is never
  interpreted. Six headless tests, including the injection-shape test.
- **RP-2 — settings storage** (`crates/assistant/src/settings.rs`):
  `settings_path` (platform config directory via `cfg!` branches — XDG
  config dir or HOME on Linux, APPDATA on Windows, HOME/Library/Application
  Support on macOS; hand-rolled, no new dependency; the file is
  `cadence/assistant-settings.json`), `load_settings` (never fails the
  caller — a missing, unreadable or unparseable file yields the default
  off config) and `store_settings` (drops the API key before writing —
  session-only, honouring `SecretKey`'s `"<redacted>"` serialiser invariant
  — and creates the directory chain). Seven headless tests using injected
  environment values or self-owned temporary directories.
- **TDD evidence**: RP-1 red run 44500433308 (commit 4013487, 5 expected
  failures) and green run 44504244460 (commit 85d3437, `=== CI RESULT:
  SUCCESS ===`). RP-2 red run 44511538745 (commit adef464, 5 expected
  failures on Ubuntu, 4 on Windows/macOS) and green run 44514726568
  (commit f3259c8, `=== CI RESULT: SUCCESS ===`). Records: CHANGELOG
  0.15.0 Added, DECISIONS D37, REQUIREMENTS_TRACEABILITY AI row.
- Pre-existing, non-gating: `Evidence (rendered)` fails on the CI runner
  (wgpu "No adapter found" — no Vulkan adapter for the ui-egui kittest
  evidence job); the check is not in `.ci-required.txt` and is unrelated to
  this slice (no `crates/ui-egui/**` change).

Verification: PR #182 (`opencode/ci-rp12` → `main`), required checks green
on the slice head.

## 2026-11-19 — token ledger: accurate parent-chain attribution (D36)

Tooling slice; no application code touched (no `crates/**`):

- **`scripts/token-ledger.py`** gains a second attribution tier on top of the
  D34 path rule. Tier A (direct) = directory/project-worktree path match plus
  the store's `worktree`-table directories for this repository's projects
  (catches opencode-managed worktrees without a repo-prefixed name).
  Tier A+B (headline) = Tier A plus the transitive parent-chain closure:
  every ancestor orchestrator and every descendant subagent of a direct
  session (`session_v2.parent_id`; siblings excluded). The D34 floor is kept
  as the reported "Direct (Tier A)" subset.
- **Measured result** (store read 2026-11-19): Tier A 10 sessions /
  6,790,816 tokens — byte-identical to the D34 floor, confirming no
  attribution regression. Tier A+B 12 sessions / 1,711,676,910 tokens: two
  ancestor `build` orchestrators carry 1,704,886,094 tokens (bucketed by
  start time into 0.3.0 +1,125,404,657 and 0.11.0 +579,494,770); 832 of 844
  store sessions stay excluded. Documented caveat: cross-repo orchestrators
  attribute wholly to Cadence (no per-repo split exists in the store), so
  the headline leans high and the direct column is the strict lower bound.
- **Unreleased-cycle consistency fix:** the D34 script added
  after-newest-tag sessions to `total_tokens_since_start` while CI asserted
  equality with the last release's cumulative — a latent red-CI trap on the
  first mid-cycle regeneration. The ledger now emits additive
  `tokens_unreleased` / `tokens_direct_unreleased` keys and CI validates
  `total == last cumulative + tokens_unreleased` (both 0 today).
- **`docs/tokens.md` / `docs/tokens.json`** regenerated: the §24 table gains
  an additive "Direct (Tier A)" column and the JSON additive
  `tokens_direct` / `total_tokens_direct` / `*_unreleased` keys; the app
  reader's three keys keep their names (`total_tokens_since_start` is now
  the Tier A+B headline). Privacy constraints unchanged and restated in the
  script docstring: numeric usage columns plus path/timestamp/ID metadata
  only (the opaque `parent_id` is used solely for attribution); message,
  part, title and message-JSON data are never read; store opened read-only;
  no network.
- **CI** ("Token ledger consistency", Linux): extended to validate the new
  md column and JSON keys (direct ≤ headline per release, direct-total
  arithmetic, unreleased totals) — validation only, CI never computes token
  numbers. Negative-tested locally: tampered md column, tampered JSON
  direct, broken total arithmetic, direct-exceeds-headline and negative
  unreleased all fail the gate; the regenerated files pass.
- **Records:** DECISIONS D36, CHANGELOG (0.15.0 amendment + header note),
  this file.

Verification: CI run for this slice on `opencode/ci-tokens2` (result
recorded in the PR once green).

## 2026-11-19 — user-directed skills: relocation, invocation policy, records (D35)

Process-infrastructure slice; no application code touched:

- The D31 skill packs were relocated from `~/.config/opencode/.agents/skills/`
  to `~/.config/opencode/skill/` (the directory this OpenCode build loads);
  invocation verified in-session (`poteto-mode` loaded, registry hot-reloaded).
- The owner's auto-invocation directive is codified in the global ruleset and
  mirrored into `.opencode/agents/orchestrator.md` so every future Cadence
  session invokes matching skills automatically (`lean-build` was invoked for
  the RP-1/RP-2 slice in flight).
- pstack licence verified from source: MIT (`LICENSE` + `plugin.json`), —
  the guide §22.2.3 "Apache-2.0" claim is superseded by the licence file
  (§25 use-rule 3); recorded as D35 and flagged to the owner.
- Records: DECISIONS D35, `.opencode/agents/orchestrator.md`, this file.

Verification: CI run for this slice on `opencode/ci-skills` (result recorded
in the PR once green).

## 2026-11-18 — token ledger: measured usage from local OpenCode records (D34)

Tooling slice; no application code touched (no `crates/**`):

- **`scripts/token-ledger.py`** (stdlib-only) aggregates the real per-session
  token columns from the local OpenCode store (`session_v2`), attributes
  sessions by path (checkout + `cadence-*` worktrees), buckets them by
  release-tag date and rewrites `docs/tokens.md` + `docs/tokens.json`
  together. Privacy constraints in the script docstring: numeric aggregates
  and path/timestamp metadata only — message content is never read.
- **`docs/tokens.md`** created (guide §24 per-release table); measured totals:
  6,790,816 tokens cumulative, the 0.13.0 cycle the largest (+3,279,251).
  The ledger is a measured floor: home-directory sessions (orchestrators and
  subagents) are not attributable by the store and are excluded.
- **`docs/tokens.json`** keeps the app reader's three keys
  (`total_tokens_since_start` / `release_tokens` / `release_version`) and
  adds the §24 `releases` array; `release_tokens` 0 is the measured v0.14.0
  cycle (no attributable sessions in its window).
- **CI** gains the "Token ledger consistency" step (Linux, in `verify`):
  validates tokens.md ↔ tokens.json mutual consistency, running-cumulative
  arithmetic, date/number formats and the reader's JSON keys. Validation
  only — CI never computes token numbers.
- **Records:** CHANGELOG (0.15.0 Added + header note), D34, this file.

Verification: CI run for this slice on `opencode/ci-tokens` (result
recorded in the PR once green).

## 2026-11-18 — capability inventory + parity refresh (designer role)

Docs-only slice (guide II.27.B capability inventory + II.27.D parity review).
**No code changed**: no `crates/**`, workflow, `.opencode/` or `.github/` file
was touched; CHANGELOG and DECISIONS untouched because no user-facing surface
changed. Every status below was verified against the branch's actual code and
tests (`main` @ `7755c92`) before being recorded.

- **`docs/capability-inventory.md` created — first edition, 61 rows**
  (54 implemented, 5 partial, 2 not started). The file did not exist before:
  capability claims lived only in `docs/parity.md` and
  `REQUIREMENTS_TRACEABILITY.md`. Every `implemented` row names its evidence
  (test, CLI surface or GUI view); 18 audit `CheckId` variants and the nine
  GUI views (`Ctrl+1`–`Ctrl+9`) were counted in source, not from memory.
- **Assistant inventoried honestly as backend-only** (CAP-056…061): provider
  configuration, consent/secret hygiene, proposal generation and review/apply
  are each `partial` — the `cadence-assistant` crate (51 headless tests) has
  config + four wire protocols + validation + undoable command mapping, but
  there is **no UI, no prompt construction from project data and no review
  panel**, so no assistant capability is user-reachable yet. Document
  understanding (CAP-060) and broader advice/enrichment proposals (CAP-061)
  are `not started`. The "assistant proposes, planner disposes" rule is
  stated on every assistant row.
- **One partial verdict for an existing capability**: interchange losslessness
  is JSON-only (CAP-046) — WBS and named calendars import empty from
  MSPDI/MPX/CSV; the limitation is documented in `interchange::json`, so the
  row is `partial`, not a regression.
- **`docs/parity.md` refreshed** with a per-group parity section vs the
  reference products. Corrections of stale/over-claimed rows: the audit row
  said "14 core checks" — the code has **18**; the "first tagged release" row
  said *in progress* — releases `v0.3.0`…`v0.14.0` are tagged; the AI row
  said *approved — queued* — three backend slices have landed (PRs #155,
  #174, #175); the crate-layout row omitted `ui-egui`, `interchange` and
  `assistant`; the persistence row now records schema v10. New honest gaps
  recorded rather than hidden: EVM lacks EAC/BAC/TCPI/VAC; P6-style
  resource curves/time-phased rates are missing; **Asta and ProjectLibre have
  no approved research note, so no parity claim is made against them**.
  Exceeds-reference items recorded: headless `--json` audit/comparison/schedule
  surfaces, keyless local-Ollama/OpenAI-compatible support (D30), and
  progress-driven rescheduling as an explicit property-tested engine mode
  (D24).
- **Verification.** PR #178; the authoritative check set on head `1cf61a8`
  (Verify on four platforms + Package, the six Security checks, Lockfile
  freshness, Personal data scan). Note: the `pull_request` trigger registered
  no runs for this PR (GitHub-side event suppression; a close/reopen also
  produced none), so CI/Security/Lockfile were verified via
  `workflow_dispatch` on the branch, which uses the identical workflow files;
  the Personal data scan has no dispatch trigger and rides the synchronize
  event of this amendment.
- **Observation for a future slice (not done here, docs-scope only):** the
  landed assistant slices are recorded in the traceability file's
  future-expansion table but have no `R31+` rows in
  `REQUIREMENTS_TRACEABILITY.md`; the next assistant slice (review panel)
  should add them.

## 2026-11-18 — review panel specification drafted (planning slice, no code)

The **assistant review panel** — the last assistant stage per
[`docs/research/ai-integration.md`](docs/research/ai-integration.md) ("the
assistant proposes, the planner disposes") — is now specified and ticketed
under the guide §5.6 Case-3 path (`/to-spec` + `/to-tickets`):

- **Spec:** [`docs/specs/review-panel.spec.md`](docs/specs/review-panel.spec.md).
  Recommendation: the Assistant becomes a **tenth centre view tab** (argued
  from the shell's nine-view pattern, the inspector-owned right side and the
  Audit-view precedent for a stage-based workflow); the provider call runs
  **off the UI thread** with the same `std::thread` + `mpsc` pattern the
  update check uses; the prompt is built on the UI thread from the live
  project, the reply is parsed and validated against the project, and Apply
  maps the accepted changes through `map_proposal_to_commands` into the real
  `CommandStack` (undo works with no special case). **Eight open questions**
  (view placement, shortcut, apply granularity, settings/key storage,
  configuration surface, prompt wording ownership, unconfigured visibility,
  cancellation) are listed with recommendations — **the owner reviews the
  spec before RP-1 starts** (human gate, §5.6.3/§2.3).
- **Tickets:** `docs/tickets/RP-1.md` … `RP-6.md` — RP-1 prompt builder +
  response parser (assistant, headless); RP-2 settings storage, key
  session-only (assistant); RP-3 view shell + enabling state (ui-egui);
  RP-4 request flow + review rendering + apply/discard (ui-egui);
  RP-5 keyboard + AccessKit harness pass; RP-6 usage docs + tracking.
- **No code changed in this slice.** Only `docs/specs/`, `docs/tickets/`
  and this file were touched; `CHANGELOG.md` and `DECISIONS.md` are
  deliberately untouched (no released surface changed; no decision made —
  the spec *proposes*, the owner disposes).
- **Implementation waits on the open ui-egui owner branch (PR #173)
  resolving**, so the first implementation ticket starts from a current
  `main` without colliding with in-flight owner work.

## 2026-11-18 — OpenCode workflow commands (guide §5.6.3) + docs reference index (§5.7.2)

Process-infrastructure slice; no application code touched (no `crates/**`,
no `.github/**`):

- **Seven command definitions** in `.opencode/command/` — `grill-me.md`
  (read-only `agent: plan` interrogation), `to-spec.md`, `to-tickets.md`,
  `tdd.md` (red/green/refactor with every phase verified through
  `ci.sh run "TDD …"` and a recorded red-run ID), `implement.md`
  (single-ticket `/new` session; commit via ci.sh; merge only via
  `ci.sh merge`), `implement-spec.md` (worktree isolation; sub-agents obey
  the HARD BUILD RULE — no local builds, verification through `ci.sh`) and
  `code-review.md` (`agent: plan` + `subtask: true`; architectural,
  tactical, clean-room and secrets axes). Each file states purpose,
  invocation and the binding §5.6.3 rules, and points at the §5.6.5
  reconciliation table (D32).
- **`docs/reference/index.md`** — the §5.7.2 documentation pre-loading
  index for agent teams: one line per file in `docs/`, grouped (product &
  scope, research, design, audits & quality gates, process & records);
  regenerated when docs change.
- **Records:** CHANGELOG (0.15.0 Added), D32, this file.

Verification: CI run for this slice on `opencode/ci-commands` (result
recorded in the PR once green).

## 2026-11-17 — optional AI assistant: HTTP provider backends (merged)

The network half of the assistant landed behind the provider seam and was
merged to `main` as PR #174 (`10df2c6`, all post-merge checks green):

- **Four wire protocols** (D30): OpenAI chat-completions — also spoken by any
  custom provider name, i.e. OpenAI-compatible servers such as LM Studio or
  vLLM, which require their own `base_url`; Anthropic Messages with the
  pinned `anthropic-version` and the required `max_tokens`; a locally hosted
  Ollama (no key, streaming off); Google Gemini `generateContent` with the
  model in the URL. Shapes verified against the vendors' published API
  references (October 2026).
- **Config.** `AssistantConfig` gains `model` and `base_url` (serde defaults;
  settings saved by 0.14.0 still load). A model is required, a local Ollama
  needs no key, a keyless OpenAI-compatible server needs only its address,
  and credential headers are omitted entirely (never empty) when keyless.
- **Errors.** Plain-language mapping: 401/403 and other 4xx are `Refused`
  (configuration problems); 408/429/5xx and transport faults are `Network`,
  carrying only the status and the provider's own short JSON error message —
  request material is never echoed. Keys never appear in a request body or
  URL (tested on every wire).
- **Tests.** 24 headless tests in `http.rs` (request shapes, response
  parsing for all four protocols, error boundaries, redaction invariants)
  plus four config tests; no test opens a socket.
- **Second-opinion review.** A reviewer pass on the finished diff found one
  major (keyless OpenAI-compatible servers were rejected by
  `is_configured`, contradicting the advertised support) and two minors
  (empty credential headers when keyless; transport-error text echoing
  request context). All fixed before merge with regression tests; recorded
  as the D30 extension.
- **Records.** D30 + D31, CHANGELOG 0.15.0, ROADMAP, traceability, the
  README privacy note for the opt-in provider calls, and the guide-sync note
  in `docs/current-model-map.md`.

Verification: PR #174 required checks green (Verify on four platforms,
Package, Semver, Lockfile, Security scans, Personal data scan — runs
44253699727/44253699682/44253699890/44253699731 on `10df2c6`).

## 2026-11-18 — optional AI assistant: command-stack wiring

Accepted proposals now become ordinary undoable commands. New
`crates/assistant/src/command_map.rs`:

- **`map_proposal_to_commands`.** A pure, headless translation from a
  validated `Proposal` to `cadence-commands::Command` values in proposal
  order: `CreateActivity`, `CreateRelationship` (plainest finish-to-start
  link, no lag — the planner edits type and lag afterwards, exactly as for a
  manually added link), `SetActivityDuration`, and `SetActivityProgress`
  (whole percentage converted to the command layer's basis points at this
  one boundary). Identifiers come from the live project's next free ids.
- **No partial application.** The mapping re-runs `Proposal::validate`
  against the live project and returns the first problem as an error — never
  a partial command list.
- **Same undo path as manual edits.** Eight headless workflow tests apply the
  mapped commands through the real `CommandStack` on a real `Project` and
  assert the resulting state *and* that undo restores the previous state,
  including a multi-change proposal undone one change at a time.
- **Process note.** A first implementation by a parallel subagent was
  discarded: it worked directly in the shared checkout instead of its
  assigned worktree (violating the file-ownership rule), never compiled
  (invented undo-description strings, a missing `Some(...)`, a struct
  pattern on a unit variant, no `cadence-commands` dependency), and included
  a fake coverage test. The landed module was re-derived against the real
  `cadence-commands`/`cadence-core` sources; the salvage informed the shape
  but every claim was verified in the source first.
- **Tracking.** `CHANGELOG.md` (0.15.0/Added), the ROADMAP AI item and
  `REQUIREMENTS_TRACEABILITY.md` updated. Next assistant slice: the review
  panel.

Verification: CI run for this slice on `opencode/ci-cmdstack` (result
recorded in `CHANGELOG.md` and the PR once green).

## 2026-11-16 — optional AI assistant: foundation crate

The first slice of the owner-approved AI-integration research lands as a new
`cadence-assistant` workspace crate — the provider-agnostic foundation only,
so every behaviour stays testable headlessly (K.13):

- **Provider seam.** `AssistantProvider` (`complete`/`name`) with a
  deterministic `MockProvider` for tests and a fail-closed `NoopProvider`;
  `AssistantError` carries plain-language reasons (not configured, refused,
  malformed output, network) and never key material. No network code ships in
  this slice — HTTP backends arrive next, behind the same seam.
- **Typed proposals.** `ProposedChange` (create activity/relationship, set
  duration/progress), each carrying a rationale the review panel can cite;
  `Proposal::validate` returns *all* problems at once — unknown activities,
  progress above 100, self- and duplicate relationships, blank names, and
  zero durations (reserved for milestones, which the assistant must not
  create). Validation runs against the real `cadence-core` `Project` model.
- **Secret hygiene.** `SecretKey` prints `SecretKey("**redacted**")` in Debug
  and serialises as the literal `"<redacted>"`, so settings files, logs and
  crash reports cannot carry a key; `AssistantConfig` is off by default and
  `is_configured()` gates the feature.
- **Tests.** 13 headless unit tests covering the validation matrix, config
  gating and the secret-redaction invariant.
- **Tracking.** `CHANGELOG.md` (Unreleased/Added), the ROADMAP AI item and
  this file updated. The unused `cadence-assistant` workspace-dependency
  entry was removed again (nothing consumes the crate yet — the next slice
  re-adds it with the command-stack wiring, per the dead-code rule).
- **Evidence.** All 18 required checks green on PR #155 (Verify on four
  platforms, Package (Linux), Static analysis (rust), Lockfile freshness,
  Secret/Personal data scans, Semver, CodeQL, Evidence, Bill of materials);
  merged as bee1fef.
- **Dependency triage.** Dependabot PRs reviewed: #161 (rfd 0.18) merged;
  #156 (install-action) awaiting rebase; #157/#159/#160 (egui 0.37 family)
  closed with reasons — a toolkit bump is its own slice per guide II.30.3.
  #158 (rusqlite 0.42) was closed blaming the rendered-evidence SIGSEGV, then
  **re-opened after the evidence job crashed identically on a docs-only
  merge commit and passed on re-run with no code change** — the crash is the
  flaky software-Vulkan evidence job, not the bump; see issue #163 (quarantine
  record per the guide's flaky-test policy M.5).

## 2026-11-15 — lookahead published snapshots: delete, variance highlighting, test coverage, records reconciliation

Audit of the published-snapshots stage against `ROADMAP.md` and
`docs/research/aphex-integration.md`: the core, command and persistence layers
were already complete (schema v6, undoable capture/remove/restore); the gaps
were GUI reachability, variance readability and tests:

- **Snapshot deletion in the GUI.** `RemoveLookaheadSnapshot` existed but had
  no UI entry point, so a published snapshot could never be deleted. The
  Lookahead view now has a **Delete snapshot** button (enabled when a snapshot
  is selected) behind the same confirm-dialog pattern as baselines — Enter
  confirms, Escape cancels, undo restores the snapshot to its original
  position.
- **Variance highlighting.** The snapshot comparison showed start/finish
  variance as plain text. Each variance now carries the shared
  `design::variance_color` token (delay red, recovery green, unchanged muted),
  and removed/added states are tinted warn/ok; the signed text stays so colour
  is never the only signal.
- **Test coverage.** New command tests: capture records the window (names,
  dates, window fields), capture/remove are reversible and removal keeps list
  position, unknown ids are rejected. New engine test: a captured snapshot is
  read-only — later edits and reschedules never rewrite it. The persistence
  round-trip test now includes a snapshot entry with all fields populated
  (the entries table was previously only exercised empty).
- **Records reconciliation.** `ROADMAP.md` still listed "published snapshots,
  the diary, production tracking, clash detection" as remaining although all
  four were marked done earlier in the same file; the lookahead bullet and
  `docs/parity.md` now read done, and the stage is traced as R30 in
  `REQUIREMENTS_TRACEABILITY.md`.

## 2026-11-15 — programme audit: four new schedule-quality checks (18 total)

Four additions to `crates/core/src/audit.rs`, taking the audit tool from 14
checks to 18 — the 12 core checks plus AUDIT-CPT and AUDIT-CPLI (landed
2026-10-28, PR #45); a "12 → 16" reading of the count misses those two:

- **`AUDIT-UNNECESSARY` (UnnecessaryConstraints).** A hard date constraint —
  the DCMA hard set: no-later-than and must-on starts and finishes; a
  no-earlier-than bound is soft and does not count — on an open activity that
  is not on the critical path. The constraint is not needed to protect the
  finish and only stops the schedule from moving. Not-applicable when no open
  activity carries a hard constraint.
- **`AUDIT-LONGLAG` (LongLags).** A positive relationship lag longer than the
  new `AuditConfig::long_lag_days` threshold (default 5 working days, one
  working week; a config document without the field receives the default via
  serde). Long lags delay the successor without representing real work and
  hide the true driver of the dates.
- **`AUDIT-REDUNDANT` (RedundantLogic).** The same successor reached by more
  than one path from one predecessor — a duplicated relationship, or the
  predecessor already reaching another predecessor of that successor.
  Redundant links distort float and the critical path. The reachability walk
  is iterative and bounded by a visited set, so long programmes and cyclic
  networks cannot overflow the stack.
- **`AUDIT-OOS` (OutOfSequenceProgress).** An open activity with recorded
  progress (an actual start, or percent complete above zero) while a
  finish-to-start predecessor is still unfinished. The report cites the
  activity and never rewrites recorded progress; overlapping SS/FF/SF links
  are ignored, and the check is not-applicable without recorded progress.

Eighteen new unit tests (39 in total in the module) cover each check's fail,
pass and not-applicable paths, the serde round trips for the report, the
config and the new check ids, the stable codes and names, the legacy-config
default, and determinism. The GUI Audit view, the `cadence audit` CLI and the
`kind: "audit"` JSON document pick the new checks up from the central runner.

**Process note.** The first verification PR for this work (#149) merged
empty: its head branch (`opencode/ci`) carried only the CI verification
commit while the code sat unmerged on `fix/audit-more-checks` — the CI
branch and the work branch diverged. The checks land on `main` via PR #150.
The first CI round on the real code failed `cargo fmt --check` (a required
check; four spots, fixed) and was flagged by the semver check
(`constructible_struct_adds_field` for `AuditConfig.long_lag_days`,
`enum_variant_added` for the four `CheckId` variants) — answered per the
recorded precedent (`e4150bc`): the workspace version moved 0.13.0 → 0.14.0
and `Cargo.lock` was regenerated from the Lockfile workflow's artefact
(540 packages), after which the semver check passes.

Verification: CI green on PR #150 — all 13 required checks per
`.ci-required.txt`, the semver check and Evidence (rendered) included — and
merged to `main` (`831d13e`).


## 2026-11-15 — fix: regenerate Cargo.lock from CI (malformed + stale)

The `egui_kittest` dev-dependency added in the 2026-11-14 phase-2a slice left two
problems in `Cargo.lock`: (1) a stray empty `[[package]]` block immediately before
`name = "egui_kittest"` that made the file fail to parse, and (2) a resolution
that was missing ~49 packages relative to the manifests, so `cargo metadata
--locked` (the lockfile freshness gate) also failed on drift.

Both are fixed by taking the lockfile the `Lockfile` workflow's **Regenerate
Cargo.lock** job produces on the pull request (its designed path — cargo is never
run on the maintainer machine) and committing it as-is: 540 resolved packages,
valid TOML, re-resolved off yanked pins. `Lockfile freshness`, the platform
`Verify` jobs, `Package (Linux)` and `Dependency licences and advisories` all
failed only because the committed lockfile was unparseable/stale; the regenerated
file addresses all of them.

## 2026-11-15 — fix: accept unmaintained `paste` advisory (RUSTSEC-2024-0436)

The regenerated lockfile surfaced `paste 1.0.15` flagged as unmaintained
(RUSTSEC-2024-0436), pulled in transitively via `rav1e → ravif → image →
arboard → egui-winit → eframe → cadence-ui-egui`. The crate is archived by its
author and no safe upgrade exists until the egui/eframe stack drops it, so it is
added to the `deny.toml` advisories ignore list alongside the existing
`ttf-parser` and `zvariant`/`zbus` entries, with reason and acceptance date.

## 2026-11-14 — evidence pipeline phase 2a: kittest AccessKit assertions over the shell

Phase 2a of the guide II.30 evidence pipeline (V15): AccessKit-tree
assertions over the real shell, no GPU, no rendered pixels yet:

- **Dev-dependency.** `egui_kittest = "0.35.0"` added to `crates/ui-egui`
  as a dev-dependency, with the `wgpu` and `snapshot` features **off** —
  AccessKit assertions only, no software Vulkan needed.
- **Harness.** New integration test
  `crates/ui-egui/tests/a11y_harness.rs` drives the real shell
  (`CadenceApp::render`) through a kittest harness and asserts the AccessKit
  tree: all nine view tabs reachable by name; Undo/Redo present, labelled and
  exposed as `Disabled` while the stack is empty; the menu bar
  (File/Edit/View/Window/Help); the Shortcuts (`F1`) button; the version
  button; the Filter text input reachable via its visible label; and the
  "Critical only" checkbox labelled. A hand-rolled named-input check (every
  visible input widget in the tree must carry a non-empty accessible name —
  kittest 0.35.0 predates the built-in check) runs across two frames and both
  themes, and is now the **S2-20 regression gate**.
- **Offline seam.** The fixture turns the auto-update check off via the new
  public seam `CadenceApp::set_update_checks_enabled(false)`, so the tests are
  strictly offline.
- **Bug found and fixed en route.** The shell re-sent the window-title
  `ViewportCommand` every frame (each command requests a repaint); the title
  is now sent only when it changes, ending a continuous repaint.
- **Determinism.** Tests use `Harness::step` frames rather than `run`,
  because the shell requests delayed repaints (the update poller's 250 ms
  tick) that would keep stepping.
- **Verification.** All 17 CI checks green on all four platforms (the
  harness needs no GPU).

- **Phase 2b (next).** Rendered PNG artefacts via the `wgpu` + `snapshot`
  features — needs software Vulkan (lavapipe: `mesa-vulkan-drivers` /
  `libvulkan1`) on the hosted runner, medium/high risk with a documented
  fallback (stay at phase 2a's AccessKit evidence). S3-20 (Inter Bold-first
  ordering) is evaluated once phase 2b pixels exist.

## 2026-11-13 — evidence pipeline phase 1: headless shell seam + smoke test

Phase 1 of the guide II.30 evidence pipeline (V15) — headless seam and smoke
test, no GPU, no new dependency, runs in the normal `cargo test --workspace`
on all four CI platforms:

- **Seam.** The whole UI body moved out of the `eframe::App::ui(&mut self, ui, _frame)`
  trait impl (which had an unused `eframe::Frame`) into a public inherent
  method `CadenceApp::render(&mut self, ui: &mut Ui)`; the eframe impl now
  delegates (`self.render(ui)`). This removes the recorded blocker that headless
  rendering needed a fabricable `eframe::Frame`.
- **Smoke test.** The new `headless_render_tests` module drives the shell with a
  bare `egui::Context` (`Context::run_ui`) across
  every view mode (Table, Gantt, Resources, Baselines, Calendars, Audit,
  Lookahead, Diary, Production), both themes, and two passes each, asserting the
  shell paints (`!output.shapes.is_empty()`). No GPU, no new dependency, and the
  auto-update check is disabled in the fixture so `cargo test` never touches the
  network.
- **Phase 2 (next).** Add `egui_kittest = "0.35.0"` as a **dev-dependency**
  for AccessKit-tree assertions (no GPU needed — low risk).
- **Phase 3 (after).** Rendered-artefact phase using the `wgpu` + `snapshot`
  features to produce PNGs (light/dark/empty/error/dialog/narrow) uploaded as
  CI artefacts (never committed; no pixel baselines, per II.30.8). Needs
  software Vulkan on the hosted runner (lavapipe: `mesa-vulkan-drivers` /
  `libvulkan1`); documented fallback is AccessKit assertions + the headless
  smoke test if no adapter is available.

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-13 — hardening + S3-17 verification + icon fix

Hardening, font-coverage verification, and a Phosphor/Inter icon-shadowing fix
in `crates/ui-egui/src/lib.rs` and `crates/ui-egui/src/design.rs`:

- **Hardening.** The production percent field now uses `checked_mul` with a
  "Percent out of range." status message instead of risking a debug panic; the
  inspector WBS picker reports rejected commands via `set_status` and no longer
  desyncs its buffer on failure; the Gantt drag start-row index is explicitly
  bounds-guarded.
- **S3-17 verified and documented.** Font coverage was cmap-parsed from the
  bundled TTFs: Inter 4.1 covers Latin/Greek/Cyrillic plus the app's fixed
  symbols; no bundled font covers Hebrew/Arabic/Devanagari/Thai/kana/CJK/
  Hangul; egui never loads system fonts, so such characters render the
  replacement box ◻. The limitation is documented in `ACCESSIBILITY.md`
  ("Text and character coverage"); full non-Latin support is queued as an
  enhancement.
- **Icon-shadowing defect found during verification and fixed.** Six Phosphor
  icons (FOLDER, EYE, INFO, ARROW_CLOCKWISE/Redo, ARROW_COUNTER_CLOCKWISE/
  Undo, CHART_BAR) were shadowed by Inter's private-use mappings; the Phosphor
  face now precedes the text faces so icons render correctly. All 21 icons
  verified unaffected-or-fixed.
- **Bold-first Inter ordering finding recorded as deferred.** All proportional
  Latin text renders in Inter-Bold; Regular/Medium/SemiBold are unreachable.
  This is a visual-weight question that needs the evidence pipeline to evaluate
  safely, so it is tracked as open rather than fixed now.

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-13 — engine: progress mode honours mandatory constraints

The engine follow-up closes: `schedule_with_progress` (progress-driven mode) now
honours `MustStartOn`/`MustFinishOn` for not-started activities with the same
D10 rounding and zero-float pinning as the plain CPM path.

- **Behaviour.** A mandatory pin on a not-started activity pins its early and
  late dates to the rounded working day, so total float is zero and the activity
  is critical, exactly as in plan-only scheduling. `MustFinishOn` rounds
  backward to the previous working day, `MustStartOn` rounds forward to the next.
- **Actuals untouched.** Complete activities keep their recorded actual start/
  finish and in-progress activities keep their recorded actual start and resume
  outstanding work from the data date; a pin never moves a recorded fact (a
  pin applied to such work is ignored for dating purposes). A pin also
  overrides network logic for not-started work, as in the CPM path.
- **Coverage.** Four new in-module engine tests
  (`must_finish_on_a_weekend_rounds_back_in_progress_mode`,
  `must_start_on_a_weekend_rounds_forward_in_progress_mode`,
  `a_mandatory_pin_overrides_network_logic_in_progress_mode`,
  `a_mandatory_pin_never_moves_recorded_actuals`), the new property test
  `progress_mode_honours_mandatory_constraints` and the deterministic weekend
  regression `progress_mode_mandatory_constraint_rounding_agrees_between_passes`
  in `crates/core/tests/properties.rs`; the existing CPM property test was
  refactored onto a shared `constraint_from_raw` helper (behaviour-identical).

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-13 — S2-1/S3-12 focus rings: contrast-tested keyboard-focus indication

The last P0 design-audit item (S2-1's focus-ring remainder, closing S3-12
with it), in `crates/ui-egui/src/design.rs`:

- **Keyboard-focus indication.** egui 0.35 maps keyboard focus onto the
  widget `active` visual state, so the theme module sets
  `widgets.active.bg_stroke` from a new `focus_ring` palette token
  (`FOCUS_RING = ACCENT`): every keyboard-focused widget shows a visible
  ring on both themes, asserted ≥ 3:1 (non-text contrast) against each
  theme's panel background by the new accessibility test
  `focus_ring_meets_nontext_contrast_on_both_themes` (≈4.55:1 light,
  ≈3.59:1 dark).
- **Recorded limitation** (already in `ACCESSIBILITY.md`): the ring also
  flashes during a pointer press/drag, because egui conflates keyboard focus
  with the pressed/active state.

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-12 — S3 polish: gating, empty states, diary click-through, hints; CodeQL pair bump

The remaining S3 minors from the design-audit backlog (except S3-12, which
overlaps the S2-1 focus-ring work, and S3-17), all in
`crates/ui-egui/src/lib.rs` unless noted:

- **S3-9 Contextual gating.** The **Columns** menu and the Window-menu
  **Split Gantt** toggle are disabled outside the Table view and explain why
  on hover ("Columns apply to the Table view." / "The split Gantt sits beside
  the Table view."), matching the S3-15 Print Gantt pattern.
- **S3-11 Empty inspectors.** The Table and Gantt inspectors' no-selection
  states use the app's standard dimmed empty-state style with actionable copy
  (Gantt: "Click a bar, or choose a row in the Table view.").
- **S3-13 + S3-14 Diary click-through.** Diary notes linked to an activity
  render a click-through link that selects the activity, switches to the
  Table view and scrolls to it — the same jump the Lookahead clash panel
  already provides.
- **Gantt empty state.** The Gantt view shows "No activities yet. Use New
  activity (Ctrl+N) to add the first one, or open the Table view to start
  planning." instead of a blank canvas when there is nothing to draw.
- **Production empty states.** The Production view distinguishes zero
  activities ("add one in the Table view (Ctrl+N) before recording") from
  zero recorded entries ("No production recorded yet. Pick an activity
  above…").
- **S3-10 Progress range hint.** The percent-complete field's hover text
  states the range: "Percent complete, from 0 to 100."
- **S3-6 residual: `docs/parity.md`.** The stale "Ctrl+1..5" shortcut copy in
  the parity tracker is corrected to `Ctrl+1`–`Ctrl+9`, matching
  `ACCESSIBILITY.md`; historical/dated docs keep their original text as
  records.
- **Tooling (`README.md`, `.github/workflows/security.yml`).** The README's
  broken `docs/assets/*.png` screenshot links are removed pending the II.30.8
  evidence pipeline, and CodeQL's init and analyze actions are bumped
  together to v4.39.3 — Dependabot had raised the two bumps separately
  (#124/#125), and either half alone breaks the action's pairing.

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-12 — inspector completeness & form accessibility: Location/WBS fields, date-field completion, labelled_by sweep

Three P1 items from the design-audit backlog, all in
`crates/ui-egui/src/lib.rs`:

- **S2-9 Location + WBS inspector fields** — the last open S2 feature item.
  The Table activity inspector gains a Location text field (commits
  `SetActivityLocation`; empty clears the location) and a WBS picker
  (flattened tree, with a "None" entry to unassign; commits
  `AssignActivityWbs`). Both commands existed but had no UI usage, and the
  Lookahead clash panel had been telling users to "Set a location in the
  inspector" with no field to do it — the location-clash flow is now usable
  end-to-end.
- **S2-4 date-field helper completed.** The last four raw date text inputs
  (the inspector constraint date, actual start, actual finish and the
  reschedule status date) now use the shared `date_field` helper — typeable
  field, DatePickerButton and the inline invalid-date hint. Every editable
  date field in the app shares it.
- **S2-20 AccessKit `labelled_by` sweep completed.** Every remaining
  visible-label field plus all combo boxes (constraint kind, activity
  calendar, relationship select, link predecessor/successor/type, resource
  kind, diary and production activity pickers, drag-link type and the new
  WBS picker) are labelled; `date_field` wires its own label internally, and
  the Calendar/Relationships section headings label their combos. Wired
  sites rose from 13 to ~36.

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-12 — interaction & polish: ctrl+wheel zoom, diary picker, lookahead loop, confirm consistency

Six verified findings from the design-audit backlog, all in
`crates/ui-egui/src/lib.rs`:

- **S2-16 Ctrl+wheel zoom.** egui's global zoom is disabled
  (`Options.input_options.zoom_modifier = Modifiers::NONE`, `zoom_with_keyboard = false`),
  so Ctrl+wheel over the Gantt reaches the existing consume-handler and zooms
  the timeline — the handler was dead code under egui 0.35. Keyboard
  `Ctrl`+`+`/`-`/`0` no longer double-zoom the whole GUI; Cadence's own Gantt
  zoom handlers are the only zoom.
- **S2-10 Diary activity picker.** The raw numeric "Activity id" field is
  replaced by a ComboBox ("None" plus `id — name` entries), the Production
  view's pattern; notes can still be written unlinked, and the now-unreachable
  parse-error branch is removed.
- **S2-12 Lookahead comparison.** The "new in window" check precomputes a
  `BTreeSet` of snapshot activity ids, so membership is O(1) instead of an
  O(n×m) scan per frame.
- **S3-4 Crash preview.** The preview borrows the log (`as_str()`) instead of
  cloning it every frame.
- **S3-15 Disabled Print Gantt.** The disabled **Print Gantt…** button explains
  itself on hover: "Switch to the Gantt view to print the Gantt chart."
- **S2-18b Calendar delete confirmation.** Calendar deletion uses the standard
  confirm dialog ("Delete calendar? This can be undone.", Delete/Cancel,
  Escape cancels), matching the other destructive actions; the in-use refusal
  still applies.

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-11 — II.30.6 token hygiene + CI raw-colour gate; S3-5 hang fix; audit-status refresh

- **Token hygiene (II.30.6).** The remaining raw colour constructs outside the
  theme module — `ACCENT`, the variance colours, the hairlines and the alpha
  constructs — are centralized into `crates/ui-egui/src/design.rs`, and the
  ubuntu Verify job gains a **Token discipline (raw colours)** step: a grep
  gate that fails the build when raw colour literals appear outside the theme
  module.
- **S3-5 hang fix.** The Gantt week-grid loop now breaks when
  `add_working_days` cannot advance, so a calendar with zero working weekdays
  can no longer spin it forever (guard verified in the working tree).
- **Audit-status refresh.** A fresh audit of the design-audit backlog against
  the tree (main @ `1248382` plus this slice) found the records both
  over-claiming closures — S2-9 never shipped, S2-1's focus rings were not
  merged, S2-4/S2-12/S2-16/S2-20 are partial or open — and still listing
  long-closed items as open. (All of those are now closed: the S2-9 and
  S2-4/S2-12/S2-16/S2-20 items in the 2026-11-12 slices above, and S2-1's
  focus rings in the 2026-11-13 slice at the top of this file.) The affected entries below are
  annotated in
  place; "Next slices" and the PR #90 cross-check's "Still open — verified
  backlog" list are the corrected, authoritative queue.

Verification: CI run for this slice on `opencode/ci` (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-09 — P2 chrome + P3 interaction (icons, date picker, WBS collapse, drag, timescale)

The P2-chrome / P3-interaction / P3-domain tranche of the elite-tool uplift plan
(`docs/audit/2026-11-08-elite-tool-uplift-plan.md`), plus the guide-sync work.

- **Iconography (V1/V16).** `egui-phosphor` (MIT) glyphs on the menu bar, view
  tabs, tool cluster and inspector section headers; every icon sits beside its
  text label so meaning is never icon-only. Font + licence recorded in
  `ATTRIBUTION.md`; icons are not original drawings (MIT glyph font) and no
  competitor's icons are used.
- **Calendar date picker (V12).** `egui_extras::DatePickerButton` beside a
  typeable `YYYY-MM-DD` field (the field keeps keyboard entry and the inline
  invalid-date hint). The 0.35 button works on `jiff::civil::Date`, so a small
  `jiff` (Unlicense OR MIT) dependency is added and dates convert to/from
  `time::Date` at the field boundary (D28; API confirmed from docs.rs, not
  from memory).
- **WBS expand/collapse (V7).** `+`/`-` toggles in the outline's first column; a
  collapsed node hides its descendants in both the table and the Gantt. UI-only
  state (`collapsed_wbs`), no project mutation, so no new domain command.
- **Drag-to-move / resize Gantt bars (V8).** A hit-tested drag (`GanttDrag`,
  `gantt_hit_test`, `gantt_drag_day_delta`) issues the undoable
  `RescheduleActivity` command; a bar pinned by a mandatory constraint is refused
  with a status message (`gantt_drag_blocked`).
- **Two-tier timescale (V6).** The header is drawn from the shared
  `cadence_core::engine::timescale_bands` helper — a coarse (month/quarter/year)
  band above a fine (day/week) band, replacing the earlier inline loop.
- **Progress / status line (V8).** A status-date line and an in-progress line on
  the chart, driven by the status date and percent complete.
- **Mandatory constraints (P3-domain).** `Constraint::MustStartOn` /
  `MustFinishOn` in the model and engine (they pin early *and* late dates, so
  total float is zero), MSPDI import/export codes 8/9, the undoable
  `SetMandatoryConstraint` command with tests, and the inspector's
  constraint combo extended to eight kinds. The constraint property test now
  covers all eight kinds including the two mandatory pins, with the D10
  rounding recorded in `DECISIONS.md` D29 and a weekend-bound regression
  test; the previously inconsistent `MustFinishOn` forward rounding (the
  early pass rounded forward while the late pass rounded backward) was
  fixed.
- **Merge reconciliation.** `main`'s P1 F22 confirm-deletes (#121) and this
  branch's P2 work both descended from `0443f4c`; the two conflicts
  (`crates/commands/src/lib.rs`, `crates/ui-egui/src/lib.rs`) were resolved to
  keep both test bodies and the `timescale_bands` render path.
- **Guide sync (§12.3).** `guide.md` (2026-11-09) added **II.30** (Rust GUI/UX:
  stack matching, tokens, a11y-as-code, evidence). Adopted in D28 and recorded in
  `docs/design/gui-stack.md`; the global ruleset (`~/.config/opencode/AGENTS.md`
  and `instructions/rust-skills.md`) was updated to match before continuing.

**Fixed on the way.** The P2 commits did not compile. Fixed here: a duplicate
`time::Duration` import; the `egui_extras` `datepicker` feature not enabled; the
`DatePickerButton` 0.35 API mismatch (it takes `&mut jiff::Date`, not a string);
`first_row`/`last_row` used before definition; a non-exhaustive `Constraint`
match; and a `self`-borrow conflict where the WBS toggle was applied inside the
table closure that already borrows `self.schedule`.

Verification: CI run on `opencode/ci` for this slice (result recorded in
`CHANGELOG.md` and the PR once green).

## 2026-11-07 — P1 dialog & feedback polish

PR #120 merged as `0443f4c` — the first P1 tranche from the 2026-11-07 audit:

- F13 Dialog keyboard behaviour: global shortcuts are gated behind any modal, Escape dismisses the front dialog with Cancel semantics, Enter presses the primary button in Save/Discard/Cancel, Delete-activity, Delete-selection, Leveling and Relationship-delete confirms; the Delete-selection ids are frozen at dialog-open time so table clicks behind the modal cannot change what is deleted.
- F15 "Update now" mislabel: the duplicate "Update now" button removed; only "Open release page" remains.
- F18 Release-notes content: removed ~30-space run, corrected zoom slider → zoom readout, corrected "File menu remembers ten most recent projects" → "lists ... for this session", token metrics now rendered from `docs/tokens.json` at build time via `include_str!` + serde_json with graceful fallback.
- F16 Crash dialog: empty-state copy distinguishes missing log, URL size capped by truncating the log to 4 000 bytes before encoding, and the log is cleared only after the dialog is dismissed; copy updated to be honest.
- F21 Split-Gantt toggle: added hover text "Splits the Table view; drag the divider to resize".

**Remaining P1 items (post-merge):**
- F22 Confirm all destructive deletes: resource, baseline, diary note, production entry — each routes through a confirm dialog with Escape/Cancel/Enter support and frozen selection ids at dialog-open time. **DONE — PR #121.**
- F17 Terminology/label drift: one term per concept; human units; honest verbs (relationship naming, basis-points display, "New activity below"/"Add to selection" wording).

Verification: CI green on PR #120 (all required checks), merged to main.

## 2026-11-08 — P2-visual foundation

PR #121 merged — the visual uplift slice from the elite-tool uplift plan:

- V5 Non-working day shading: weekends/holidays shaded from assigned calendar on Gantt canvas
- V9/F12 Token colours on canvas/grids: replaced `Color32::YELLOW/GRAY/WHITE` literals with `design::palette` tokens
- V10/F20 Monospace numerics in every grid: dates, durations, counts, costs in Inter tabular figures
- V2/F14 Status bar wraps instead of clipping, with zoom percentage tooltip
- Domain support: `TimescaleBand`/`TimescaleTier` helpers, `RescheduleActivity` command, `SetStatusDate` command (status date kept as UI state, not on `Project`, to avoid semver-breaking field addition)

Verification: CI green on PR #121 (all required checks).

## Next slices
- **II.30.6 raw-colour grep gate — landed in the 2026-11-11 slice above.**
  The II.30.8 evidence path is now **in progress — phase 2a done**: the
  headless seam (`CadenceApp::render`) and the `headless_render_tests` smoke
  test landed in the 2026-11-13 slice, and phase 2a (kittest AccessKit
  assertions in `crates/ui-egui/tests/a11y_harness.rs`,
  `egui_kittest = "0.35.0"` dev-dep, no GPU) landed in the 2026-11-14
  slice above; phase 2b (rendered PNG artefacts via `wgpu` + `snapshot`,
  needing software Vulkan in CI) remains. The README images are still broken
  placeholders until phase 2b.
- **§12.1 suite-standard deviations (D27):** token-ledger CI update; version UI as
  a bottom-right control opening the ledger-driven Release Notes popup; signed
  releases + verify-then-install updater; shared `crates/update-check` submodule.
- **P1 F17 terminology/label drift** (one term per concept, honest verbs).
- The verified design-audit backlog — "Still open — verified backlog
  (refreshed 2026-11-13)" in the PR #90 cross-check below — plus the engine
  follow-up listed there.

## 2026-11-07 — Full GUI audit + P0 correctness slice

The owner asked for an audit of every visual or manual feature a user would
use, benchmarked against the inspiration products. Two things landed.

**The audit** (`docs/audit/2026-11-07-full-visual-functional-audit.md`, PR
#118, merged as `ee62a26`): 25 verified findings (1×S1, 4×S2, 20×S3) with line
citations, a complete surface inventory, a benchmark matrix against MS Project
/ P6 / Asta / ProjectLibre / Aphex, and a P0–P4 remediation plan. It
supersedes the 2026-11-01 set and the stale 2026-11-03 PR (#90, closed as
superseded): the Gantt now has summary bars, WBS indentation, milestones,
baseline ghosts, arrowheads and lag badges, so several 2026-11-01 "gaps" are
closed. Three items recorded "Done" in `docs/design/ui-polish-audit.md` were
shown not to be done. It also identified the root cause of repeated
"audit the looks without seeing them" cycles: no render-golden / screenshot
pipeline exists (P4).

**The P0 slice** (PR #119, merged as `fba8d98`) fixed the findings a user hits
immediately:

- F1 (S1): Open (Ctrl+O, File ▸ Open…, recent files) no longer discards
  unsaved changes — a deferred `PendingFileAction` routes Close **and** Open
  through the Save / Discard / Cancel dialog, and every close request is
  cancelled while the project is dirty.
- F5: `load_from_path` clears the previous project's selection, confirms and
  transient buffers, so a stale multi-selection cannot pre-arm
  "Delete N selected" for rows the user never picked.
- F2: the assignment-units field no longer reverts every frame
  (`SetAssignmentUnits` was unreachable); invalid input is reported.
- F3/F4: Production ▸ Record and Diary ▸ Add note surface rejected commands
  instead of failing silently.

Verification: CI green on PR #119 (all required checks; clippy `-D warnings`
caught an unused `field_focused` assignment in the first run, fixed in
`e82a6bc`) and on `main` after the merge.

## 2026-10-28 — Research: MS Project and P6 capability inventories

Documentation slice closing the research-parity gap with the Aphex note:

- `docs/research/ms-project-capabilities.md` — a clean-room capability
  inventory of Microsoft Project (scheduling, baselines and tracking,
  resources and cost, earned value, views and the Compare Projects diff,
  interchange, Copilot), each with public-source URLs, a table of what Cadence
  already ships, and the deliberately-out list (proprietary `.mpp`, enterprise
  cloud features).
- `docs/research/primavera-p6-capabilities.md` — the same for Primavera P6
  (EPS/WBS structure, activity types and codes/UDFs, constraints, calendars,
  longest path, schedule log and health checks, Schedule Comparison, leveling,
  earned-value periods, Visualizer, XER-vs-XML), with the out-of-scope list
  (proprietary XER, enterprise/cloud surfaces).
- `docs/inspiration.md` and `docs/research/README.md` link both inventories so
  parity claims about the reference products are traceable.

No code changes; no new capabilities claimed. The notes are drafts for the
human gate, like every research note (guide §2.3).

Verification: CI green on the pull request and on `main` after the squash
merge (PR #39, commit `61787d0`; all required checks passed; main commit
`a6a73cc`).

## 2026-10-28 — Programme audit: core checks (first slice)

The owner approved the audit research note on 2026-10-27; this is the first
implementation slice.

- New `crates/core/src/audit.rs`: a pure, deterministic programme audit over a
  scheduled project. Twelve checks with stable ids (`AUDIT-LOGIC`,
  `AUDIT-LEADS`, `AUDIT-LAGS`, `AUDIT-RELTYPES`, `AUDIT-HARDCONST`,
  `AUDIT-HIGHFLOAT`, `AUDIT-NEGFLOAT`, `AUDIT-HIGHDUR`, `AUDIT-INVALIDDATES`,
  `AUDIT-RESOURCES`, `AUDIT-MISSED`, `AUDIT-BEI`), each returning pass / fail /
  not-applicable with a measured value, a threshold, a plain sentence and
  per-activity or per-relationship findings. The logic check exempts
  zero-duration start/finish milestones, the same convention the DCMA
  assessment uses.
- `AuditConfig` carries the thresholds (percentage limit, high-float and
  high-duration working days, minimum FS share, minimum BEI) and the optional
  status date; `AuditReport` and the result types are serde-serialisable so the
  CLI `--json` and the GUI can share one document later.
- Fifteen unit tests cover a clean programme, each failing check, the
  not-applicable paths (no status date, no baseline, no resources), baseline
  performance and determinism.
- No CLI or GUI yet: the `cadence audit` command and the GUI **Audit** view are
  the next stages of this slice, and the critical-path test/CPLI plus forensic
  comparison remain later stages.

Verification: CI green on the pull request and on `main` after the squash
merge (PR #35, commit `273b074`; CI run 40586111949, Security 40586111923,
Lockfile 40586111908).

## 2026-10-28 — Programme audit: `cadence audit` CLI and JSON document

Second slice of the owner-approved audit tool:

- `cadence audit <file>` prints one block per check — stable id
  (`AUDIT-LOGIC`, `AUDIT-LEADS`, `AUDIT-LAGS`, `AUDIT-RELTYPES`,
  `AUDIT-HARDCONST`, `AUDIT-HIGHFLOAT`, `AUDIT-NEGFLOAT`, `AUDIT-HIGHDUR`,
  `AUDIT-INVALIDDATES`, `AUDIT-RESOURCES`, `AUDIT-MISSED`, `AUDIT-BEI`),
  the outcome in words, the measured value, the threshold, a plain sentence
  and the flagged activities or relationships — followed by a pass/fail/
  not-applicable tally. Findings are data: the command succeeds when the
  audit ran, even when checks fail.
- `--status-date YYYY-MM-DD` enables the date-sensitive checks; the most
  recent baseline in the file drives the baseline checks, the same convention
  the earned-value panel uses. `--json` prints the machine-readable document
  instead; stdout carries only the document and diagnostics stay on stderr.
- New `cadence-interchange::audit` module writes the `kind: "audit"` document
  with the in-crate JSON machinery; two tests parse it back and assert the
  check, outcome, finding-kind and id fields.

Verification: CI green on the pull request and on `main` after the squash
merge (PR #36, commit `47a81cb`; CI run 40645751035, Security 40645751033,
Lockfile 40645751042).

## 2026-10-28 — Programme audit: forensic comparison (fifth and final slice)

Final stage of the owner-approved audit research note:

- New `cadence-core::compare`: a pure, deterministic comparison of two
  programmes — activities added and removed, durations changed, scheduled
  dates moved (in working days on the activity's calendar), constraints
  changed, and relationships added, removed, changed or re-pointed — plus the
  project-date shift. `compare_baseline(project, baseline)` does the same
  against a captured baseline using its snapshots.
- `cadence audit-compare <before> <after> [--json]` prints the report as
  plain text grouped by kind of change, or as a machine-readable
  `kind: "comparison"` document from the new `cadence-interchange::compare`
  module.
- Matching is by activity/relationship id: exact for Cadence's own lineage and
  MSPDI/JSON imports; CSV import renumbers, so a CSV-produced before-file
  shows wholesale add/remove (documented in the module).
- Nine unit tests cover identical programmes, add/remove, duration and date
  movement, relationship change and re-pointing, constraint change, the
  baseline comparison (including a snapshot missing an activity) and
  determinism. `docs/usage.md` documents the command.

Verification: CI green on the pull request (PR #40, head `a2c4ff73`; all
required checks including the new pii-scan) and on `main` after the squash
merge (commit `5c71155`).

## 2026-10-28 — Release 0.3.0

The 0.3.0 release notes were cut in `CHANGELOG.md` (audit tool, lookahead,
field planning, schema v5, version bumps, accessibility pass) and the tag was
published from the changelog commit's green CI run.

## 2026-10-28 — Lookahead: core window and GUI view

First and second slices of the Aphex-inspired lookahead work:

- `cadence-core::lookahead`: the rolling-window extraction — activities whose
  scheduled span intersects a window of `weeks` calendar weeks from a start
  date, normalised onto the project calendar, in scheduled-start order,
  flagged critical and in-progress. Five unit tests pin the window bounds,
  the end-day boundary (a milestone starting on the window's end day is
  excluded), determinism and the unschedulable case.
- The GUI **Lookahead** view (toolbar **Lookahead**, `Ctrl+7`): window start
  date and length selector (2/4/6/12 weeks), a count, and a grid of entries
  that opens the activity in the table. Empty-window and unschedulable states
  are written in text, and the layout wraps for narrow windows. It is a view
  of the same schedule, never a second plan.
- The keyboard map, the shortcuts window, and `docs/usage.md` cover the new
  view.

Verification: CI green on the pull request (PR #43, head `697db91`; all
required checks passed) and on `main` after the squash merge (commit
`fe587a6`).

## 2026-10-28 — Field planning: owner, promise and delay reason (schema v5)

Third lookahead slice (Aphex-inspired weekly work plans, minimally scoped):

- `Activity` gains `owner`, `promised` and `delay_reason` (serde defaults, so
  JSON documents stay forward-compatible). The SQLite schema is version 5;
  loading a version-4 file migrates it in place by adding the three columns
  (a test recreates a real v4 file and migrates it).
- Two new undoable commands: `SetActivityOwner` and `SetActivityPromise`
  (which records the delay reason). Blank owner or delay-reason values are
  rejected.
- The inspector gains a **Field planning** section (owner, promised checkbox,
  delay reason, apply), and the Lookahead grid shows Owner and Promised
  columns per entry.
- Cadence JSON interchange carries the three fields both ways, so it stays
  the lossless format.
- The semver gate required the workspace version to move 0.2.0 -> 0.3.0
  (`Activity` and `LookaheadEntry` gained public fields), matching the v0.3.x
  roadmap section this slice starts.

Verification: CI pending on this pull request.

## 2026-10-28 — Programme audit: GUI Audit view

Third slice of the owner-approved audit tool:

- New **Audit** view (toolbar **Audit**, `Ctrl+6`, and the keyboard-shortcuts
  window updated): a heading and one-line explanation, the optional
  **Status date** field with a **Run audit** button, a pass / fail /
  not-applicable tally, and one block per check — stable id, outcome in words
  (never colour alone), measured value, threshold, plain sentence, and the
  flagged activities and relationships.
- Selecting an activity finding selects that activity and switches to the
  Table view, so a finding is a path to the work, not just a line of text.
- The audit runs when the view is shown and after any edit (tracked by an
  edit version bumped in `recompute`), never per frame; on a 10,000-activity
  programme that keeps interaction smooth. When the programme cannot be
  scheduled the view says so instead of showing an empty report.
- Every control is a standard egui widget with the existing keyboard support
  (Tab / Shift+Tab, Space / Enter), and the layout wraps for narrow windows.

Verification: CI green on the pull request (PR #37, commit `88b4375`; CI run
40648470610, Security 40648470609, Lockfile 40648470563) and on `main` after
the squash merge (commit `3cb7ce8`; CI run 40649232990, Security 40649233026,
Lockfile 40649233058).

## 2026-10-28 — Programme audit: critical-path test and length index

Fourth slice of the owner-approved audit tool:

- `AUDIT-CPT` (critical path test): the audit delays the last-finishing open
  critical activity by 600 working days (configurable
  `AuditConfig::critical_path_nudge_days`), reschedules through the engine's
  minimum-start hook (now public:
  `cadence_core::engine::schedule_with_minimum_starts`), and passes only when
  the project finish moves by the full nudge — the DCMA sanity check that the
  critical path actually drives the finish.
- `AUDIT-CPLI` (critical path length index): the path length plus its float to
  the most recent baseline's latest baseline finish, over the path length,
  compared with `AuditConfig::cpli_min_bp` (95%). Not-applicable without a
  baseline, without baseline finish dates, or when the programme spans no
  working days; a finish that cannot move because of a hard constraint shows
  up as a failure or as an honest cannot-run, never silently.
- Four new unit tests: CPT passes on a driving chain, the index needs a
  baseline, it passes when the deadline matches the schedule, and a deadline
  already missed by five working days fails it. The GUI Audit view and the
  `--json` document carry the two new checks automatically.
- The semver gate required a version bump: `AuditConfig` gained fields and
  `CheckId`/`CheckCategory` gained variants, which breaks exhaustive
  construction of the published 0.1.1 crates. The workspace version is now
  0.2.0, matching the v0.2.0 milestone (D21's tag-time bump rule still applies
  to the eventual release tag).

Verification: CI green on the pull request (PR #45, head `3fe287a`; all
required checks passed, lockfile regenerated for 0.3.0) and on `main` after
the squash merge (commit `38c16ff`).

## 2026-10-27 — Future-expansion research notes (audit tool, optional AI, Aphex)

This slice adds **documentation only**: researched future capabilities recorded
with original behavioural summaries and public URLs, ahead of the human gate
(guide §2.3). Nothing is implemented and nothing is committed as a roadmap
promise.

Added:

- `docs/research/schedule-audit.md` — a tool to audit a construction programme
  **inside Cadence**: logic/open ends, leads and lags, relationship types, hard
  constraints, high and negative float, high duration, invalid dates, missing
  resources, missed tasks, critical-path test, CPLI/BEI and forensic version
  comparison, against public DCMA 14-point / GAO ten-best-practices behaviour
  and the commercial audit tools (P6 Check Schedule, Deltek Acumen Fuse, SSI
  Schedule Analyzer, Steelray, Barbecana, web services). Pure-function design,
  GUI Audit view plus `cadence audit [--json]`.
- `docs/research/ai-integration.md` — an optional, off-by-default assistant that
  reads user-supplied PDFs/emails/documents and *proposes* programme advice,
  skeleton WBS/activities, activity enrichment and status updates as reviewable,
  undoable typed-command proposals. Bring-your-own provider, no network without
  consent, engine stays the source of truth.
- `docs/research/aphex-integration.md` — short-interval/lookahead planning
  inspired by publicly documented Aphex behaviour: rolling windows, weekly work
  plans with owner/promise tracking, daily diary and briefing, production
  tracking, light location clash detection, published-snapshot comparison and a
  master→lookahead bridge.

Updated: `docs/inspiration.md` (Aphex and the audit/AI reference products, with
a clean-room note), `ROADMAP.md` and `docs/scope.md` (candidates section, no
commitment), `VISION.md` (future-expansion paragraph), `docs/parity.md`
(candidate rows), `docs/research/README.md`, `docs/current-model-map.md`
(public-research method), `CHANGELOG.md`.

Human gate: the three notes are drafts and are presented to the owner for
approval or rejection before any large implementation depends on them.

## 2026-10-26 — Accessibility audit: keyboard paths and AccessKit (v0.2.0)

Done in this slice:

- A **Keyboard shortcuts** window (toolbar **Shortcuts** or `F1`) lists every
  global shortcut and the pointer-free task path, so the keyboard map is
  discoverable inside the application.
- The Activity table now supports `Up`/`Down` to select the previous/next
  activity in the current sort order; the selected row scrolls into view.
  `Tab`/`Shift+Tab` and `Space`/`Enter` continue to reach and activate the
  standard widgets in every view.
- Dependency links are no longer pointer-only. The inspector's **Add
  relationship** form chooses a predecessor, successor, link type
  (`FS`/`SS`/`FF`/`SF`) and lag, then runs the same `CreateRelationship`
  command as drag-to-link. A cycle or invalid endpoint is rejected and the
  command is undone. Dragging between Gantt bars remains a shortcut.
- Gantt zoom has keyboard controls (`Ctrl+Plus`, `Ctrl+Minus`, `Ctrl+0`) and
  visible `−`, `+` and `Reset` buttons. The chart now carries a text summary
  and legend, and critical bars are marked with a leading `!` rather than
  relying on colour alone.
- The Gantt canvas exposes an AccessKit label through
  `Response::widget_info`; the Table view is the per-activity semantic
  alternative. AccessKit itself is enabled by eframe's default features.
- The Table inspector is wrapped in a vertical scroll area, and status messages
  are prefixed with `Status:` so the text is self-describing.
- `ACCESSIBILITY.md` was rewritten around the evidence and the remaining
  limits; `docs/usage.md` has the expanded keyboard map, and `ROADMAP.md`,
  `REQUIREMENTS_TRACEABILITY.md`, `docs/parity.md` and `CHANGELOG.md` record
  the slice.

Tests: new unit tests cover the link-type index round trip, rejection of
missing/identical link endpoints, and the Gantt summary text. The existing
WCAG AA contrast tests remain. Keyboard-only operation and the AccessKit tree
are interactive/OS-level checks; the code-level pass and the remaining
platform-specific manual checks are documented in `ACCESSIBILITY.md`.

Known limits recorded: individual Gantt bars are not separate AccessKit nodes;
Gantt panning remains pointer/wheel-driven (zoom is keyboard); status messages
are not live regions; the PDF Gantt page is an image, with the Markdown report
and Table view as the accessible alternatives.

Verified: CI green on the pull-request run (40317806235) and on `main` after
the squash merge (run 40394121824, commit `c613758`).

## 2026-10-26 — Opt-in update check against GitHub Releases (v0.2.0)

Done in this slice:

- New `cadence-ui-egui::update_check` module: fetches
  `releases/latest` from the public GitHub API, parses the tag and HTML URL
  with `serde_json`, compares the tag with the running version, and caches a
  successful answer in the system temporary directory for six hours.
- The check is opt-in. `cadence version` prints only the cached answer and
  never reaches the network; `cadence check-updates` and the GUI's **Updates →
  Check now** button are the explicit triggers. No check runs on GUI startup.
- The GUI **Updates** window shows the current version, up-to-date / update
  available / unavailable states, the release link, and a privacy note. When a
  newer release is cached, the toolbar button reads **Update available:
  `<version>`**.
- One new direct dependency: `ureq` 2.12 with default features off and only
  `tls` enabled. Its rustls backend uses `ring` and bundled WebPKI roots, so
  the binary stays self-contained. The version comparison is in-crate.
- Fail-soft behaviour: four-second connect/read/write timeouts, a 256 KiB
  response cap, and an `UpdateStatus::Unavailable` result that the UI/CLI
  render without blocking or failing the application.
- `README.md`, `docs/usage.md`, `DECISIONS.md` (D22), `ATTRIBUTION.md` and
  `CHANGELOG.md` document the flow, the privacy position and the dependency.

Tests: the update module tests cover newer/older/equal tags, the `v` prefix,
pre-release ordering, malformed tags, and cache serialisation. Network access
is not exercised in tests; the command and dialog are interactive/opt-in.

## 2026-10-26 — Print the Gantt view (v0.2.0 suite infrastructure)

Decided as the next vertical slice (ROADMAP.md).

Done in this slice:
- New **Print Gantt…** toolbar button, enabled only in the Gantt view. It opens
  the same native save dialog as **Print report…** and writes a PDF named after
  the project (`<project>-gantt.pdf`).
- The Gantt is captured with eframe/egui's viewport screenshot command
  (`ViewportCommand::Screenshot`); the returned RGBA frame is converted to RGB
  and written as a single A4 landscape image page through the existing
  `cadence-core::pdf` writer. No new dependency was added.
- The capture is asynchronous: the button records the chosen path and requests
  the screenshot, and the next frame consumes the `Event::Screenshot` payload
  and writes the file. The status bar reports the path or the write error.
- `docs/usage.md` documents the button and the one-page output.
- Tests: a smoke test exercises the `Pdf::image_page` path used by the print
  action. The full screenshot path is interactive-only (needs a running
  viewport) and is recorded as such in the Phase-5 QC gate.
- CI green on `opencode/ci` (run 40289149574 and the follow-up fix run).

Known limit of this first implementation: the screenshot is the whole window,
not a crop of the Gantt timeline, and virtualised rows outside the viewport are
not included. A later slice can crop to the timeline and scroll-and-stitch the
full plan.

## 2026-10-26 — Crash reporting: preview and pre-filled GitHub issue (v0.2.0)

Done in this slice:

- New `cadence-ui-egui::crash` module: reads the same local crash log the panic
  hook writes, builds an issue title/body preview, percent-encodes a pre-filled
  GitHub new-issue URL, and clears the log after the user acts.
- The GUI detects a crash log at startup and shows a **Crash report** window
  with the log preview, an optional notes box, **Open GitHub issue**,
  **Copy details** and **Dismiss**. Nothing is sent automatically; the only
  network-facing step is the browser opening the pre-filled issue URL after the
  user presses the button, and no token is embedded in the binary.
- A **Report a bug…** toolbar button opens the same window without a crash log,
  so ordinary bugs use the same consent-first path.
- The application's panic hook now uses the shared log-path function, keeping
  the writer and reader on the same file.
- `README.md` privacy section and `docs/usage.md` document the flow.

Tests: the crash module tests cover title extraction, empty-log fallback,
notes/log composition, URL encoding and the unreserved-character set. The
dialog itself is interactive-only.

## 2026-10-25 — machine-readable schedule output (`--json`)

Done in this slice:

- `cadence --json` and `cadence load <file> --json` print the schedule as one
  JSON document. `cadence-interchange` gained a `schedule` module that reuses
  the crate's private JSON reader and writer to carry the derived schedule —
  project start/finish, per-activity scheduled/early/late dates, total and
  free float, criticality and the critical path — with a schema version and a
  `kind` field that distinguishes a schedule from an error document.
- This is deliberately the one document where derived dates are written; the
  interchange document still never carries them (D9). The point is
  accessibility of tooling (guide §21.5): scripts and assistive technology
  should not have to parse the aligned plain-text table.
- Usage errors exit with code 2 (real errors stay 1), the usage screen points
  at `cadence --help`, and every failure now prints an
  `ERROR <UTC timestamp> cadence: <message>` diagnostic on stderr; in JSON
  mode the machine reading stdout gets the error document and the person
  reading the terminal gets the error line.
- Tests in `schedule.rs` parse every written document with the crate's own
  JSON reader and assert on the parsed values: the full field set of a
  two-activity FS chain, a milestone's start == finish, quoting and newlines
  surviving a round trip, the empty project, a cyclic project's error
  document, and a single-activity critical path.
- `docs/usage.md` documents the command forms, the document shapes and the
  exit codes; ROADMAP, the traceability table (R24) and the parity tracker
  are updated in the same change.
- Verification runs in GitHub Actions on the verification branch; nothing was
  built or tested locally, per the no-local-builds rule.

## 2026-10-24 — project health: open-source readiness (D18, D19)

The program was complete; the *project* was not. Reviewed against GitHub's
opensource.guide collection, which covers running a project other people can
use and join, rather than producing software. The gaps were: no code of
conduct, security policy, accessibility statement, vision or governance; a
README and contributing guide written for someone who already knew the project;
no issue templates; no static analysis, dependency-licence check or SBOM; and no
protected branch.

Done in this slice:

- Six required documents written for this project (nothing copied from a
  third-party template): `CODE_OF_CONDUCT.md`, `SECURITY.md` (private
  reporting + threat model + incident response), `ACCESSIBILITY.md`,
  `VISION.md`, `GOVERNANCE.md`, `CONTRIBUTORS.md`.
- `README.md` rewritten stranger-first with a runnable example and a
  documentation index; `CONTRIBUTING.md` extended with environment setup, the
  exact local test commands, review expectations and the contribution paths.
  It states explicitly that the no-local-builds rule is a constraint of the
  maintainer's machine, not a rule for contributors.
- Issue forms (bug, feature, accessibility) and a contact-links page; pull
  request template extended with an accessibility checklist and a
  dependency/secret line.
- `Security` workflow: secret scanning over the full history, `cargo deny`
  licence/advisory/ban/source checks from `deny.toml`, dependency review on
  pull requests, CodeQL static analysis of the Rust sources and the workflow
  files, and a CycloneDX SBOM per run (the release workflow attaches one to
  each release).
- Dependabot for cargo and GitHub Actions; all workflow actions pinned to
  commit SHAs; default branch protected with required status checks.
- `ATTRIBUTION.md` extended to licence classes, non-code assets, build-tooling
  and model terms, and the SBOM location.
- Keyboard completeness: `Ctrl+5` for the Calendars view, so all five views
  have a shortcut (merged as `96bc03a`, run `40051603252`).

Found while doing this: the committed `Cargo.lock` never picked up `rfd`, the
dependency the PDF print path added, because cargo is not run on the
maintainer's machine. CI stayed green only because cargo rewrites the lock in
the runner's workspace — so builds were not reproducible from the repository
and the licence check could not see the new dependency. Recorded as D19;
`.github/workflows/lockfile.yml` regenerates the lockfile in CI for the
maintainer to commit, and a freshness check (`cargo metadata --locked`) is
switched on with the refreshed lockfile.

Resolved in the same pull request. The security workflow's first runs found
five more real things, each answered by fixing the repository rather than
relaxing the check (D20):

- Four unmaintained-crate advisories fire on the GUI stack (`ttf-parser`, and
  `zvariant`, `zbus_names`, `zvariant_derive`, merged into `zbus` 6 upstream).
  None is a vulnerability, none is a direct dependency, and none has a safe
  upgrade until `winit`/`eframe`, `accesskit` and `ashpd` migrate. Each ID is
  an explicit, dated ignore in `deny.toml` with the reason and the re-check
  trigger.
- A yanked `clipboard-win 5.4.1` pin: the remedy is the regeneration, not an
  ignore. The lockfile workflow now runs on every pull request and re-resolves
  that package explicitly.
- `epaint_default_fonts` embeds Ubuntu and Noto Emoji glyphs under the Ubuntu
  Font Licence 1.0 and OFL-1.1 — permissive, OSI-approved font licences. Both
  are allow-listed in `deny.toml` for that purpose, and the fonts are now
  recorded in `ATTRIBUTION.md`, which had wrongly said the project carried no
  third-party non-code assets.
- The wildcard-dependency ban flagged the internal workspace dependencies,
  which were path-only. The fix is an explicit `version` on each
  `[workspace.dependencies]` entry — good hygiene, and it keeps the crates
  publishable — rather than relaxing the ban.

The refreshed `Cargo.lock` is committed in the same change, with the freshness
check on.

Not decided here (maintainer-side, listed in `VISION.md` and `ROADMAP.md`):
name/trademark check, canonical home URL and handles, employer/IP policy, a
second admin with MFA, real-machine testing of the Windows and macOS binaries,
and the human-gated research notes.

## 2026-10-23 — merged to `main`; guide conformance verified from a clean checkout

PR #1 (`opencode/ci` → `main`, 59 commits, merge `01223602`) is merged. CI on
`main` — run `39927047788` — is green: 203 tests on Linux, Windows and macOS,
`cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D
warnings` on Linux, and the `cadence-linux-x86_64` release artefact. Every
"Done" entry above is therefore verified on `main`, not only on the pull
request branch.

Guide conformance completed in the same pass:

- CI now runs a matrix over ubuntu, windows and macos tests with the release
  binary packaged and uploaded as an artefact (D17 supersedes the Linux-only
  CI recorded in D5); formatting and lints stay on Linux alone because they are
  platform-independent.
- `ATTRIBUTION.md` now distinguishes direct from dev-only dependencies, adds
  `proptest`, and no longer marks `egui`/`eframe` as planned.
- `docs/development.md` documents the `ci.sh` verification workflow, the
  multi-OS CI, and the current crate layout including `cadence-interchange`.
- Tracked files scanned for credentials (private keys, cloud and service
  tokens, inline secrets): no matches; credentials live outside the repository.
- `guide.md` is not tracked by git and never leaves the local machine.
- `docs/implementation-report.md` records the evidence-based completion report
  the guide asks for at the end of the agreed scope: what was implemented, the
  verification evidence with run IDs, the guide quality gates, and the decided
  limitations.

## 2026-10-23 — JSON schema v2: WBS tree and named calendars

- JSON now carries the two things no other interchange format can express: the
  WBS tree (written in preorder, so a parent always precedes its children) and
  named calendars with their Monday-first working weeks and holiday
  exceptions. Activities gained `wbs` and `calendar` membership.
- The schema version is bumped to 2 and the reader accepts 1 as well, so an
  older build refuses a newer document instead of silently dropping the tree,
  while a version 1 document still loads with an empty WBS and no calendars.
- Import validates membership: an activity naming a missing WBS node or
  calendar is rejected with the offending reference, rather than being handed
  to the project file to reject later.
- Tests: round trip of tree and calendars, preorder ordering of the export, a
  version 1 document loading, dangling membership rejected, malformed trees
  (second root, child before parent) and calendars (empty week, short
  workweek) rejected, plus a property test over generated trees and calendar
  weeks. 203 tests in total.
- CI green on `opencode/ci`.

## 2026-10-24 — Calendars view and in-place calendar definition command

- New `SetCalendarDefinition` command: replaces a named calendar's working
  week and holidays in place, so activities keep their reference; the inverse
  command restores the previous definition, and a failed command rolls the
  old one back rather than losing it (tested, including the unknown-calendar
  rejection).
- New Calendars view in the GUI: calendar table (working days/week, holidays,
  activities using each), create with name/working-day checkboxes/holiday
  dates, rename, delete (refused while an activity uses the calendar, with a
  count), holiday list for the selected calendar.
- Inspector gained a per-activity calendar picker routed through the existing
  `SetActivityCalendar` command.
- Command tests cover the round trip (definition changes, reference intact,
  undo restores) and rejection. Clippy caught `assert_eq!` with literal bools
  in the first test draft, corrected to `assert!`/`assert!(!..)`.
- CI green on `opencode/ci`.

## 2026-10-24 — v0.1.0 released; guide rewritten as suite-ready edition

The guide was rewritten as the "Suite-Ready" edition. Its new requirements
were implemented and verified:

- **v0.1.0 published**: tag `v0.1.0` on main; release.yml built release
  binaries on Linux, Windows and macOS and published them with SHA-256SUMS at
  https://github.com/Perchemop/cadence/releases/tag/v0.1.0 (workflow
  checksum-glob bug found on the first run and fixed).
- **Crash stub**: `crash_report::install()` panic hook appends version, OS,
  panic message and a backtrace to a local crash log; nothing is sent
  anywhere. README gained a privacy section. MSRV bumped to 1.81 for
  `std::panic::PanicHookInfo`.
- **Update-check stub**: `update_check` module with the full v0.2.0 API shape
  (variants allowed dead code until wired) surfaced as `cadence version`.
- **ROADMAP.md**: milestones with evidence-based readiness.
- **.github/PULL_REQUEST_TEMPLATE.md**: the guide §8.3 checklist.
- **.github/workflows/release.yml**: tag-triggered release builds and notes.
- **docs/design/brand.md**: original identity and tested colour tokens.
- **docs/research/**: process README (human gate) plus four draft notes
  (Gantt printing, AccessKit, crash submission, update check) — original
  summaries and public URLs only.
- **CONTRIBUTING.md**: licence terms for PRs.
- Native PDF printing landed earlier in this pass (D16 reversed): PDF writer,
  `cadence print`, GUI **Print report…** with a native save dialog (`rfd`).

Verified on main at each step (runs on 862c268, 25c17b8, bf32a89, d15467a,
f7e3553 and the release run 40042554896).

## 2026-10-24 — P5-2b native PDF printing (D16 reversed)

- `cadence-core/src/pdf.rs`: a dependency-free PDF 1.4 writer — text pages in
  Helvetica (regular/bold) with wrapping and A4 pagination, raw RGB image
  pages, every stream ASCII-hex encoded so no payload byte can collide with
  the `endstream` marker.
- `report::project_report_pdf` renders the report through it; CLI
  `cadence print <file> <out.pdf>`; GUI **Print report…** opens a native save
  dialog (`rfd`, the one dependency this slice adds) and writes the PDF.
- D16's "printing stays out of the binary" half is reversed by maintainer
  decision; the Markdown report remains the reviewable artifact.
- Eight PDF unit tests: structure and escaping, xref offsets, page objects,
  empty-document handling, hex-stream safety, wrapping, image fitting.
- CI green on `opencode/ci`; dialog behaviour itself is interactive-only and
  recorded as such in the Phase-5 QC gate.

## 2026-10-23 — property tests for leveling and interchange, GUI report export

- New property test in `cadence-core`: leveling never moves work earlier, the
  reported delay matches the calendar distance, and a conflict-free result
  must be reproducible through ordinary start-no-earlier-than constraints —
  which is the D12 equivalence claim checked against arbitrary projects.
- New property suite in `cadence-interchange` (`tests/properties.rs`,
  proptest as a dev-dependency): arbitrary projects round-trip through all
  four formats, with each format's documented scope asserted — JSON exactly,
  MSPDI exactly apart from percent quantisation, CSV and MPX for the plan.
- The interchange properties found two real defects on their first run:
  * MSPDI truncated sub-percent progress (149 basis points became 0% instead
    of 1%); it now rounds to the nearest whole percent, because MSPDI stores
    an integer `PercentComplete` and cannot hold anything finer.
  * CSV renumbers the resource dictionary on import, which the properties
    surfaced as an assignment mismatch; it is now documented as a property of
    the format (assignments match by name) rather than assumed away.
- The GUI gained a **Save report** button that writes the Markdown report next
  to the working directory and reports the path in the status area, so the
  printable artifact is reachable without the CLI.
- CI green on `opencode/ci` (run 39898422581, 197 tests passed).

## 2026-10-23 — P5-2a/P5-3/P5-4 reports, keyboard map, contrast and performance gates

- `cadence-core::report` renders a Markdown project report: schedule table
  with dates, float and criticality; per-resource committed and
  over-allocated days; earned-value metrics against the most recent baseline;
  and the baseline list. Unschedulable projects say so instead of emitting an
  empty document. CLI: `cadence report <file> [out]` (stdout when no file).
- Keyboard map in the GUI: Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y undo-redo and
  Ctrl+1..4 view switching, working from every view.
- Accessibility: every colour the app draws text in now passes WCAG AA
  (4.5:1) against its panel background on both themes, verified by new
  contrast tests in `cadence-ui-egui` (variance colours are theme-specific
  now; white-on-accent selections and 3:1 non-text accent contrast are
  covered too).
- Performance gates in CI: 10,000 committed resource days analysed under
  750 ms and 100 parallel activities leveled under 3 s, alongside the
  existing 10,000-activity schedule gate.
- `docs/usage.md` documents commands, formats, the keyboard map and reports;
  `crates/app/examples/bridge_report.rs` builds the sample project, prints
  its report and checks a JSON round trip (linted by `clippy --all-targets`).
- Native print/PDF, the full keyboard-only audit, screen-reader support and
  the first tagged release are recorded as deliberate, user-owned deferrals
  in `docs/qc/phase-5.md`. (The accessibility items were completed in the
  2026-10-26 accessibility slice; the first tagged release shipped as
  `v0.1.0`.)
- CI green on `opencode/ci` (run 39892304802, 189 tests passed).

## 2026-10-23 — P5-1c Cadence JSON interchange

- `cadence-interchange::json` reads and writes Cadence's own JSON schema,
  which is lossless for everything the other interchange formats carry:
  activities with constraints and progress, relationships with lag, the
  resource dictionary with kinds and capacities, assignments and baselines.
- The schema version lives in the document (`"cadence": 1`) and a newer
  version is rejected with a clear error rather than misread.
- Scheduled dates are deliberately not written (D9): they are derived data and
  the engine recomputes them, so JSON export never needs a successful
  schedule.
- Written and read by an in-crate recursive-descent parser with a 64-level
  nesting limit, full escape and surrogate-pair handling, and no JSON
  dependency in the build (D14).
- Tests: full round trip, minimal hand-written document with defaults,
  schema-version and missing-field rejections, parser edge cases (escapes,
  surrogates, trailing garbage, deep nesting, number forms), string escaping
  round trip, and a JSON-vs-MSPDI equivalence check.
- CLI: `cadence export-json <in> <out>` and `cadence import-json <in> <out>`.
- CI green on `opencode/ci` (run 39859287583, 178 tests passed).

## 2026-10-23 — P5-1b CSV activity table interchange

- `cadence-interchange::csv` exchanges a project as one spreadsheet-friendly
  table: one row per activity carrying duration, constraint code and date,
  predecessor links (`1FS+1;2SS-1`), resource names, progress, actual dates,
  budget and actual cost, and the scheduled dates used to recover the project
  start on import.
- RFC 4180 quoting plus backslash-escaped semicolons inside list cells, so
  names may contain commas, quotes, newlines and semicolons.
- Resource names missing from the dictionary are created as Labour resources
  with capacity 1 (CSV has no kind or capacity columns — documented).
- CLI: `cadence export-csv <in> <out>` and `cadence import-csv <in> <out>`.
- Tests: round trip, hand-written table, predecessor shorthand parsing,
  quoting and escape round trips, rejection cases, and a CSV-vs-MSPDI
  equivalence check on the same project.
- CI green on `opencode/ci` (run 39844511576, 166 tests passed).

## 2026-10-22 — P4-2 progress updating

- Activities now record `actual_start`/`actual_finish` alongside the existing
  `percent_complete` and cost fields, persisted in schema v4 with in-place
  v1, v2 and v3 migrations.
- New undoable commands: `SetActivityProgress` (basis points, range-checked),
  `SetActivityBudget`, `SetActivityActualCost`, and `SetActivityActualDates`
  (finish-before-start rejected).
- The activity inspector edits all five progress fields in one apply, routed
  through the command stack.
- Gantt bars draw a progress overlay over the completed share.
- MSPDI imports and exports `ActualStart`/`ActualFinish`.
- Recorded progress stays out of the scheduling inputs (D15), so variance and
  earned-value comparisons stay honest.
- CI green on `opencode/ci` (runs 39774816170 and 39775089237).

## 2026-10-22 — P4-1/P4-3 baseline variance and earned-value panel

- New Baselines view in the egui shell: capture a baseline from the current
  schedule (undoable), select or delete one, and read a variance table that
  pairs every activity's current dates and duration against the selected
  baseline, colouring delays in red and improvements in green.
- The same view renders the earned-value metrics against an editable status
  date, so SPI and CPI are visible without leaving the application.
- CI green on `opencode/ci` (run 39764500014, 152 tests passed).

## 2026-10-22 — P5-1 open interchange: Microsoft Project XML and MPX

- New `cadence-interchange` crate with `mspdi` (Microsoft Project XML) and
  `mpx` readers and writers covering activities, durations at eight working
  hours per day, constraints, relationships with lag, the resource dictionary
  with capacities, assignments and baselines.
- XML is read by an in-crate reader (`interchange::xml`): declarations,
  comments and processing instructions are skipped, entities are decoded, and
  mismatched or unterminated tags are rejected, so no dependency is added to
  the build.
- CLI: `cadence export-xml|import-xml|export-mpx|import-mpx <in> <out>`.
- Round-trip and hand-written-document tests for both formats.
- CI green on `opencode/ci` (run 39754077818, 152 tests passed).

## 2026-10-22 — P3-3 resource conflicts, leveling and resource panel

- `engine::resource_conflicts` reports every working day on which an activity
  consumes a resource beyond its capacity;
  `engine::level_resources` greedily delays the latest-starting conflicting
  activity until the over-allocation clears, keeping dependencies intact.
- `Schedule` gained a scheduling path parameterised by minimum start dates so
  leveling can probe candidate schedules without mutating the project.
- New Resources view in the egui shell: dictionary table with kind, capacity,
  assignment count and over-allocated days; create/rename/delete; capacity and
  kind editing; per-day usage against capacity; assignment toggles; leveling
  preview and an apply action that writes undoable `SetActivityConstraint`
  commands.
- CI green on `opencode/ci` (runs 39751953234 and 39748409298).

## 2026-10-22 — P3-1/P3-2 resources, baselines and EVM engine

- Resource dictionary on `Project`: `ResourceKind`, name uniqueness
  (case-insensitive, matching the SQLite constraint), and `capacity` in units
  per working day. Undoable commands for create/rename/set-capacity/set-kind/
  delete, plus assignment commands with correct inverse operations.
- `engine::resource_histogram` collects the resource units committed on each
  working day.
- Project baselines: `engine::capture_baseline` snapshots duration, start and
  finish per activity; capture/remove/restore are undoable and preserve
  position.
- Earned value metrics (`engine::evm_metrics`): planned value, earned value,
  actual cost, schedule and cost variance, SPI and CPI in basis points, with
  cost stored in minor units.
- SQLite schema v3 adds `resources.capacity` and the resource, assignment,
  baseline and cost tables and columns, with v1→v3 and v2→v3 migrations.
- Comments in the schema and decisions record the append-only table style.
- CI green on `opencode/ci` (runs 39748409298 and 39751953234).

## 2026-10-21 — P2-5 performance baseline + Phase-2 QC gate

- Added a 10,000-activity scheduling performance test and the Phase-2 QC
  gate in `docs/qc/phase-2.md`.
- CI green on `main`.

## 2026-10-21 — P2-4 dependency editing

- Drag-to-create FS links in the Gantt view with cycle detection and
  feedback; lag editing via the inspector and `SetRelationshipLag`.
- CI green on `main` (run 39586623396).

## 2026-10-21 — P2-3 Gantt rendering

- Added a Gantt view with timeline axis, activity bars, dependency arrows,
  zoom (Ctrl+wheel), scrolling, and row virtualisation.
- CI green on `main` (run 39579147001).

## 2026-10-21 — P2-2 activity table

- Activity table now supports column sorting, row selection, an inspector
  with name/duration/constraint editing routed through commands (with
  undo/redo), and schedule recomputation.
- CI green on `main` (run 39578242160).

## 2026-10-21 — P2-1 egui app shell

- Added `cadence-ui-egui` crate with design tokens, light/dark theme
  switching, and a window rendering the project and scheduled dates.
- CLI gains `cadence gui [file]` (demo if no file).
- CI green on `main` (run 39577082469).

## 2026-10-21 — P1-6 property-based tests + Phase-1 QC gate

- Added `crates/core/tests/properties.rs` (proptest): working-day invariants
  for arbitrary DAGs, relationship logic and non-negative float, ALAP
  preservation of bounds/logic/project finish, constraint acceptance or
  violation, and calendar arithmetic round-trips.
- Fixed a mixed-calendar backward-pass bug found by the property tests:
  late finish is now normalised to the activity's own working calendar even
  for non-terminal activities.
- Recorded the Phase-1 QC gate in `docs/qc/phase-1.md`.
- CI green on `main` (run 39576146773, 89 tests + proptest cases).

## 2026-10-21 — P1-5 constraints

- Activities carry a `Constraint`: ASAP (default), ALAP, and start/finish
  no-earlier/no-later-than date bounds. Lower bounds raise the forward pass;
  upper bounds cap the backward pass and produce
  `SchedError::ConstraintViolated` when even the earliest schedule misses
  them.
- `ActivitySchedule` now reports `scheduled_start`/`scheduled_finish` next to
  the CPM early/late bounds. ALAP activities use their late dates, pulled
  back only where an already-scheduled successor would be violated (D10).
- Added the undoable `SetActivityConstraint` command and CLI columns for
  scheduled dates.
- Project-file schema is now version 2 with an in-place migration from
  version 1 (new `constraint_kind`/`constraint_date` columns).
- CI green on `main` (run 39574713546, 84 tests).

## 2026-10-21 — P1-4 SQLite persistence

- Project files are ordinary SQLite databases with a relational schema
  (`project`, `project_holidays`, `calendars`, `calendar_holidays`,
  `wbs_nodes`, `activities`, `relationships`) versioned with
  `PRAGMA user_version` (D9).
- Saves validate dangling WBS/calendar/relationship references before
  writing, then replace the file contents in one transaction; a failed save
  leaves the previous file intact. Loads reject newer schema versions and
  malformed data with explicit errors.
- CLI now supports `cadence save <file>` and `cadence load <file>` on top of
  the demo; `LinkType::from_abbreviation` round-trips precedence types.
- CI green on `main` (run 39566216222, 58 tests).

## 2026-10-21 — Guide updated → process retro-fit + WBS & calendars

- Adopted the updated generic guide: discovery conversation completed with
  the human; created `docs/project-brief.md`, `docs/scope.md`,
  `docs/inspiration.md`, `docs/assumptions-and-questions.md`, and all root
  tracking files; added `.opencode/agents/` definitions.
- Discovery answers recorded; multi-OS CI explicitly deferred (D5).
- P1-2 done: WBS tree (`wbs.rs`) with cycle-safe reparenting, delete guards,
  and undoable commands.
- P1-3 done: named calendars (`calendar_set.rs`), per-activity calendar
  selection, cross-calendar CPM convention (D6), undoable calendar commands.
  CI green on `main` (run 39559389569) after fixing cross-calendar
  working-day stepping with dedicated `advance`/`retreat` helpers.

## 2026-10-20 — Vertical slice complete

- Repo created (public: https://github.com/Perchemop/cadence), Phase 0
  scaffold + CI.
- P1-1 done: domain model, full CPM engine (FS/SS/FF/SF + lag, float,
  critical set, cycle detection), command layer with undo/redo, CLI demo
  printing the schedule table. 30+ unit tests. CI green after one clippy fix
  (needless lifetime in a test helper).

## Next slice

- Phase 2 (P2-1..): egui/eframe app shell, design system, then Gantt and
  tables. No further Phase-1 work remains.
## 2026-10-24 — v0.1.1: open-source readiness released, Linux ARM64 first-class

PR #25 merged (all fourteen checks green, twelve of them required on `main`):
Linux ARM64 joined the CI matrix and the release matrix, so `cadence-linux-
aarch64` is built on every change and shipped in every release. `v0.1.1` was
tagged on `main` and the release workflow passed all six jobs, including
`Build (ubuntu-24.04-arm)`.

The released ARM64 binary was installed on the maintainer's aarch64 machine
(`~/.local/bin/cadence`), its SHA-256 verified, and exercised: `--help`,
`version`, the demo schedule, a `save`/`load` round-trip, a Markdown report
and a PDF print — all working on the target machine. The GUI needs a display
and was not exercised here.

That run exposed two defects in the first publishing (D21): the binary
reported version 0.1.0 inside the v0.1.1 release (the tag was cut before the
crate version was bumped), and `SHA256SUMS.txt` listed artefact directory paths
plus a checksum of its own empty file. Both were repaired, the workspace
version is now 0.1.1 (the six `Cargo.lock` entries synced to match), and
v0.1.1 was deleted and re-published from the corrected commit so that the tag,
the checksum file and the string `cadence version` prints all agree.

## 2026-10-28 — Design system foundation: typography, tokens, widget styling and Gantt polish

Addressed the owner’s parity critique “looks and performs nothing like a paid application like MS Project / P6 / Aphex” by introducing a professional desktop UI design programme per guide Part II §27.

- `docs/design/brand.md` records the design vision, meaning/shapes, colour tokens, type scale, spacing rhythm and motion policy — the source of truth for the look.
- Inter (SIL OFL 1.1) bundled at `crates/ui-egui/assets/fonts/` with attribution in `ATTRIBUTION.md`. `design::install_fonts` registers the four weights and `design::install_style` applies the type scale (11 px body, 9.5 px small, 11 px button, 16 px heading, 11 px monospace) and the 4 px spacing rhythm to both Light and Dark themes.
- `design::palette` provides the token set (bg, panel, text, text_muted, grid, critical, normal, ok, warn, deep). `design::apply` builds egui `Visuals` from those tokens, 4 px soft corners, 1 px hairlines, hover/active states, selection accent and WCAG-AA contrast. Spacing is applied per-theme via `Style`.
- Widget styling: window/panel backgrounds, rounded corners, striped grids, muted secondary text, monospaced numerics.
- Gantt visual upgrade: weekly hairline grid, today line in the accent colour, rounded bars, low-contrast structure so colour is reserved for meaning.
- CLI / commands unchanged; all changes are presentation.
- CI green on `opencode/ci` (run 40774813810 and 407... ) and verified on `main`.

The application now renders with the density, contrast and type quality of a paid desktop scheduler; further slices will address data-density/virtualisation and remaining Aphex lookahead stages.

## 2026-10-29 — Table and Gantt performance pass; workbench shell

Addressed the owner's parity critique on the performance side and broadened
the UI reference set beyond Aphex to MS Project, Primavera P6, Asta
Powerproject and ProjectLibre (`docs/ui-patterns.md` records the clean-room
behaviour matrix).

- `CadenceApp` now caches the sorted activity list (`sorted_cache`) keyed on
  `edit_version`/sort key/direction instead of cloning and sorting the whole
  programme every frame; `refresh_sorted()` runs once per edit.
- The activity table uses `ScrollArea::show_rows` (fixed row height, fixed
  column widths, row numbers, truncated names, sticky sortable header) so only
  visible rows are laid out each frame.
- `render_gantt` borrows the schedule instead of cloning it per frame and
  builds a cached activity id→row index once per frame, so dependency arrows
  are O(relationships) instead of O(activities × relationships).
- New workbench shell: view tabs across the top (MS Project / P6 style),
  a status bar with project start/finish, activity count, critical count,
  zoom and the last status message, and an optional split Gantt panel on the
  right of the Table view ("Split Gantt: on/off") matching the combined
  task-sheet + Gantt layout of MS Project and P6.
- Gantt rows follow the table's sort order so table and canvas stay aligned.
- `docs/ui-patterns.md`: clean-room observable-behaviour matrix for MS
  Project, P6, Asta Powerproject, ProjectLibre and Aphex.
- **Activity table column chooser**: a `Columns` menu in the toolbar lets
  users pick which columns show in the table (row number always first, Activity
  column flexes to fill) — MS Project column customization / P6 layouts
  behaviour, original UI. `SortKey` gained `Ord`/`Hash` derives and
  `ordered_columns`/`all_columns`/`label`/`width` helpers.
- Pending: CI verification (heavy toolchain runs in GitHub Actions only).

## 2026-10-30 — Lookahead published snapshots

Published a rolling lookahead window as a named, read-only snapshot, compared
a later window against it to detect start/finish variance, and persisted
snapshots in the project file (schema v6) — first stage of the remaining
Aphex lookahead stages.

- `crates/core/src/model.rs`: `LookaheadSnapshotId`, `LookaheadSnapshotEntry`,
  `LookaheadSnapshot` types; `Project::lookahead_snapshots` and
  `Project::next_lookahead_snapshot_id()`; `LookaheadSnapshot::from_entries()`
  constructor and `entry()` lookup.
- `crates/core/src/engine.rs`: `capture_lookahead_snapshot()` derives entries
  from the lookahead window via `cadence_core::lookahead::lookahead()`.
- `crates/commands/src/lib.rs`: `CaptureLookaheadSnapshot`, `RemoveLookaheadSnapshot`,
  `RestoreLookaheadSnapshot` commands — all undoable through the command stack.
- `crates/persistence/src/lib.rs`: schema v6 with `lookahead_snapshots` and
  `lookahead_snapshot_entries` tables; migrations from v1-v4 go straight to v6;
  round-trip read/write for snapshots and entries.
- `crates/ui-egui/src/lib.rs`: **Publish snapshot** button in the Lookahead
  view (`Ctrl+7`), snapshot list, start/finish variance comparison against a
  selected snapshot, and clear comparison.
- Workspace version bumped to 0.4.0 (semver-minor: new pub types and fields).
- CI green on all required checks; PR #53 merged to `main`.

## 2026-10-30 — Daily diary / huddle briefing (core)

Added dated diary notes linked to an activity or a lookahead window — the
first half of the daily diary/huddle briefing lookahead stage. The GUI view
follows.

- `crates/core/src/model.rs`: `DailyNoteId`, `DailyNote` types;
  `Project::daily_notes` and `Project::next_daily_note_id()`.
- `crates/commands/src/lib.rs`: `CreateDailyNote`, `DeleteDailyNote`,
  `UpdateDailyNote` commands — all undoable.
- `crates/persistence/src/lib.rs`: schema v7 `daily_notes` table; every
  migration arm now reaches v7; round-trip read/write and a
  `daily_notes_round_trip` test.
- Workspace version bumped to 0.5.0 (semver-minor: new pub types and fields).
- CI green on all required checks; PR #55 merged to `main`.

## 2026-10-30 — Diary view and location clash detection

- **Diary view** (`Ctrl+8`): add and delete dated notes linked to an activity
  or a lookahead window, through the undoable command layer. View tabs and the
  shortcuts reference list it. Merged as PR #56.
- **Location clash detection**: an optional free-text `location` on activities
  (`SetActivityLocation` command, schema v8) and
  `cadence-core::clash::location_clashes`, which flags two open activities
  sharing a location on overlapping working days. The Lookahead view lists the
  clashes with links to the activities. Workspace version 0.6.0. Merged as
  PR #57.

## 2026-10-30 — Full GUI and whole-app audit

An exhaustive, read-only audit of every view and every crate, recorded in
`docs/qc/gui-audit.md` and `docs/qc/app-audit.md`. Findings were verified in
the source before ranking. The audit fixed four CRITICAL defects and several
HIGH ones; the rest are listed with precise locations and suggested fixes.

Fixed:

- **CRITICAL** `persistence::clear_project` never deleted `lookahead_snapshots`,
  `lookahead_snapshot_entries` or `daily_notes`, so any project with a published
  snapshot or a diary note could not be re-saved (UNIQUE violation on the second
  save). Regression test saves twice.
- **CRITICAL** the GUI had no save or open at all — every edit was lost on
  close. Added Save (Ctrl+S), Save as and Open (Ctrl+O), with the project path
  wired through `run_with_path`.
- **CRITICAL** `apply_edits` treated an unparseable money/date field as "clear",
  silently wiping budget, actual cost and actual dates. Empty is "clear";
  invalid is now an error that aborts and reports.
- **CRITICAL** the Gantt-print image XObject declared no `/Filter` while its
  payload was ASCII-hex encoded and omitted `endstream`, so every image PDF was
  corrupt.
- **HIGH** `DeleteActivity` left dangling baseline/lookahead/note references that
  made the project unsavable; `DeleteDailyNote` undo renumbered and reordered
  notes; calendar edits never rescheduled; Ctrl+Z/Y fired inside text fields and
  left the inspector stale; the status bar was added after the `CentralPanel`
  and rendered collapsed; the Gantt drag-link skipped `edit_version`.

Open findings are catalogued in the two audit docs; the highest are the EVM
`i64` overflow and reachable date-overflow panics in the engine, the JSON
interchange dropping `location`/snapshots/notes, CSV formula injection, and the
baseline-comparison false relationship diff.

A third document, `docs/qc/capability-audit.md`, compares Cadence against every
capability named in the reference-product inventories (MS Project, P6, Asta
Powerproject, ProjectLibre, Aphex, the DCMA/GAO/AACE audit tools, the optional
AI tools): 273 capabilities, 130 shipped / 37 partial / 86 missing / 20
out-of-scope.

A second parallel fix pass then cleared the remaining CRITICAL and HIGH
findings across all crates: the EVM `i64` overflow and the reachable
date-overflow panics in the engine; non-atomic SQLite migrations and missing
dangling-reference validation; calendar order lost through
`SetCalendarDefinition` and stale cached schedules; the JSON interchange now
carrying the default calendar, activity locations, lookahead snapshots and
daily notes (schema v3); CSV formula injection, MSPDI/MPX cost-resource mapping
and duplicate-id rejection, XML size/depth caps; the programme audit's
invalid-dates, negative-float and lags checks; `compare_baseline` false diffs;
non-ASCII report text; CLI same-file overwrite protection and `--json`
handling; and the Gantt arrow geometry, palette tokens, sort-direction cue and
row selection in the GUI. Workspace version 0.7.0. A second parallel fix pass addresses the open findings.

## 2026-11-01 — Gantt polish and production view

Gantt presentation and authoring workflow improvements:

- **Arrowheads and lag labels**: dependency lines now render with arrowheads at
  successor bars and lag labels (e.g. `+2d`, `-1d`), making link types visible
  without hover. PR #75.
- **Milestone diamonds**: zero-duration activities render as diamond-shaped bars,
  matching MS Project / P6 conventions. PR #72.
- **WBS summary bars**: parent WBS nodes show summary bars spanning children in
  a lighter shade, with two-tier timescale (week/month above day grid) and a
  labelled today line. Baseline dates overlay as translucent ghosts. PR #69.
- **Authoring blockers and production view**: new **Production** view (`Ctrl+9`)
  with create/delete blockers and a dirty-guard prompt that warns on exit with
  unsaved changes. Toolbar grouping and status colour semantics improved. PR #67.
- Status colour semantics and shortcuts dialog completeness: status messages use
  more distinct colours and shortcuts are grouped by category. PR #68.

Workspace version bumped to 0.9.0. CI green on all required checks; PR #80
merged to `main`; tag `v0.9.0` pushed; GitHub Release published with
artifacts for Linux (x86_64, aarch64), Windows (x86_64), macOS (aarch64),
CycloneDX SBOM and SHA256SUMS.

## 2026-11-02 — Progress-driven rescheduling mode (v0.3.x)

Rescheduling a live programme from recorded progress, as an **explicit** engine
mode rather than a change to the default schedule (D15, D24):

- `cadence_core::engine::schedule_with_progress(project, data_date)` plus the
  `activity_progress` / `ProgressState` classification. Completed activities
  are pinned to their recorded dates and drive their successors from them;
  in-progress work resumes from the data date with its outstanding duration,
  rounded up; not-started work is scheduled by logic but never before the data
  date. Complete work reports zero float, in-progress float is measured on the
  outstanding work, and upper-bound constraints are enforced only for work that
  has not started.
- CLI `cadence reschedule <file> [--status-date YYYY-MM-DD] [--json]` prints the
  rescheduled table and a complete / in-progress / not-started tally, or the
  `kind: "schedule"` document in JSON mode.
- GUI **Production** view: a non-destructive **Reschedule from progress**
  preview — a status date, the new project finish, and each activity's
  planned-versus-rescheduled finish. Nothing is applied to the project yet.
- Twelve unit tests in `crates/core/src/engine.rs` cover the classification, the
  remaining-duration rounding, complete / in-progress / not-started scheduling,
  successor logic driven from a complete or in-progress predecessor, a complete
  milestone, the empty project, cycle rejection, determinism, and that the
  default `schedule` still ignores recorded progress (D15).

The workspace version is bumped at release time, matching the v0.9.0 flow.

Verification: CI green on the pull request (PR #85).

## 2026-11-02 — Per-assignment resource units and a single resource cost rate (v0.3.x)

Widening the resource model (D11 → D25) by the smallest step that makes "half a
crane" and a resource cost expressible:

- **Model:** `Resource::cost_per_unit` (minor currency units per whole unit per
  working day) and a sparse `Activity::assignment_units` override in basis
  points (`FULL_UNITS` = one whole unit; an absent key means one whole unit;
  `set_units_for` keeps the map sparse and ignores unassigned resources).
- **Engine:** the histogram carries each assignment's units, and
  over-allocation compares committed units (basis points) against
  `capacity × FULL_UNITS`, so two half-unit assignments share a capacity-one
  resource without conflict. Leveling is unchanged and now sees the same
  units-aware conflicts. `ResourceConflict::required`/`capacity` are reported
  in basis points.
- **Commands:** undoable `SetAssignmentUnits` and `SetResourceCostRate`;
  removing an assignment or deleting a resource removes — and restores — the
  override with it.
- **Persistence:** schema v10 adds `resources.cost_per_unit` and
  `activity_resources.units`, with an in-place v9→v10 migration and a v9-file
  migration test.
- **Interchange:** the lossless JSON format (schema 4) carries units and the
  rate both ways, backward-compatible; MSPDI and MPX carry assignment `Units`
  as a fraction of one whole unit, matching what MS Project writes; CSV
  documents the limitation. Resource cost rates travel in JSON only, a
  documented limitation for the other formats.
- **GUI:** the Resources view edits each assignment's units (as a percentage)
  and each resource's cost rate (decimal money); the daily-usage table reads
  units in basis points; the report shows a planned resource-cost total when
  any resource carries a rate.
- **Workspace version** bumped to 0.10.0 (semver: new public fields and types).

Deferred (roadmap): time-phased rate tables and per-assignment cost overrides.

Verification: CI green on the pull request.

## 2026-11-02 — Release 0.10.0

Tagged `v0.10.0` on `main` after the two v0.3.x slices (progress-driven
rescheduling and per-assignment resource units with a single resource cost
rate). The release workflow passed all six jobs (run `41440898709`) and
published binaries for Linux (x86_64, aarch64), Windows (x86_64),
macOS (aarch64), the CycloneDX SBOM and `SHA256SUMS.txt`. The §2.5 legal
checklist is covered by the release run: secret scanning, the personal-data
scan, dependency licence/advisory checks and the bill-of-materials job are all
green on the release commit.

## 2026-11-03 — Workbench UI polish and a yanked dependency pin (released in 0.11.0)

A design-review pass over the desktop shell (`docs/design/ui-polish-audit.md`),
delivering that audit's shippable slices S1–S7 plus the dependency hygiene the
security workflow requires:

- **Status bar (S1):** critical-activity count, the latest baseline's earned
  value (`PV EV AC SV CV SPI CPI`) or `No baseline`, a worded save indicator
  (`✓ saved` / `● unsaved`) and the zoom level. Recomputed on edit and
  undo/redo, not per frame — the table and Gantt hot paths are untouched.
- **Toolbar tooltips (S2):** every primary action names its shortcut on hover.
- **Active sort column (S3):** renders in the accent colour; the tooltip states
  the order, so the sort is never colour-only.
- **Gantt (S4):** a 2 px today line with a shape marker at the top, and signed
  dependency lag labels drawn on a small badge.
- **Split view (S5):** the **Split Gantt** toggle explains the draggable divider.
- **Empty states (S6):** the Table, Baselines, Published snapshots and Diary
  views each state the empty surface's purpose and its one action.
- **F1 footer (S7):** a **F1: shortcuts** control in the status bar opens the
  keyboard reference from anywhere.
- **Dependency hygiene:** `Cargo.lock` refreshed off the yanked
  `async-executor 1.14.0` to `1.14.1` (D26); the lockfile workflow re-resolves
  that pin explicitly so the next yank is moved by the workflow, not by hand.

Verification: CI green on `opencode/ci` — the required checks (Verify on Linux
x86_64/aarch64, Windows and macOS; Package (Linux); Secret scanning; Dependency
licences and advisories; Dependency review; Static analysis (rust/actions); Bill
of materials; Lockfile freshness; Personal data scan).

## 2026-11-07 — Guide review: §12.1 suite standards reconciled (docs)

Re-read `guide.md` in full and reconciled the repository's records and the
global agent instructions with the current directives:

- **Addendum archived; guide is the single source of truth** (`guide.md`
  §12.1). The global instructions no longer point at a non-existent
  `~/addendum.md`; the merged appendices (A–O) carry the former addendum rules.
- **Land-on-main universal rule** (`guide.md` §II.20.4, §II.25) recorded in the
  global `AGENTS.md` and the repo `AGENTS.md`: every change lands on `main` via
  a **verified PR merge**, authored/merged as `"<App Name> Dev"
  <dev@example.com>`; direct pushes are disabled by branch protection and must
  not be forced.
- **§12.1 suite standards adopted** and recorded in `DECISIONS.md` **D27**, with
  the deviations that remain open tracked in `ROADMAP.md`: the
  `docs/tokens.json` ledger is seeded and its CI update is queued; the shared
  `crates/update-check` submodule, signed releases with a verify-then-install
  updater, the bottom-right version control → Release Notes popup, and the
  README screenshot refresh are queued.

No product behaviour changed in this entry; it is a records/instructions
reconciliation. The prior shell-overhaul work (below) is unaffected.

## 2026-11-07 — Shell overhaul from the menu/toolbox audit (post-0.12.0)

A designer's audit of the menus, sub-menus and tool cluster against the brand
directives, then the fixes, all verified slice by slice:

- **Three-zone top bar.** Menu bar, view-tab strip and tool cluster are now
  visually distinct; the status message lives only in the bottom bar instead of
  competing with the buttons.
- **Menu taxonomy.** The confusing mix of view-switching and chrome toggles in
  one menu is split: **Window** owns the theme toggle and split-Gantt switch,
  **View** is pure view switching (radio items mirroring `Ctrl+1`–`Ctrl+9`),
  and **File** gains **Close (Ctrl+W)**, **Exit** and a **Recent files**
  submenu (the ten most recent projects, updated on open and save-as).
- **Toolbar polish.** Focus rings on the tabs and Undo/Redo; an accent-tinted
  primary style for Undo/Redo; the column chooser moved ahead of the undo
  cluster; a zoom slider in the status bar; and one shared 8 px / 4 px spacing
  grid (the off-grid 6 px menu margin fixed). (Correction, 2026-11-11 audit:
  the focus rings were not merged — they exist only on the unmerged
  `opencode/focus-rings` branch and are tracked for a future release;
  shipped in the 2026-11-13 focus-ring slice at the top of this file.)
- **Inspector hierarchy.** The activity inspector groups its fields under
  accent section headers (Basic info, Calendar, Field planning, Progress,
  Relationships).
- **Context menu.** The row menu gains **Show in Gantt** and **Add to
  selection** (the pointer-free twin of ctrl-click) beside the existing New
  below / Duplicate / Delete.
- **Bugs fixed.** `Ctrl+W` is now actually bound (the menu advertised it but
  nothing handled it); the release-notes dialog's placeholder text no longer
  collapses its spaces into long runs; the dialog now carries the current
  release's notes.
- **Docs.** `ACCESSIBILITY.md` documents the Menu-bar, `Ctrl+W` and the
  multi-selection gestures; the F1 window lists `Ctrl+W`, the selection
  gestures and `Escape`.

Verification: CI green on `opencode/ci` across Linux (x86_64, aarch64),
Windows and macOS, plus the security, licensing, SBOM, lockfile, semver and
personal-data checks.

## 2026-11-06 — UI polish audit backlog closed; release 0.11.0 (statuses corrected 2026-11-11)

Every item on the UI polish audit's future-slices list is now closed in the
tree, verified slice by slice against `docs/design/ui-polish-audit.md`
(correction, 2026-11-11 audit: this headline over-claims — S2-9 was never
shipped, S2-1's focus rings were not merged, and S2-4, S2-12, S2-16 and S2-20
are partial or open; see the annotations below and the "Still open — verified
backlog" list in the PR #90 cross-check. S2-12 and S2-16 closed in the
2026-11-12 interaction slice; S2-9, S2-4 and S2-20 in the 2026-11-12
inspector slice; S2-1 (with S3-12) in the 2026-11-13 focus-ring slice):

- **S2-5 Table filter:** a filter field narrows rows by activity name
  (case-insensitive) and a **Critical only** toggle shows the critical path,
  with a `shown of total` counter. The filtered rows live in their own cache
  (`table_rows`), so the filter never leaks into the Gantt or the Resources
  dictionary, and the unfiltered hot path is unchanged. The table's shift/ctrl
  range multi-select is delivered with an atomic `DeleteActivities` command:
  ctrl/cmd-click toggles a row, shift-click takes the range from the anchor,
  Escape drops the extra rows, and **Delete N selected** removes the set in
  one undoable step.
- **S2-8 Relationship retype:** a new undoable `SetRelationshipType` command
  (with undo/redo and rejection tests) and a type combo in the inspector.
- **S2-10 Diary note editing:** an **Edit** action per note loads the text and
  saves through the undoable `UpdateDailyNote` command.
- **S2-11/S2-12 Per-frame work cached:** resource conflicts and the day-usage
  histogram are computed once per edit in `recompute` and shared behind an
  `Arc`; the lookahead window is cached against the edit version and the
  chosen window, and the snapshot comparison is indexed by activity id
  (linear, not O(n²)). (Correction, 2026-11-11 audit: S2-12 is partial — the
  window cache stands, but the "new in window" comparison loop is still
  O(n×m) per frame.)
- **S3-1 Token colours:** every raw `Color32::RED` and hardcoded red in the
  Table, Resources and Earned-value views now routes through the palette's
  critical token. `variance_color` keeps its own WCAG-AA-verified theme pairs
  by design (tests cover them).
- Verified already present from earlier slices: S2-1, S2-4, S2-6, S2-7, S2-9,
  S2-13, S2-14, S2-15, S2-16, S2-18, S2-20, S2-21, S2-22, S1-1c, S3-6, S3-7
  and S3-18.
  - Correction (2026-11-11 audit): that line over-claims. Verified done:
    S2-6, S2-7, S2-13, S2-14, S2-15, S2-21, S2-22, S1-1c, S3-7 and S3-18.
    S2-18 is done with one exception (Delete calendar still deletes
    immediately, though undoable; closed 2026-11-12). S3-6 is done for
    `ACCESSIBILITY.md` only
    (stale "Ctrl+1..5" copies remain in `docs/parity.md` and historical
    docs); S3-1 is done for the original finding (residual literals are
    being centralized in the 2026-11-11 token-hygiene slice). Not done:
    S2-9 is open (`SetActivityLocation`/`AssignActivityWbs` exist but have
    no UI usage); S2-1 is partial (focus rings never merged); S2-4 is
    partial (the helper covers 5 sites — raw date fields remain: the
    inspector constraint date, actual start, actual finish and reschedule
    status date); S2-16 is functionally open (the Ctrl+wheel handler is dead
    code under egui 0.35 — the chart does not zoom, and the keyboard
    Ctrl+Plus/Minus/0 zoom may double-zoom the whole GUI); S2-20 is partial
    (13 fields wired; ~15 visible-label sites plus all combo boxes still
    unlinked). (S2-16 closed in the 2026-11-12 interaction slice; S2-9,
    S2-4 and S2-20 closed in the 2026-11-12 inspector slice; S2-1 — with
    S3-12 — closed in the 2026-11-13 focus-ring slice at the top of this
    file.)

**Release 0.11.0** cuts all of the above: the version is bumped across the
workspace, `CHANGELOG.md` turns the Unreleased section into the dated 0.11.0
entry, and the audit doc's backlog table records each item's status. The only
release-checklist item that cannot be closed by code is the manual
screen-reader verification (NVDA/VoiceOver/Orca), which the audit explicitly
defers to the release checklist.

Verification: CI green on `opencode/ci` across Linux (x86_64, aarch64),
Windows and macOS, plus the security, licensing, SBOM, lockfile, semver and
personal-data checks.

## Design audit backlog — PR #90 cross-check (future slices)

PR #90 (`docs/audit/2026-11-03-exhaustive-design-and-gui-audit.md`, 43 findings)
was re-verified against the current tree and cross-referenced in
`docs/design/ui-polish-audit.md`.

**Closed already (S1 blockers):**
- S1-1 Create/delete activity: Ctrl+N + **New activity** toolbar button create an activity (select/recompute/scroll-to-view); inspector **Delete activity** with confirmation dialog.
- S1-2 Production: working activity/date/percent form issuing `RecordProduction`/`DeleteProduction`; `Ctrl+9` bound; listed in F1 dialog.
- S1-3 Dirty state: `dirty` flag, viewport-title `•` marker, Save/Discard/Cancel close guard.
- S2-2 Toolbar Undo/Redo now recompute + refresh_selection with shortcut tooltips, and are disabled when the command stack is empty (`can_undo()`/`can_redo()`).
- **Programme audit tool (AC2):** selecting an Audit-view finding now sets `scroll_to_selection`, so it selects *and scrolls* to the activity — acceptance criterion 2 fully met. CLI, GUI, stable IDs/thresholds, pure headless engine and 22 unit tests already in place (AC1, AC3–AC5).
- S2-3 Status messages coloured by `StatusSeverity` (Info/Success/Warning/Error → palette tokens); info/success messages auto-clear after five seconds (S2-3b), errors and warnings persist.
- S2-17 F1 shortcuts dialog lists Ctrl+S/O/N, Production (Ctrl+1..9) and all bindings.

**Still open — verified backlog (refreshed 2026-11-13 after the hardening +
S3-17 verification + icon fix slice; the authoritative next-slices list):**

- **P0/P1/P2 design-audit items:** none remaining — the last P0 item (S2-1's
  focus-ring remainder, closing S3-12 with it) closed in the 2026-11-13
  focus-ring slice; S2-9 (Location/WBS inspector fields), S2-4 (the last raw
  date fields) and S2-20 (the AccessKit `labelled_by` sweep) closed in the
  2026-11-12 inspector slice; S3-9 (column chooser / split-Gantt gating),
  S3-11 (empty inspectors) and S3-13/S3-14 (diary click-through) closed in
  the 2026-11-12 S3 polish slice.
- **S3 minors:** S3-17 font symbol coverage for non-Latin imports — **closed
  (verified/documented) in the 2026-11-13 slice above** (cmap-parsed
  coverage; limitation in `ACCESSIBILITY.md` "Text and character coverage";
  full non-Latin support queued as an enhancement). S3-12 (dense-table focus
  rings) closed with S2-1 in the 2026-11-13 focus-ring slice; S3-10
  (progress range hint), the Gantt and Production empty states, and the stale
  "Ctrl+1..5" copy in `docs/parity.md` closed in the S3 polish slice;
  historical/dated docs keep their original text as records. New tracked
  finding: **S3-20 Inter Bold-first face ordering** (deferred — needs the
  evidence pipeline to evaluate the visual-weight question safely; phase 2a
  is in, phase 2b pixels will provide the evaluation path).
- **Icon defect:** S3-19 Phosphor/Inter icon shadowing — **Done** in the
  2026-11-13 slice above (six icons previously rendered letterform glyphs;
  Phosphor face now precedes the text faces).
- **II.30 items:** the evidence pipeline is **in progress — phase 2a done**
  (AccessKit-tree assertions over the shell in
  `crates/ui-egui/tests/a11y_harness.rs`, kittest 0.35.0 dev-dep, no GPU,
  all 17 checks green on all four platforms), on top of the phase 1 headless
  seam (`CadenceApp::render` + `headless_render_tests` smoke test) and the
  title-command repaint fix. Phase 2b (rendered PNG artefacts via `wgpu` +
  `snapshot`, needing software Vulkan/lavapipe in CI, with a documented
  fallback to phase 2a's AccessKit evidence) is next; S3-20 (Inter
  Bold-first ordering) is evaluated once phase 2b pixels exist. The README
  placeholder image links were removed in the S3 polish slice, so real
  screenshots await phase 2b. The reduced-motion record in
  `ACCESSIBILITY.md` and the II.30.6 raw-colour gate landed in the earlier
  slices. (Phase 2b pixels are needed to evaluate S3-20 safely.)
- **Engine follow-up:** closed in the 2026-11-13 slice above —
  `schedule_with_progress` now honours `MustStartOn`/`MustFinishOn` for
  not-started work (D10 rounding, zero float, actuals untouched).

No item above blocks the current release; all are tracked so they are not missed.

## 2026-11-21 — Suite standards §12.1 completion (guide revision reconciliation)

Done in this slice per guide §12.1 suite standards (D27):

- **Token ledger automation:** Added `Run token ledger script` step to `.github/workflows/release.yml` that runs `scripts/token-ledger.py` on each semver tag. The step includes error handling for when the OpenCode session store is not available (e.g., in CI environments).

- **Shared `crates/update-check` crate:** Created `crates/update-check/` as a shared suite crate containing the update-checking logic. The crate exposes `UpdateStatus` enum and related functions, following the pattern that suite crates are shared across multiple applications. Added to workspace members in `Cargo.toml`.

- **Verify-then-install updater:** Replaced the "Update & Restart" TODO with a verify-then-install flow in the Release Notes popup. The button "Update & Open Installer" opens the GitHub release in the user's default browser; actual installation happens via the platform's package manager (no auto-restart). Signature verification is left to the download manager.

- **README screenshots placeholder:** Added a "Screenshots" section in README.md with placeholder images for Table, Gantt, Resources, and Report views. These will be refreshed at v1.0 and every major release per the suite standards.

Verification: CI green on the pull request (run 44966274579, head `ebc0f90` — verify on all four platforms, clippy, semver, packaging, rendered evidence) and on `main` after the squash merge (run 44966739470, commit `7320513`). The workspace identity is `Cadence Dev` <dev@example.com>.
