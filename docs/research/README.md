# Research notes

Research for capabilities not yet implemented lives here as **original
behavioural summaries with public URLs**. The clean-room rules in `guide.md`
apply: no proprietary source, no decompilation, no pasted copyrighted text,
no pixel-copying of another product's UI — public documentation, public
specifications and observed behaviour only.

## Human gate

Per the guide (§2.3, §8.4): before a large implementation Task depends on a
note in this directory, a human reviews it and approves it. Rejected notes are
rewritten or deleted. Approved notes become the acceptance criteria for the
work.

## Current notes

- [`printing-the-gantt-view.md`](printing-the-gantt-view.md) — capturing the
  rendered Gantt view and putting it into a PDF page.
- [`screen-reader-support.md`](screen-reader-support.md) — AccessKit
  integration through eframe.
- [`crash-report-submission.md`](crash-report-submission.md) — turning the
  local crash log into an optional GitHub Issue.
- [`update-check.md`](update-check.md) — checking GitHub Releases for a newer
  version, failing soft offline.

None of these have been approved for implementation yet; they are the reading
list for the next slices.
