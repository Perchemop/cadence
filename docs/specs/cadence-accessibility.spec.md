# CADENCE-ACCESSIBILITY Feature Specification

## Overview

**Epic**: CADENCE-ACCESSIBILITY — WCAG 2.2 AA Compliance
**User Value**: Ensure Cadence is usable by people with disabilities (visual, motor, cognitive) — professional construction planning must be accessible
**Target Users**: Users with visual impairments (screen reader, low vision, color blindness), motor impairments (keyboard-only, switch control), cognitive impairments
**Status**: Draft for review
**Standard**: WCAG 2.2 Level AA (guide II.21)

---

## Functional Requirements (EARS Format)

### FR-01: Keyboard Navigation
- **FR-01.1**: The system shall provide complete keyboard operability for all functionality (WCAG 2.1.1).
- **FR-01.2**: The system shall provide a global keyboard shortcut map accessible via F1.
- **FR-01.3**: The system shall provide view-specific keyboard navigation:
  - **Table**: Arrow keys (navigation), Enter (edit), Space (select), Ctrl+A (select all), Delete (delete), Ctrl+C/V/X (copy/paste/cut).
  - **Gantt**: Arrow keys (pan), +/- (zoom), Home/End (jump), Shift+drag (link), Space (select).
  - **Resources**: Arrow keys, Enter (edit assignment), Delete (remove).
  - **Inspector**: Tab (field navigation), Arrow keys (dropdown), Enter (confirm), Escape (cancel).
  - **Dialogs**: Tab (focus order), Escape (close), Enter (default action).
  - **Command Palette (Ctrl+K)**: Fuzzy search, Enter (execute), Escape (close).
- **FR-01.4**: The system shall provide density cycling shortcuts: Alt+1/2/3 for Compact/Standard/Comfortable, Alt+Shift+D for cycle.
- **FR-01.5**: All keyboard shortcuts shall be documented in F1 help and F1 shall be globally accessible.

### FR-02: Focus Management
- **FR-02.1**: The system shall provide a visible focus ring on all focusable elements with ≥3:1 contrast ratio against adjacent colors (WCAG 2.4.7).
- **FR-02.2**: The system shall trap focus within modal dialogs and restore focus to trigger element on close (WCAG 2.4.3).
- **FR-02.3**: The system shall restore focus on: dialog close, view switch, activity deletion, command palette close, toolbar customize close.
- **FR-02.4**: Focus order shall follow logical reading order (top-to-bottom, left-to-right).
- **FR-02.5**: Skip links shall be provided for main content areas.

### FR-03: Screen Reader Support (AccessKit)
- **FR-03.1**: The system shall expose AccessKit landmark regions: MenuBar, TabList, ToolBar, Region (Table/Gantt/Resources), Complementary (Inspector), StatusBar.
- **FR-03.2**: The system shall expose view-specific semantic roles:
  - Table → Grid with column/row headers
  - Gantt → Tree with TreeItem per bar
  - Resources → Table with sortable columns
  - Baselines/Calendars → List + Table
  - Audit/Lookahead/Diary/Production → List/Table as appropriate
- **FR-03.3**: The system shall expose live regions: status (polite), errors (assertive), filter results (polite), density changes (polite).
- **FR-03.4**: The system shall expose heading hierarchy: h1 (window), h2 (view titles), h3 (panel/section headers).
- **FR-03.5**: The system shall expose non-empty accessible names for all interactive elements via `labelled_by` or intrinsic text.

### FR-04: Color Contrast (WCAG 2.2 AA)
- **FR-04.1**: The system shall meet 4.5:1 contrast ratio for body text on all backgrounds (Light, Dark, HighContrastLight, HighContrastDark).
- **FR-04.2**: The system shall meet 3:1 contrast ratio for large text (≥18pt or ≥14pt bold), UI boundaries, focus rings, and interactive elements.
- **FR-04.3**: The system shall validate all 22 semantic color role pairs across 4 themes (Light, Dark, HighContrastLight, HighContrastDark).

### FR-05: High Contrast Mode
- **FR-05.1**: The system shall provide dedicated High Contrast Light and High Contrast Dark palette variants.
- **FR-05.2**: The system shall detect OS high contrast preference (Windows HCM, macOS Increase Contrast, Linux forced colors).
- **FR-05.3**: The system shall meet WCAG AAA contrast (7:1 normal, 4.5:1 large) in high contrast mode where feasible.
- **FR-05.4**: The system shall not use color as the sole indicator of state (WCAG 1.4.1).

### FR-06: Reduced Motion
- **FR-06.1**: The system shall respect OS `prefers-reduced-motion` preference.
- **FR-06.2**: The system shall provide in-app reduced motion toggle (GlobalSettings.reduced_motion).
- **FR-06.3**: When reduced motion is enabled, all animations/transitions shall complete in ≤50ms or be disabled.
- **FR-06.3**: The system shall provide opt-out pattern for future animations (WCAG 2.3.3).

### FR-07: Toolbar Density Accessibility
- **FR-07.1**: The system shall provide 3 density modes: Comfortable (default), Compact, Condensed.
- **FR-07.2**: The system shall provide keyboard density cycling: Alt+1/2/3 for modes, Alt+Shift+D for cycle.
- **FR-07.3**: The system shall announce density changes via AccessKit live region (polite).
- **FR-07.4**: All toolbar targets shall meet minimum 24pt (Compact), 28pt (Standard), 32pt (Comfortable).
- **FR-07.5**: Primary actions shall meet 32pt minimum in all densities.

### FR-08: CI Gates
- **FR-08.1**: The system shall add 5 blocking CI gates to `.ci-required.txt`:
  - `a11y-contrast-tests`: 22+ semantic color pair contrast tests
  - `a11y-harness-tests`: AccessKit tree assertions (landmarks, roles, live regions, names)
  - `a11y-target-size-tests`: Minimum 24pt all targets, 32pt primary
  - `a11y-focus-ring-tests`: Focus ring visibility, trapping, restoration
  - `a11y-density-tests`: Density cycling, announcement, persistence
- **FR-08.2**: All gates shall be blocking (listed in `.ci-required.txt`).
- **FR-08.3**: All gates shall run on `ubuntu-latest` with stable Rust toolchain.

---

## Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | Contrast test execution time | < 5s |
| NFR-02 | AccessKit harness execution time | < 30s |
| NFR-03 | Focus ring render time | < 1ms per element |
| NFR-04 | Live region announcement latency | < 100ms |
| NFR-05 | Density change announcement latency | < 100ms |
| NFR-06 | High contrast mode activation | < 100ms |
| NFR-06 | Reduced motion toggle | < 50ms |

---

## Acceptance Criteria (Given/When/Then)

### AC-01: Contrast Compliance
**Given** the application is running in Light theme
**When** the contrast validation runs
**Then** all 22 semantic color pairs meet ≥4.5:1 (text) or ≥3:1 (UI) ratios
**And** the same holds for Dark, HighContrastLight, HighContrastDark themes

### AC-02: Keyboard Navigation
**Given** the user navigates using only keyboard
**When** the user performs any core workflow (create activity, link, schedule, report)
**Then** every step is completable via keyboard alone
**And** no step requires pointer interaction

### AC-03: Focus Management
**Given** a modal dialog opens
**When** the dialog opens
**Then** focus moves to the first focusable element
**And** Tab cycles within the dialog
**And** Escape closes the dialog and restores focus to trigger

### AC-04: Screen Reader Support
**Given** a screen reader (NVDA/VoiceOver/Narrator) is active
**When** the user navigates the main window
**Then** the screen reader announces: window title, landmark regions, view tabs, focused element role/name/value
**And** live regions announce status changes without focus loss

### AC-05: High Contrast Mode
**Given** Windows High Contrast Mode is enabled
**When** the application starts
**Then** the High Contrast palette is applied
**And** all contrast ratios meet WCAG AAA (7:1/4.5:1)
**And** no information is conveyed by color alone

### AC-06: Reduced Motion
**Given** OS reduced motion is enabled
**When** the application performs any animated transition
**Then** the transition completes in ≤50ms or is disabled
**And** no layout shifts occur due to animation

### AC-07: Toolbar Density Accessibility
**Given** the user cycles toolbar density with Alt+Shift+D
**When** density changes
**Then** AccessKit announces "Toolbar density: Compact/Standard/Comfortable" (polite)
**And** all toolbar targets meet minimum size for the density

### AC-08: CI Gates Pass
**Given** a pull request is opened
**When** CI runs
**Then** all 5 accessibility gates pass
**And** merge is blocked if any gate fails

---

## Error Handling

| Error Condition | Detection | Response |
|----------------|-----------|----------|
| Contrast test failure | CI test failure | Block merge, show failing pairs |
| AccessKit tree missing role | Harness test failure | Block merge, show missing element |
| Focus trap failure | Harness test failure | Block merge, show dialog context |
| Target size violation | Test failure | Block merge, show element & size |
| Live region missing | Test failure | Block merge, show missing region |

---

## Implementation TODO Checklist

### Phase 1: Contrast Tokens + CI Gates (Week 1)
- [ ] Add contrast validation to `design_tokens.rs` (WCAG 2.2 AA)
- [ ] Add `contrast_ratio()`, `relative_luminance()`, `meets_wcag()` functions
- [ ] Define 22 semantic color pair assertions (4 themes × roles)
- [ ] Add `crates/ui-egui/tests/contrast_tests.rs` with headless tests
- [ ] Add 5 CI gates to `.ci-required.txt`
- [ ] Add 5 CI jobs to `.github/workflows/ci.yml`
- [ ] Run CI verification

### Phase 2: Keyboard + Focus (Week 2)
- [ ] Document complete keyboard shortcut map (F1, Ctrl+K, Ctrl+1-9, Alt+1-3, etc.)
- [ ] Implement focus ring with ≥3:1 contrast (ACCENT color)
- [ ] Implement focus trapping in all dialogs
- [ ] Implement focus restoration rules for all triggers
- [ ] Add skip links for main content areas
- [ ] Add unit tests for focus ring, trap, restoration

### Phase 3: Screen Reader + AccessKit (Week 3)
- [ ] Add landmark regions to all views (MenuBar, TabList, ToolBar, Region, Complementary, StatusBar)
- [ ] Implement view-specific semantics (Table→Grid, Gantt→Tree, etc.)
- [ ] Add live regions: status (polite), errors (assertive), filter (polite), density (polite)
- [ ] Implement heading hierarchy (h1/h2/h3)
- [ ] Add AccessKit labels to all interactive elements
- [ ] Add `a11y_harness.rs` assertions for landmarks, roles, live regions

### Phase 4: High Contrast + Reduced Motion (Week 3-4)
- [ ] Add HighContrastLight/HighContrastDark palette variants
- [ ] Implement OS high contrast detection
- [ ] Add WCAG AAA contrast targets for HC mode
- [ ] Implement reduced-motion detection (OS + in-app toggle)
- [ ] Add reduced-motion opt-out for animations
- [ ] Verify no color-only state indicators

### Phase 5: Density Accessibility (Week 4)
- [ ] Implement 3 density modes (Comfortable/Compact/Condensed)
- [ ] Add keyboard density cycling (Alt+1/2/3, Alt+Shift+D)
- [ ] Add AccessKit live region announcement for density changes
- [ ] Verify minimum target sizes (24pt/28pt/32pt)
- [ ] Add density tests to a11y harness

### Phase 5: CI Gates + Polish (Week 5)
- [ ] Add 5 CI gates to `.ci-required.txt`
- [ ] Add 5 CI jobs to `.github/workflows/ci.yml`
- [ ] Verify all 5 gates block merge on failure
- [ ] Run full accessibility audit with NVDA/VoiceOver/Narrator
- [ ] Document keyboard shortcuts in F1 help

---

## Validation

Run CI verification:
```bash
ci.sh run "CADENCE-ACCESSIBILITY Phase 1-5"
```

Expected: All 5 accessibility gates pass, all tests pass, clippy clean, formatting clean.