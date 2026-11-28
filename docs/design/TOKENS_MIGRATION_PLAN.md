# CADENCE-DESIGN-TOKENS Migration Plan

**Epic:** CADENCE-DESIGN-TOKENS — Design Token System Implementation
**Status:** Planned
**Target:** Replace `crates/ui-egui/src/design.rs` with `design_tokens.rs` + `theme.rs`

---

## 1. Current State Analysis

### `crates/ui-egui/src/design.rs` (Current)
- **Palette struct**: 14 colour fields for one theme (light/dark via `palette(dark)`)
- **Constants**: `ACCENT`, `FOCUS_RING`, `ACCENT_ON` (module-level)
- **Chrome module**: `primary`, `ghost`, `tab` button helpers
- **Functions**: `install_fonts`, `install_style`, `apply`, `apply_visuals`, `variance_color`, `translucent`
- **Typography**: Hardcoded in `install_style` (Body 11px, Small 9.5px, Heading 16px, Monospace 11px)
- **Spacing**: Hardcoded in `install_style` (item_spacing 8×4, button_padding 8×4, indent 16)
- **Radius**: Hardcoded (4px widgets, 6px windows)
- **Elevation**: Implicit via egui defaults
- **Motion**: Near-zero (only egui native ~100ms hover fades)
- **Density**: Not implemented

### Gaps vs. Discovery Requirements
| Requirement | Current | Required |
|-------------|---------|----------|
| Semantic color roles | Partial (Palette) | Complete (surface, text, accent, border, focus, status, overlay) |
| Typography scale | 4 sizes | 9 roles (display, h1-h3, body, small, caption, mono, button) |
| Spacing system | 3 values | 9 tokens + component spacing |
| Radius system | 2 values | 6 values (none, sm, md, lg, xl, full) |
| Elevation | None | 4 levels |
| Motion specs | None | Duration, easing, reduced-motion |
| Density levels | None | 3 levels (compact/standard/comfortable) |
| Serialization | No | Yes (serde) |
| Headless testable | Partial | Full |

---

## 2. New Architecture

### Files Created
```
crates/ui-egui/src/
├── design_tokens.rs   # Pure token types (serde, headless-testable)
├── theme.rs           # egui integration (Visuals + Style mapping)
└── design.rs          # DEPRECATED — to be removed after migration
```

### Module Structure
- **design_tokens.rs**: `DesignTokens`, `ColorSystem`, `TypographyScale`, `SpacingSystem`, `RadiusSystem`, `ElevationSystem`, `MotionSystem`, `DensitySystem`, `ThemeTokens`, `ThemeVariant`, `DensityLevel`
- **theme.rs**: `apply_theme`, `apply_visuals_only`, `install_fonts`, `chrome` module, `StatusSeverity`, `variance_color`, color parsing utilities

---

## 3. Migration Steps

### Phase 1: Validation (Current)
- [x] Create `design_tokens.rs` with complete token types
- [x] Create `theme.rs` with egui integration
- [x] Create `docs/design/tokens.json` schema
- [x] Unit tests in `design_tokens.rs` (serialization, spacing, density, radius, typography, motion, elevation, status colors)
- [x] Unit tests in `theme.rs` (color parsing, font_id, variance, status severity, chrome buttons)

### Phase 2: Integration Testing
- [ ] Run `cargo test -p cadence-ui-egui` — all tests pass
- [ ] Run accessibility tests (`cargo test -p cadence-ui-egui a11y`) — AccessKit tree unchanged
- [ ] Run headless render tests — visual parity with current theme
- [ ] Verify both light and dark themes render correctly

### Phase 3: Wire into Application
- [ ] Update `crates/ui-egui/src/lib.rs`:
  - Replace `design::apply(ctx, dark)` with `theme::apply_theme(ctx, variant)`
  - Replace `design::install_fonts(ctx)` with `theme::install_fonts(ctx)`
  - Replace `design::palette(dark)` calls with `DesignTokens::create(variant)` + token access
  - Replace `design::variance_color` with `theme::variance_color`
  - Replace `design::chrome::*` with `theme::chrome::*`
  - Replace `design::ACCENT` with token access
- [ ] Update `CadenceApp::design_tokens()` to use new system
- [ ] Update `StatusSeverity::color` to use new tokens

### Phase 4: Cleanup
- [ ] Remove `crates/ui-egui/src/design.rs`
- [ ] Remove `pub mod design;` from `lib.rs`
- [ ] Update `docs/design/gui-stack.md` with new module paths
- [ ] Update `CHANGELOG.md`

### Phase 5: Evidence & Documentation
- [ ] Generate rendered evidence (light/dark, all views, empty/error states)
- [ ] Update `ACCESSIBILITY.md` with new contrast test coverage
- [ ] Record in `DECISIONS.md`

---

## 4. Breaking Changes & Compatibility

### API Changes
| Old API | New API | Notes |
|---------|---------|-------|
| `design::apply(ctx, dark)` | `theme::apply_theme(ctx, ThemeVariant)` | Enum instead of bool |
| `design::apply_visuals(ctx, dark)` | `theme::apply_visuals_only(ctx, ThemeVariant)` | Renamed for clarity |
| `design::palette(dark)` | `DesignTokens::create(ThemeVariant)` | Returns full token set |
| `design::variance_color(val, dark)` | `theme::variance_color(val, &tokens)` | Takes token reference |
| `design::chrome::primary(label)` | `theme::chrome::primary(&tokens, label)` | Takes token reference |
| `design::chrome::ghost(label)` | `theme::chrome::ghost(&tokens, label)` | Takes token reference |
| `design::chrome::tab(dark, selected, label)` | `theme::chrome::tab(&tokens, selected, label)` | No dark param needed |
| `design::ACCENT` | `tokens.color.accent.primary` | Access via tokens |
| `design::FOCUS_RING` | `tokens.color.focus.ring` | Access via tokens |
| `design::ACCENT_ON` | `tokens.color.text.on_accent` | Access via tokens |

### Backwards Compatibility
- No backwards compatibility layer — clean break
- All call sites updated in Phase 3
- `ThemeVariant` enum replaces `bool` for theme selection

---

## 5. Test Coverage

### Existing Tests to Preserve
- `crates/ui-egui/tests/a11y_harness.rs` — AccessKit tree assertions
- Headless render tests in `lib.rs` — shell rendering

### New Tests Added
- `design_tokens.rs` module tests: serialization, spacing tokens, density scaling, radius values, typography scale, motion reduced overrides, theme variants, elevation levels, status color distinctness
- `theme.rs` module tests: color parsing (hex, rgb, rgba), font_id weights, variance colors, status severity colors, status severity classification, chrome buttons

### CI Gates
- `cargo test --workspace` — all tests pass
- `cargo clippy -- -D warnings` — no warnings
- `cargo fmt --check` — formatted
- Accessibility tests pass on all 4 CI platforms

---

## 6. Rollback Plan

If integration fails:
1. `git revert` the migration commit
2. `design.rs` remains intact
3. No schema changes to persist

---

## 7. Timeline

| Phase | Estimate | Dependencies |
|-------|----------|--------------|
| 1. Validation | 1 day | None |
| 2. Integration Testing | 1 day | Phase 1 |
| 3. Wire into Application | 1 day | Phase 2 |
| 4. Cleanup | 0.5 day | Phase 3 |
| 5. Evidence & Docs | 0.5 day | Phase 4 |
| **Total** | **4 days** | |

---

## 8. Success Criteria

- [ ] All existing tests pass
- [ ] New token tests pass
- [ ] Visual parity with current themes (light/dark)
- [ ] Accessibility tests pass (AccessKit tree unchanged)
- [ ] No clippy warnings
- [ ] `docs/design/tokens.json` validates against schema
- [ ] `docs/design/gui-stack.md` updated
- [ ] `CHANGELOG.md` entry added
- [ ] `DECISIONS.md` entry added

---

## 9. Related Documents

- `docs/design/brand.md` — Design vision (source of truth)
- `docs/design/tokens.json` — Token schema (machine-readable)
- `docs/design/gui-stack.md` — GUI stack record
- `docs/discovery/inspiration-apps-ui-ux-discovery.md` — Discovery research
- `guide.md` II.30 — Rust GUI/UX rules

---

## 10. Decision Log

This migration implements **Decision D33** (to be recorded):
> Adopt structured design token system (design_tokens.rs + theme.rs) replacing ad-hoc design.rs. Tokens are serde-serializable, headless-testable, and cover all II.30 categories: color, typography, spacing, radius, elevation, motion, density.