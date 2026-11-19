# Resource catalog

Adopted and evaluated external resources, recorded per guide §25 use-rule 2:
catalog reference, source URL, licence, version/date pinned, and the
`DECISIONS.md` reference for each adoption. Per §25 use-rule 3, the source
repository's own LICENSE file governs over any catalog or guide-table claim;
discrepancies found during review are stated in the licence column.

## Agent skills and reference tooling (guide §19/§22 — reviewed 2026-11-19)

Per §25 use-rule 3 the source repo's LICENSE governs over any catalog/guide
claim; per §25 use-rule 2 the adoption decisions cross-reference
`DECISIONS.md` — **D31** (installation, 2026-11-18) and **D35** (relocation +
invocation policy + pstack licence verification, 2026-11-19). Every item below
is **maintainer-machine tooling or process posture**: nothing here ships in
the Cadence binary or the release assets (D31; `ATTRIBUTION.md`).

| Item | Source URL | Catalog | Licence (verified from source) | Pinned | Decision | Cadence application |
|------|-----------|---------|-------------------------------|--------|----------|---------------------|
| pstack skill pack | https://github.com/cursor/plugins/tree/main/pstack | guide §22 | **MIT** (© Lauren Tan; verified from `pstack/LICENSE` and `plugin.json` — the guide §22.2.3 "Apache-2.0" claim is superseded per §25 use-rule 3) | 2026-11-18 (D31) | **Concepts adapted; `poteto-mode` installed** (D31, relocated to `~/.config/opencode/skill/` per D35) and verified active in-session | Playbooks are read as reference per §22.2 (understand the existing system → choose a design → verify on the real app → review the diff → report). The repo's §5.6.3 command files (PR #176) already operationalise its verification and PR postures: `/tdd` red-run CI IDs, `/code-review` axes, `/grill-me` → `/to-spec` → `/to-tickets` → `/implement`. No pstack code is vendored into the repository. |
| caveman skill pack | https://github.com/JuliusBrussee/caveman | guide §19 | **Apache-2.0** (verified from the repository's LICENSE, per D31 — supersedes the §19 table's "MIT" claim) | 2026-11-18 (D31) | **Installed** (D31, relocated per D35); invocation **manual-only** | Terse-style aid for maintainer-side sessions; toggled by the owner, never auto-applied (owner invocation policy, D35). The §II.22 plain-language rule wins for release notes, PR text and any user-facing copy, so caveman output never reaches repo artefacts. |
| rtk (command-output proxy) | https://github.com/rtk-ai/rtk | guide §19 | Apache-2.0 | v1.2.0, 2026-11-18 (D31) | **Installed** (D31) — prebuilt `aarch64-unknown-linux-musl` binary at `~/.local/bin/rtk` (no brew on this machine; `cargo install` banned, §4 item 5) | **Not yet wired into daily use.** Adoption-for-use is deferred pending a measured trial; per the measured-only rule (§19 rule 3) any token-savings figure must come from this project's own runs recorded in `docs/tokens.md`, never vendor claims. |
| code-context-engine | https://github.com/elara-labs/code-context-engine | guide §19 | MIT (core); one optional component dual AGPL-3.0/commercial | — (not adopted) | **Skipped for now** | §19 rule 6 — one context tool at a time; rtk already occupies output filtering. Revisit only with a DECISIONS-recorded reason (including the AGPL split recorded in `ATTRIBUTION.md` before any adoption). |
| headroom | https://github.com/headroomlabs-ai/headroom | guide §19 | Apache-2.0 | — (not adopted) | **Skipped for now** | Same one-context-tool rule as code-context-engine. |
| code-review-graph | https://github.com/tirth8205/code-review-graph | guide §19 | MIT | — (not adopted) | **Skipped for now** | Same one-context-tool rule; its optional `fail-on-risk` gate would also need a DECISIONS-recorded threshold (§19 rule 7) — deferred with the tool. |
| §19 Top-10 concepts: superpowers (test-first), karpathy code-taste, ponytail (less code), addyosmani agent-skills (spec→ship), ui-ux-pro-max + taste-skill (design intelligence) | see guide §19 table | guide §19 | varied — per upstream repo, checked at any future adoption | — (not installed) | **Covered by repo-native equivalents — not installed** | The test-first / spec→ship / review postures live in the §5.6.3 command files (`tdd`, `code-review`, `grill-me`, `to-spec`, `to-tickets`, `implement`, `implement-spec` — PR #176); the design-intelligence role is carried by guide §20/§21/XI design checklists plus §II.30 stack-matching (web design kits are inspiration-only for the egui app). **taste-skill / ui-ux-pro-max noted as candidate reference for the upcoming review-panel UI slice (RP-3/RP-4)**, egui-adapted only. |
| §25.4 awesome-llm-apps skill patterns: Project Graveyard, First Reader, Scope Creep Detector, Commit Archaeologist, Dependency Doctor, Advisor Orchestrator Worker | https://github.com/shubhamsaboo/awesome-llm-apps | §25.4 | Apache-2.0 (catalog claim; re-verify against the source at any adoption) | — (patterns only) | **Pattern names recorded for discovery** | Four of these patterns already exist natively in the skill directory (verified 2026-11-19): `commit-archaeologist`, `dependency-doctor`, `first-reader`, `scope-creep-check`. The remaining two (project-graveyard, advisor-orchestrator-worker) are deferred until a task needs them. |

### Review notes (2026-11-19)

- **Licence discrepancies found and resolved per §25 use-rule 3:** the guide
  §19 table lists caveman as MIT — the source repository's LICENSE, as
  source-verified and recorded in D31, says **Apache-2.0**. The guide §22.2.3
  text claims pstack is Apache-2.0 — the source `pstack/LICENSE` and
  `plugin.json` say **MIT** (© Lauren Tan), as source-verified and recorded in
  D35. `ATTRIBUTION.md` already carries both corrected values.
- **No shipped-artefact impact:** all items are maintainer-machine tooling or
  process posture (D31, D35). No application source, asset, CI definition or
  release artefact is affected by this catalog.
- **Clean-room unchanged** (D31; §19 rule 5): these are style, rigour and
  efficiency aids; everything they help produce stays subject to the same
  clean-room test, CI gates and review as any other change.
- **Measured-only rule** (§19 rule 3): no token-savings figure from any vendor
  appears here; any measured effect of rtk or the skill packs belongs in
  `docs/tokens.md`, from this project's own runs.
