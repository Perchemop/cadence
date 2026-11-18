# RP-3 — Assistant view shell: tenth view tab, enabling state and disabled states

**Index:** RP-1 → RP-2 → RP-3 (this) → RP-4 → RP-5 → RP-6. Spec:
[`docs/specs/review-panel.spec.md`](../specs/review-panel.spec.md).

## Objective

Add the **Assistant** view to the shell as the tenth centre view (spec Q1
recommendation), with its three states: the **enabling state** shown when
`AssistantConfig::is_configured()` is false (explanation + configuration
form, satisfying research criterion 1), the **request state** shown when
configured (prompt field + submit button — the submit path itself is RP-4),
and the **waiting state** placeholder (request button disabled, "Waiting for
the provider…" — the thread arrives in RP-4). No provider call happens in
this ticket; no proposal is rendered yet.

## Target file paths

- `crates/ui-egui/src/lib.rs` (view-mode variant, tab strip entry, View-menu
  radio item, render method, `CadenceApp` assistant state fields, the two
  offline-support seams, settings load at construction)
- `crates/ui-egui/Cargo.toml` (re-add the `cadence-assistant` workspace
  dependency — it was removed under the dead-code rule when nothing consumed
  the crate; it returns exactly now)
- root `Cargo.toml` only if the workspace-dependency entry itself needs
  re-adding

## Blocking dependencies

- RP-1 (the view renders configured/unconfigured states only, but RP-3
  compiles against the assistant crate's public types).
- RP-2 (settings load at app construction).

## Contract (from the spec — implement exactly this)

- `ViewMode` gains the Assistant variant; the tab strip, the View-menu radio
  list and the view-switch match each gain the entry, following the existing
  patterns (icon glyph + space + label; radio item; shortcut arm — no global
  chord per spec Q2 recommendation (a)).
- `CadenceApp` gains: the loaded `AssistantConfig` (via
  `cadence_assistant::load_settings()` at construction), the request text
  buffer, the in-flight receiver field (typed, unused until RP-4), the
  proposal-under-review state (typed, unused until RP-4), and the
  current validation-error list (typed, unused until RP-4).
- Public offline-support seams, following the `set_update_checks_enabled`
  precedent, documented as offline test/harness support:
  `set_assistant_config(&mut self, AssistantConfig)` and
  `set_assistant_proposal(&mut self, Proposal)` (the latter stored for RP-4
  to render).
- Enabling state: short plain-language explanation (what the assistant is,
  that it is optional, that a request sends project data to the configured
  provider), then the configuration form — provider name, model, optional
  server address, password-masked API key field, enable checkbox — writing
  through `cadence_assistant::store_settings` on apply. When unconfigured,
  **no provider is constructed, no thread is spawned, no network path
  exists**.
- Request state (configured): the request text field (labelled), the
  "Request proposal" button (disabled in this ticket — RP-4 wires it), and
  the collapsed configuration section.
- Design tokens only (`design::palette` roles; no raw colour literals
  outside `design.rs` — the CI grep gate must pass).

## Verifiable acceptance criteria (CI-checkable)

- [ ] Headless render tests (the existing in-crate pattern driving
      `CadenceApp::render` with a bare `egui::Context`): the Assistant view
      renders in both themes and over two passes, unconfigured (enabling
      form) and configured (request state), without any network access.
- [ ] Unconfigured invariant test: with the default config, rendering the
      Assistant view constructs no provider and spawns no thread (assert via
      the typed receiver field remaining `None`).
- [ ] The ten view tabs are all reachable and labelled; the View menu lists
      the Assistant view as a radio item.
- [ ] `set_assistant_config` switches the view between the enabling and
      request states in a headless test.
- [ ] The a11y harness still passes unchanged (its nine-tab fixture is
      extended to ten in RP-5; this ticket must not break the existing
      assertions — every new input widget already carries a non-empty
      accessible name so the harness's hand-rolled named-input check passes
      with the new view present in either state).
- [ ] CI green: `Verify` on all four platforms, `Static analysis (rust)`,
      `Lockfile freshness`, `Secret scanning`, `Personal data scan`.

## Out of scope

The provider call and polling (RP-4); review rendering and apply (RP-4);
harness assertions for the new tab (RP-5); docs (RP-6).
