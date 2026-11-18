# Token usage ledger

Measured record of model-token consumption for Cadence development
(guide §24). Measured-only rule: every number below comes from the
local OpenCode usage records — never estimated, never invented.

- **Data source:** the local OpenCode session store (SQLite, typically
  `~/.local/share/opencode/opencode.db`), read by
  `scripts/token-ledger.py`. The script reads only numeric usage
  columns and path/timestamp metadata; it never reads message content.
- **What counts:** the sum of the store's per-session token columns
  (input, output, reasoning, cache read, cache write) for sessions
  attributed to this repository by path (the checkout and its
  `cadence-*` worktrees), bucketed by release-tag date.
- **Known limitation (conservative bias):** sessions launched from the
  home directory — including orchestrator sessions and their
  subagents — are not attributable to a repository by this store and
  are excluded. This ledger is therefore a measured **floor**, not a
  ceiling.
- **Regenerate:** run `python3 scripts/token-ledger.py` before tagging
  a release; it rewrites this file and `docs/tokens.json` together.

| Version | Date | Tokens this release | Tokens cumulative | Notes |
|---------|------|--------------------:|------------------:|-------|
| v0.1.0 | 2026-10-24 | 11,802 | 11,802 | |
| v0.1.1 | 2026-10-24 | 7,070 | 18,872 | |
| v0.3.0 | 2026-10-28 | 13,333 | 32,205 | |
| v0.8.0 | 2026-11-01 | 0 | 32,205 | |
| v0.9.0 | 2026-11-02 | 3,479,360 | 3,511,565 | |
| v0.10.0 | 2026-11-02 | 0 | 3,511,565 | |
| v0.11.0 | 2026-11-06 | 0 | 3,511,565 | |
| v0.12.0 | 2026-11-07 | 0 | 3,511,565 | |
| v0.12.1 | 2026-11-07 | 0 | 3,511,565 | |
| v0.13.0 | 2026-11-15 | 3,279,251 | 6,790,816 | |
| v0.14.0 | 2026-11-16 | 0 | 6,790,816 | |
