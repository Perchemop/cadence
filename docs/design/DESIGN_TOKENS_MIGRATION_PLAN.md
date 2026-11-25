# Design Tokens Migration Plan

**Epic:** CADENCE-DESIGN-TOKENS
**Status:** Ready for Implementation
**Target:** Replace `crates/ui-egui/src/design.rs` legacy implementation with comprehensive token system in `design_tokens.rs`

---

## 1. Current State Analysis

### Existing Implementation (`design.rs`)
- **Palette struct**: 13 colour fields (bg, panel, text, text_muted, grid, critical, normal, ok, warn, deep, non_working, status_date, focus_ring)
- **Constants**: ACCENT, FOCUS_RING, ACCENT_ON
- **Functions**: `palette(dark)`, `variance_color()`, `translucent()`, `install_fonts()`, `install_style()`, `apply()`, `apply_visuals()`
- **Chrome helpers**: `chrome::primary()`, `chrome::ghost()`, `chrome::tab()`
- **Typography**: Hard-coded sizes (11, 9.5, 16) in `install_style()`
- **Spacing**: Hard-coded (8×4 item_spacing, 8×4 button_padding, 16 indent)
- **Radius**: Hard-coded (4 widget, 6 window/menu)
- **No**: Density tokens, elevation system, motion specs, reduced-motion support, Theme trait

### Gaps vs. Specification (guide II.30 / brand.md / discovery)
| Feature | Current | Required | Gap |
|---------|---------|----------|-----|
| Colour roles | 13 | 24+ | Missing: surface_sunken, text_secondary, text_on_accent, accent_hover/active, status_*_on, border_strong, selection_*, overlay_scrim, disabled_* |
| Typography scale | 3 sizes | 9 roles | Missing: display, h1, h3, caption, mono, button with weight/line-height |
| Spacing system | 3 values | 9 tokens + density | Missing: xs, xxl, xxxl, huge, massive; density multipliers |
| Radius | 2 values | 6 tokens | Missing: none, sm, xl, full |
| Elevation | None | 4 levels | Complete gap |
| Motion | None | 3 durations + easing | Complete gap |
| Reduced motion | Respects OS | Explicit token | Partial |
| Density | None | 3 levels | Complete gap |
| Theme abstraction | Functions | Trait + registry | Complete gap |

---

## 2. Migration Strategy

### Phase 1: New Token System (✅ Done)
- Created `design_tokens.rs` with complete token definitions
- Implemented `Theme` trait with `LightTheme`/`DarkTheme`
- Added `ThemeRegistry` for mode management
- Maintained 100% backward compatibility via re-exports in `design.rs`

### Phase 2: Validation Tests (✅ Done)
- Created `tests/design_token_tests.rs` with:
  - WCAG 2.2 AA contrast verification (4.5:1 text, 3:1 UI)
  - Token completeness checks
  - Density scaling verification
  - Serialization round-trip tests
  - Legacy API compatibility tests

### Phase 3: JSON Export (✅ Done)
- Created `docs/design/design-tokens.json` with full token spec
- Includes light/dark themes, all token categories, metadata

### Phase 4: Incremental Adoption (Next)
Update call sites to use new API where beneficial:

| Priority | Target | Change |
|----------|--------|--------|
| High | `CadenceApp::design_tokens()` | ✅ Already updated to use `ThemeRegistry` |
| High | New UI code | Use `design::Theme` / `design::theme_registry()` |
| Medium | `apply_theme()` calls | Can switch to `theme_registry().apply(ctx, dark)` |
| Low | `chrome::*` helpers | Keep for now; migrate when toolbar redesigned |
| Low | `variance_color()` | Keep; uses new palette internally |

### Phase 5: Legacy Removal (Future)
Once all internal code uses new API:
1. Remove `design.rs` compatibility layer
2. Rename `design_tokens.rs` → `design.rs`
3. Update imports across workspace
4. Remove `Palette` struct and legacy functions

---

## 3. Breaking Changes & Mitigations

### No Breaking Changes in Phase 1-3
- All existing public API preserved via `design.rs` re-exports
- `design::palette()`, `design::apply()`, `design::chrome::*` work identically
- `CadenceApp::design_tokens()` returns same `DesignTokens` struct

### Future Breaking Changes (Phase 5)
| Removed | Replacement |
|---------|-------------|
| `design::Palette` | `design::ColorTokens` |
| `design::palette(dark)` | `theme.colors()` |
| `design::apply(ctx, dark)` | `theme.apply(ctx)` |
| `design::ACCENT` | `theme.colors().accent` |
| `design::chrome::*` | New component API (TBD) |

**Mitigation**: Provide `design::compat` module with deprecated aliases for one release cycle.

---

## 4. Test Plan

### Automated (CI)
- `cargo test -p cadence_ui_egui design_token_tests` — all token validation
- `cargo test -p cadence_ui_egui a11y_harness` — AccessKit tree assertions
- `cargo test -p cadence_ui_egui headless_render` — render smoke tests

### Manual Verification
1. Run app in light theme → verify colours match `design-tokens.json`
2. Run app in dark theme → verify colours match `design-tokens.json`
3. Toggle density (when implemented) → verify spacing scales
4. Enable reduced motion (OS setting) → verify animations disabled
5. Keyboard navigation → verify focus ring visibility (3:1 contrast)

---

## 5. Rollback Plan

If issues discovered post-merge:
1. `git revert` the migration commits
2. `design.rs` compatibility layer ensures zero-downtime rollback
3. No database/schema changes involved

---

## 6. Dependencies

- **egui 0.36.2** — Current resolved version (gui-stack.md)
- **eframe 0.36.2** — For `ThemeMode::System` detection
- **serde** — For JSON export (already in workspace)
- **No new dependencies** — Pure Rust implementation

---

## 7. Definition of Done

- [x] `design_tokens.rs` created with complete token system
- [x] `design.rs` updated as compatibility layer
- [x] `design_token_tests.rs` with WCAG contrast verification
- [x] `design-tokens.json` exported
- [x] `CadenceApp::design_tokens()` uses new ThemeRegistry
- [ ] CI passes (format, clippy, test, build)
- [ ] Manual visual verification on light/dark themes
- [ ] Density tokens integrated into toolbar (follow-up slice)
- [ ] Reduced-motion integrated (follow-up slice)
- [ ] Legacy API deprecated with `#[deprecated]` (Phase 5)

---

## 8. Related Documents

- `docs/design/brand.md` — Design vision & colour spec
- `docs/design/gui-stack.md` — Toolkit version record
- `docs/discovery/inspiration-apps-ui-ux-discovery.md` — Research basis
- `guide.md` II.30 — Rust GUI/UX rules (tokens, contrast, evidence)
- `CHANGELOG.md` — Track migration progress