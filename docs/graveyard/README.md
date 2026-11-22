# Cadence Project Graveyard

**Tracks abandoned approaches, failed experiments, and dead ends.** Prevents rework, feeds `docs/process-feedback.md`.

| Date | Approach | Ticket | Failure Type | Lesson |
|------|----------|--------|--------------|--------|
| | | | | |

## Entry Template

Create entries in `docs/graveyard/<YYYY-MM-DD>-<slug>.md`:

```markdown
# Graveyard: <Approach Name>

## Date
<YYYY-MM-DD>

## Context
**Ticket:** <ticket-id>
**Goal:** What we were trying to achieve
**Approach:** What we tried (code, design, dependency, pattern)

## What Failed
**Symptom:** What went wrong (compile error, test failure, performance, UX)
**Root Cause:** Why it failed (fundamental limitation, wrong abstraction, missing feature)
**Evidence:** Links to failed CI runs, error logs, benchmark results

## Why It Seemed Promising
What made us think this would work? (prior art, docs, assumptions)

## Lessons Learned
1. <Lesson 1>
2. <Lesson 2>
3. <Lesson 3>

## Alternatives to Try Instead
- [ ] <Alternative 1: why it might work>
- [ ] <Alternative 2: why it might work>

## Related
- **Superseded by:** <link to successful approach>
- **Blocks:** <tickets that depended on this>
- **Process Feedback:** <link to docs/process-feedback.md entry>
```