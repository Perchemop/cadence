# Cadence brand

Original identity for Cadence. Nothing here is derived from a commercial
product's look or marks.

## Name

**Cadence** — the rhythm of work: a construction schedule is a sequence of
activities in time, and the word also reads as professional software. Chosen
for originality (no scheduling product on the shortlist used it), short,
pronounceable, and free of vendor trademark conflicts on the checklists used
at discovery.

## Window and product identity

- Window title: `Cadence — <project name>`
- CLI binary: `cadence`
- About text: version from the crate metadata; no vendor claims.

## Visual language

The UI is built from original design tokens in `cadence-ui-egui`:

| Token | Light | Dark | Meaning |
|---|---|---|---|
| Accent | `#2F6FED` | `#2F6FED` | selection, highlights, buttons |
| Critical | `#C0392B` (light) / `#E74C3C` (dark) | as stated | critical path, over-allocation, delays |
| Green | `#1E8449` (light) / `#2ECC71` (dark) | as stated | improvements, gains against baseline |
| Neutral bars | `#9BC1FF` | as drawn | non-critical activity bars |

Contrast is a tested property: WCAG AA (4.5:1) is asserted by unit tests for
text on both themes (`accessibility_tests` in `cadence-ui-egui`), and 3:1
non-text contrast for the accent.

## Logo and icons

No logo yet. When one is wanted it will be drawn as an original SVG under
`assets/` and recorded here and in `ATTRIBUTION.md`; the guide's rule is
agent- or hand-authored SVG, never a competitor's mark or trade dress.

## Voice

Short, concrete sentences. Schedules say what happened; reports never invent
dates or claim precision the engine does not have.
