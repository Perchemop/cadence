# RP-2 — Assistant settings storage (off-by-default persistence, key session-only)

**Index:** RP-1 → RP-2 (this) → RP-3 → RP-4 → RP-5 → RP-6. Spec:
[`docs/specs/review-panel.spec.md`](../specs/review-panel.spec.md).

## Objective

Give the assistant a place for its settings to survive a restart, honouring
the codebase's own secret invariant: `SecretKey` serialises as
`"<redacted>"`, so a settings file **cannot** round-trip a key — and this
module must not work around that. **No UI code**; the settings file is
loaded/stored by RP-3's shell wiring.

Verified fact driving the shape: no settings storage exists anywhere in the
repository today (the persistence crate is the project-file layer only, and
the research note forbids the key in the project file). The settings module
therefore lives in `cadence-assistant`.

## Target file paths

- `crates/assistant/src/settings.rs` (new — path, load, store)
- `crates/assistant/src/lib.rs` (module declaration + `pub use` only)

## Blocking dependencies

None (independent of RP-1; both must land before RP-3).

## Contract (from the spec — implement exactly this)

- `pub fn settings_path() -> Option<PathBuf>` — the user-level settings file
  (platform config directory: `XDG_CONFIG_HOME` then `HOME` on Linux,
  `APPDATA` on Windows, `HOME/Library/Application Support` on macOS), or
  `None` when the environment provides no directory. Hand-rolled platform
  lookup; **no new dependency**.
- `pub fn load_settings() -> AssistantConfig` — read + parse; any missing
  file, read failure or parse failure yields `AssistantConfig::default()`
  (off). Never panics, never fails the caller.
- `pub fn store_settings(config: &AssistantConfig) -> Result<(), String>` —
  writes the file (creating the directory if needed) with the key removed
  (`api_key: None`) — the key is session-only by design. Returns a
  plain-language error string on failure for the status bar.

## Verifiable acceptance criteria (CI-checkable)

- [ ] Round-trip test: store a config with enabled/provider/model/base_url
      set and a key present; load it back; assert the non-key fields
      round-trip **and** `api_key` is `None` (the key never survives the
      file).
- [ ] Legacy test: a pre-existing file containing `"<redacted>"` (the shape
      the foundation slice's serde test fixes) still loads, and the loaded
      config is not configured.
- [ ] Corrupt-file test: a file with invalid JSON loads as the default (off)
      config instead of failing.
- [ ] Path tests run with injected environment values (no reliance on the
      test machine's real home directory; no test writes outside the test's
      own temporary directory).
- [ ] All tests headless and green in CI (`Verify` on all four platforms);
      `Static analysis (rust)` clean; no new dependency (lockfile unchanged
      apart from the workspace itself).
- [ ] No file outside `crates/assistant/**` and this ticket's tracking
      entries is modified.

## Out of scope

Any UI to edit the settings (RP-3's enabling form consumes this module); OS
keystore integration (spec Q4 option (b), rejected for this slice); storing
the key in the file (spec Q4 option (c), rejected as a design violation).
