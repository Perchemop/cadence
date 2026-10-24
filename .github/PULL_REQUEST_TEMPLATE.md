# Pull request

## Summary
<!-- What this PR does in 1–3 sentences -->

## Type
- [ ] Feature
- [ ] Fix
- [ ] Docs / research (summaries + URLs only)
- [ ] Infra (CI, release, shared crates)
- [ ] Refactor

## Checklist
- [ ] Acceptance criteria for the Task are met with **real** behaviour (no fake UI)
- [ ] Tests added or updated
- [ ] CI is green (or will be after merge of required base)
- [ ] No unbounded local full builds used to “verify” on a constrained machine
- [ ] No proprietary source, assets, or RE material introduced
- [ ] Public research notes are original summaries + URLs only (if applicable)
- [ ] ROADMAP / CHANGELOG / parity updated if this completes a tracked item
- [ ] Secrets and tokens not committed
- [ ] User-facing strings and UI are original (no copied trade dress)
- [ ] New dependencies are within the allowed licence classes, recorded in `ATTRIBUTION.md`, and pass the security workflow's licence check
- [ ] No alert from the secret-scanning or SAST job, or every finding is triaged and recorded
- [ ] Any privacy-affecting change (file paths, network, logs) is reflected in the README privacy note

## Accessibility (complete for any change to the UI)
- [ ] The whole change is operable from the keyboard, with a visible focus indicator and a logical tab order
- [ ] Every control is labelled, and errors are stated in text and announced rather than shown by colour alone
- [ ] Meaning is not carried by colour or styling alone (text label, icon with text, or shape is also present)
- [ ] The OS “reduce motion” preference is respected by anything animated
- [ ] A keyboard-only pass over the affected flow has been done, plus one screen-reader spot check
- [ ] Any new limitation is recorded in `ACCESSIBILITY.md`

<!-- If this PR does not touch the UI, replace the section above with "Not applicable — no UI change." -->

## Documentation (complete for any documentation change)
- [ ] Heading levels are in order and are not skipped
- [ ] Link text is descriptive, and every new link resolves
- [ ] Tables are used only for tabular data and carry headers
- [ ] Plain language, with abbreviations expanded on first use

## Local verification
<!-- How a human can try this on the primary OS -->

## Notes / risks
