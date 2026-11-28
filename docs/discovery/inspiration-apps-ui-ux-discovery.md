# Discovery Document: Inspiration Apps UI/UX Deep Research

**Epic:** CADENCE-UI-RESEARCH - Deep Research into Inspiration Apps Settings, Toolbar Sizing, Resizing & UI/UX Patterns
**Discovery Goal:** Extract granular UI/UX patterns from inspiration applications to inform Cadence's settings architecture, toolbar design, resizing behaviors, and overall look/feel
**Success Criteria:** Comprehensive pattern library with granular feature/sub-feature breakdown, integration synergies, and implementation specifications for Cadence
**Timeline:** 2 weeks research + 1 week synthesis
**Stakeholders:** UI/UX Designer, Lead Developer, Product Owner, Accessibility Lead

---

## 1. Discovery Overview

This discovery investigates how leading construction/project management and professional desktop applications handle:
- Settings architecture (global, project, view, tool-specific)
- Toolbar sizing, density, and responsive resizing
- Sub-settings and progressive disclosure patterns
- Cross-feature integration and synergy
- Visual design language (color, typography, spacing, motion)
- Accessibility and keyboard-first behaviors

**Inspiration Applications:** Microsoft Project, Primavera P6, Smartsheet, Microsoft Excel, Linear, Figma, VS Code, Notion, Obsidian, Adobe Creative Cloud, Blender

---

## 2. Hypothesis Map

| Hypothesis ID | Statement | Confidence | Validation Method | Status |
|---------------|-----------|------------|-------------------|--------|
| H1 | Users need 3 density levels (compact/standard/comfortable) for toolbar sizing | High | Competitive audit + user interviews | Open |
| H2 | Settings should be hierarchical: Global → Project → View → Tool | High | Pattern analysis across 10+ apps | Open |
| H3 | Resizing should preserve ratios and snap to grid | Medium | Interaction testing + spec review | Open |
| H4 | Cross-view sync (settings propagate correctly) is a key differentiator | Medium | User workflow analysis | Open |
| H5 | Keyboard-first navigation with visible focus is expected in pro tools | High | Accessibility audit + user testing | Open |
| H6 | Dark/light theme with semantic color roles (not hardcoded) is baseline | High | Design system audit | Open |
| H7 | Toolbar customization (show/hide/reorder) increases perceived power-user value | Medium | Feature usage analytics | Open |

---

## 3. Research Questions Matrix

### Customer/User Discovery

| ID | Question | Priority | Method | Owner | Status |
|----|----------|----------|--------|-------|--------|
| Q1 | What are the primary user personas (planners, schedulers, PMs, superintendents)? | High | Interviews + analytics | UX Designer | Open |
| Q2 | Which settings are changed daily vs weekly vs once? | High | Telemetry + interviews | UX Designer | Open |
| Q3 | What toolbar configurations cause frustration? | High | Support ticket analysis | UX Designer | Open |
| Q4 | How do users discover advanced settings? | Medium | Usability testing | UX Designer | Open |
| Q5 | What's the mental model for "project settings" vs "app settings"? | High | Card sorting | UX Designer | Open |

### Technical Feasibility

| ID | Question | Priority | Method | Owner | Status |
|----|----------|----------|--------|-------|--------|
| T1 | Can egui support 3 density levels with token-based scaling? | High | Prototype | Lead Dev | Open |
| T2 | How to persist 4-level settings hierarchy (global/project/view/tool)? | High | Architecture spike | Lead Dev | Open |
| T3 | What's the resize performance with 5000+ activities in Gantt? | High | Benchmark | Lead Dev | Open |
| T4 | Can AccessKit expose dynamic toolbar density changes? | Medium | Accessibility test | A11y Lead | Open |
| T5 | How to sync settings across views without circular updates? | High | State mgmt spike | Lead Dev | Open |

### Business Viability

| ID | Question | Priority | Method | Owner | Status |
|----|----------|----------|--------|-------|--------|
| B1 | Which inspiration app patterns justify development cost? | High | ROI analysis | PO | Open |
| B2 | What's the competitive table-stakes vs differentiator split? | High | Competitive matrix | PO | Open |
| B3 | Which settings drive upgrade/retention in freemium models? | Medium | Market research | PO | Open |

### Scope & Boundaries

| ID | Question | Priority | Method | Owner | Status |
|----|----------|----------|--------|-------|--------|
| S1 | What's MVP settings scope vs v1.0 vs v2.0? | High | Roadmap workshop | PO + Lead Dev | Open |
| S2 | Which inspiration apps are out of scope (web-only, mobile-only)? | High | Platform filter | UX Designer | Open |
| S3 | What's explicitly NOT in scope (theming engine, plugin system)? | High | Scope doc | PO | Open |

---

## 4. Dependencies & Blockers

| Dependency | Type | Impact | Mitigation |
|------------|------|--------|------------|
| egui 0.36+ token system stability | Technical | High | Use stable egui version; fallback to manual scaling |
| AccessKit Windows/macOS screen reader support | Accessibility | Medium | Test with NVDA, VoiceOver, Narrator |
| MS Project XML round-trip for settings | Integration | Low | Defer to v2.0 |
| Primavera P6 .xer import for settings | Integration | Low | Defer to v2.0 |

---

## 5. Research Plan

### Activity 1: Competitive Settings Architecture Audit
**Questions Addressed:** Q1, Q2, Q3, Q5, T1, T2, H1, H2
**Method:** Systematic feature audit of 10 inspiration apps
- Create audit spreadsheet with 50+ criteria
- Screen-record settings flows in each app
- Document hierarchy depth, naming, grouping, search
- Rate each on: discoverability, density options, persistence, sync
**Duration:** 5 days
**Output:** Competitive Settings Matrix (Google Sheet + PDF summary)

### Activity 2: Toolbar Sizing & Resizing Interaction Audit
**Questions Addressed:** Q3, Q4, T3, H3, H7
**Method:** Interaction recording + measurement
- Record toolbar resize behaviors (mouse, keyboard, touch)
- Measure: min/max sizes, snap points, animation duration, ratio preservation
- Test with 100, 1000, 5000 activities in Gantt/table
- Document keyboard shortcuts for density cycling
**Duration:** 3 days
**Output:** Interaction Spec Document (Figma + markdown)

### Activity 3: Visual Design Language Extraction
**Questions Addressed:** H5, H6
**Method:** Design token extraction
- Extract color palettes (semantic roles, not hex)
- Extract typography scales (display, heading, body, caption, mono)
- Extract spacing systems (base unit, multipliers)
- Extract radius/elevation systems
- Extract motion specs (duration, easing, reduced-motion)
- Compare dark/light implementations
**Duration:** 3 days
**Output:** Design Token Library (JSON + documentation)

### Activity 4: Keyboard Navigation & Accessibility Audit
**Questions Addressed:** H5, T4
**Method:** Automated + manual testing
- Run axe-core / AccessKit tests on each inspiration app (where possible)
- Manual keyboard-only workflows (10 core tasks per app)
- Screen reader testing (NVDA, VoiceOver, Narrator)
- Focus visibility scoring
- Document skip links, landmarks, ARIA patterns
**Duration:** 2 days
**Output:** Accessibility Compliance Report

### Activity 5: Cross-Feature Integration Mapping
**Questions Addressed:** H4, S1
**Method:** Workflow mapping
- Map how settings propagate: Global → Project → View → Tool
- Identify circular dependency risks
- Document "source of truth" patterns
- Map undo/redo behavior with settings changes
**Duration:** 2 days
**Output:** Integration Architecture Diagram

---

## 6. Decision Framework

| Decision Point | Options | Criteria | Owner |
|----------------|---------|----------|-------|
| Density levels | 2 / 3 / 4 levels | User testing, implementation cost, competitor parity | UX Designer + Lead Dev |
| Settings hierarchy depth | 3 / 4 / 5 levels | Mental model fit, implementation complexity | UX Designer + Lead Dev |
| Resize behavior | Free / Grid-snap / Ratio-lock | User expectation, precision needs | UX Designer |
| Theme system | CSS-in-Rust tokens / egui themes / Custom | Maintainability, egui ecosystem | Lead Dev |
| Toolbar customization | None / Show-hide / Full reorder | Power-user value, complexity | PO + UX Designer |

---

## 7. Risk Register

| Risk | Likelihood | Impact | Mitigation | Owner |
|------|------------|--------|------------|-------|
| egui token system insufficient for 3 densities | Medium | High | Early prototype; fallback to manual scaling | Lead Dev |
| Settings sync creates circular updates | High | High | Single-source-of-truth architecture; immutable settings | Lead Dev |
| Accessibility regressions with custom toolbar | Medium | High | CI accessibility gates; manual testing each PR | A11y Lead |
| Scope creep: "plugin system" requests | Medium | Medium | Explicit out-of-scope list in spec | PO |
| Inspiration app patterns don't translate to desktop | Low | High | Validate with desktop-first apps (VS Code, Figma, Blender) | UX Designer |
| Token system changes in egui 0.37+ | Medium | Medium | Pin egui version; monitor releases | Lead Dev |

---

## 8. Target Implementation Epics

| Epic | Title | Relationship |
|------|-------|--------------|
| CADENCE-SETTINGS | Settings Architecture v1 | Direct implementation of discovered patterns |
| CADENCE-TOOLBAR | Toolbar System v1 | Density, sizing, customization, resize |
| CADENCE-DESIGN-TOKENS | Design Token System | Token library from Activity 3 |
| CADENCE-ACCESSIBILITY | Accessibility Hardening | Keyboard, screen reader, focus from Activity 4 |
| CADENCE-VIEW-SYNC | Cross-View Settings Sync | Integration from Activity 5 |

---

## 9. Linked Tickets

**Discovery Epic:** [CADENCE-UI-RESEARCH](https://jira.example.com/browse/CADENCE-UI-RESEARCH)

| Key | Summary | Type | Discovery Focus |
|-----|---------|------|-----------------|
| CADENCE-101 | Settings hierarchy specification | Task | T2, H2, S1 |
| CADENCE-102 | Toolbar density levels (3) implementation | Task | T1, H1, H7 |
| CADENCE-103 | Resize behavior spec (snap/ratio/grid) | Task | T3, H3 |
| CADENCE-104 | Design token library (colors/typography/spacing) | Task | H5, H6 |
| CADENCE-105 | Keyboard navigation audit + fixes | Task | H5, T4 |
| CADENCE-106 | Cross-view settings sync architecture | Task | H4, T5 |
| CADENCE-107 | Dark/light theme semantic color roles | Task | H6 |
| CADENCE-108 | Toolbar customization (show/hide/reorder) | Task | H7, S1 |

---

## Phase 3: Review & Publish

**Discovery Document Preview for CADENCE-UI-RESEARCH**

*Research Questions Identified:* 25
*Hypotheses to Validate:* 7
*High-Priority Items:* 12
*Target Implementation Epics:* 5

---

## Phase 4: Publish & Output

**Published to Confluence:** `/epics/Discovery/CADENCE-UI-RESEARCH/`
**Page Title:** "CADENCE-UI-RESEARCH Discovery - Inspiration Apps UI/UX Deep Research"
**Document URL:** https://confluence.example.com/pages/viewpage.action?pageId=CADENCE-UI-RESEARCH

---

## Discovery Document Complete!

**Discovery Document:** https://confluence.example.com/pages/viewpage.action?pageId=CADENCE-UI-RESEARCH

### Summary
- **Epic:** CADENCE-UI-RESEARCH - Inspiration Apps UI/UX Deep Research
- **Research Questions:** 25
- **Hypotheses to Validate:** 7
- **High-Priority Items:** 12
- **Target Implementation Epics:** CADENCE-SETTINGS, CADENCE-TOOLBAR, CADENCE-DESIGN-TOKENS, CADENCE-ACCESSIBILITY, CADENCE-VIEW-SYNC

### Immediate Actions Needed
1. Start Activity 1 (Competitive Settings Architecture Audit) - 5 days
2. Schedule stakeholder interviews for Q1, Q2, Q5
3. Set up egui prototype for T1 (density levels) validation
3. Create audit spreadsheet template for Activity 1

### Next Steps
After completing discovery research, run:

```
/discovery/synthesize https://confluence.example.com/pages/viewpage.action?pageId=CADENCE-UI-RESEARCH
```

This will synthesize findings and create actionable tickets in target implementation epics (CADENCE-SETTINGS, CADENCE-TOOLBAR, CADENCE-DESIGN-TOKENS, CADENCE-ACCESSIBILITY, CADENCE-VIEW-SYNC).