# Token usage ledger

Measured record of model-token consumption for Cadence development
(guide §24). Measured-only rule: every number below comes from the
local OpenCode usage records — never estimated, never invented.

- **Data source:** the local OpenCode session store (SQLite, typically
  `~/.local/share/opencode/opencode.db`), read by
  `scripts/token-ledger.py`. The script reads only numeric usage
  columns and path/timestamp/ID metadata; it never reads message
  content.
- **Attribution (two tiers, D36).** *Tier A — direct:* sessions whose
  own directory is this repository's checkout, a `cadence-*` sibling
  worktree, or a directory the store's `worktree` table lists for
  this repository — the strict path-attributable floor, shown in the
  Direct (Tier A) column. *Tier A+B — orchestration-inclusive:*
  Tier A plus every ancestor session (the orchestrators that spawned
  direct sessions) and every descendant session (further subagents),
  transitively over the store's parent links. Release rows and
  `total_tokens_since_start` report the Tier A+B headline; the
  Direct (Tier A) column reports the strict subset.
- **Known caveat (conservative over-attribution).** An orchestrator
  session that worked across multiple repositories attributes wholly
  to Cadence under the ancestor rule — the store records no
  per-repository split inside a single session. Tier A+B therefore
  leans high; the direct column is the strict lower bound. Both are
  measured figures; nothing is estimated or interpolated.
- **Privacy.** The script opens the store read-only and touches only
  numeric usage columns and opaque path/ID/timestamp metadata
  (including the parent links used for attribution); message and
  part tables, message JSON and session titles are never read.
- **Regenerate:** run `python3 scripts/token-ledger.py` before
  tagging a release; it rewrites this file and `docs/tokens.json`
  together.

| Version | Date | Tokens this release | Tokens cumulative | Direct (Tier A) | Notes |
|---------|------|--------------------:|------------------:|----------------:|-------|
| v0.1.0 | 2026-10-24 | 11,802 | 11,802 | 11,802 | |
| v0.1.1 | 2026-10-24 | 7,070 | 18,872 | 7,070 | |
| v0.3.0 | 2026-10-28 | 1,125,404,657 | 1,125,423,529 | 13,333 | |
| v0.8.0 | 2026-11-01 | 0 | 1,125,423,529 | 0 | |
| v0.9.0 | 2026-11-02 | 3,479,360 | 1,128,902,889 | 3,479,360 | |
| v0.10.0 | 2026-11-02 | 0 | 1,128,902,889 | 0 | |
| v0.11.0 | 2026-11-06 | 579,494,770 | 1,708,397,659 | 0 | |
| v0.12.0 | 2026-11-07 | 0 | 1,708,397,659 | 0 | |
| v0.12.1 | 2026-11-07 | 0 | 1,708,397,659 | 0 | |
| v0.13.0 | 2026-11-15 | 3,279,251 | 1,711,676,910 | 3,279,251 | |
| v0.14.0 | 2026-11-16 | 0 | 1,711,676,910 | 0 | |
| v0.15.0 | 2026-11-19 | 0 | 1,711,676,910 | 0 | |
| v0.16.0 | 2026-11-20 | 0 | 1,711,676,910 | 0 | |
| v0.16.1 | 2026-11-20 | 0 | 1,711,676,910 | 0 | |
| v0.16.2 | 2026-11-20 | 0 | 1,711,676,910 | 0 | |
| v0.17.0 | 2026-11-21 | 0 | 1,711,676,910 | 0 | |
| v0.17.1 | 2026-11-21 | 0 | 1,711,676,910 | 0 | |
| v0.18.0 | 2026-11-21 | 3,904,279 | 1,715,581,189 | 3,904,279 | UI redesign v0.17.0 + Wayland fix v0.17.1 + egui 0.36 upgrade |
