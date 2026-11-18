---
description: Interrogation phase — resolve every ambiguity before any code or plan is written (guide §5.6.3).
agent: plan
---

# /grill-me — interrogate before implementing

**Purpose.** Reach alignment on edge cases, data structures and failure
modes for a feature before any code or implementation plan exists.

**Invocation.** `/grill-me <feature or task description>` — read-only by
construction: the `agent: plan` frontmatter keeps this command unable to
edit files.

**Binding rules (guide §5.6.3):**

- Do not write code or implementation plans until **every** ambiguity is
  resolved.
- Ask targeted questions **one by one**, each with a recommended answer
  derived from existing Cadence patterns rather than invention — the crate
  layering (core / commands / persistence / app, D3), the single mutation
  path through `cadence-commands`, the design tokens in
  `crates/ui-egui/src/design.rs`, and the recorded decisions in
  `DECISIONS.md`.
- Record each resolved decision as you go; they are the raw material for
  `/to-spec`.
- `/new` before the next phase (§5.6.1): interrogation context never
  carries into specification or implementation.
- Clean-room check: if an answer would come from a proprietary source
  (decompiled binaries, copied manuals, string tables or presets), stop and
  route the question through the §2.3 human gate instead.

**Reconciliation.** Where anything here could conflict with the guide, the
reconciliation table in guide §5.6.5 governs: verification runs through
`ci.sh` only, git actions carry the `Cadence Dev <dev@example.com>`
identity, and the clean-room, secrets and branch-protection rules always
win.
