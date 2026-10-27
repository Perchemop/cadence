# Current model map

Which free model is assigned to which role right now (guide.md section 5.2).
The router selects these live; this file records what was actually chosen so a
later session knows where it left off. Update it when the mapping changes.

Provider order is **OpenCode free -> NVIDIA free -> paid API keys**, enforced by
the failover plugin, so the structure never depends on which model is running.

## Active provider

- Tier 1 (OpenCode free): Not exhausted at last check
- Tier 2 (NVIDIA free): Available as tier 2
- Tier 3 (paid API keys): Groq and Google, reached only after both free tiers
  are exhausted.

## Role assignment

| Role | Model | Why |
| --- | --- | --- |
| orchestrator (primary) | `opencode/big-pickle` | planning, delegation, tool calling |
| coder | `opencode/glm-5.3-flash-free#high` | code generation, multi-file edits |
| fixer | `opencode/glm-5.3-flash-free#high` | minimal targeted fixes from CI logs |
| explorer | `opencode/mimo-v2.6-flash-free` | speed for search and reads |
| reviewer | `opencode/beam-free#high` | cheap, competent read-only analysis |
| tester | `opencode/deepseek-v4.1-flash-free#high` | high-volume test writing |
| docs-writer | `opencode/big-pickle` | low-stakes documentation edits |
| ci-runner | `opencode/beam-free#high` | short, mechanical GitHub interaction |
| designer (UI/UX) | `opencode/nemotron-3.5-super-vl-free#high` | vision-capable; design + performance review |

## Fallback assignment (when OpenCode free is exhausted)

| Role | NVIDIA free model |
| --- | --- |
| orchestrator | `nvidia/stepfun-ai/step-5-preview` |
| coder / fixer | `nvidia/z-ai/glm-5.3` |
| explorer | `nvidia/openai/gpt-oss-20b` |
| reviewer | `nvidia/z-ai/glm-5.3-flash` |
| tester | `nvidia/z-ai/glm-5.3-flash` |
| ci-runner | `nvidia/openai/gpt-oss-20b` |
| ui-dev (vision) | `nvidia/nvidia/nemotron-3.5-super-vl-120b-a12b` |

## Notes

- Model IDs go stale. Verify against the live list at the start of a session and
  correct this table rather than trusting it.
- Never use Ollama or any local model hosting.
- No model change affects the process: heavy work runs in GitHub Actions.

## Public research method (guide §2.4)

Product research for future expansion is done with the session's web-search and
page-fetch tools (the same free-model tier), restricted to **public** pages:
vendor product/help pages, public standards (DCMA, GAO, AACE, PMBOK), public
tutorials and public talks. Notes are written as **original behavioural
summaries plus URLs** in `docs/research/`; source text is never pasted, and no
proprietary source, decompilation output or third-party screenshots are
committed. Context7 is not used for product research — it indexes library
documentation, not arbitrary public pages. Each note goes through the human
gate (guide §2.3) before a large implementation depends on it.

## Verified availability (re-checked 2026-10-22)

Live probes of the NVIDIA free tier, which is tier 2 in the provider order:

| Model | Result |
| --- | --- |
| `nvidia/z-ai/glm-5.3-flash` | reachable |
| `nvidia/stepfun-ai/step-5-preview` | reachable |
| `nvidia/openai/gpt-oss-20b` | reachable, intermittently |
| `nvidia/z-ai/glm-5.3` | **not reachable** — `socket connection was closed`; removed from the routing chains |

`nvidia/meta/llama-guard-4-12b` and `nvidia/google/google-paligemma` are guard
and vision models, not coding models, so they are not used for roles.

The router in `~/.config/opencode/failover.json` (`fallbacks` +
`fallbacksByAgent`) carries this order. It never re-chooses a model that has
already failed in the same session, and a model that stops connecting is
dropped after two attempts, so a stale ID degrades into the next tier instead
of stalling the session.
