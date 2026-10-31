# Cadence design brand

The design vision for Cadence: meaning, shapes, colours, type, spacing —
original work in the sense of guide Part II §27.4. This document is the source
of truth for the *look*; the implementation lives in
`crates/ui-egui/src/design.rs` and `apply_theme`.

## Positioning

Cadence is a professional scheduling tool used at a desk for hours. The
reference class is paid desktop software — MS Project, Primavera P6, Aphex —
not web dashboards and not toy apps. The design must read as: calm, dense,
precise, fast. Every pixel either carries schedule information or gets out of
the way.

## Meaning and shapes

- **The schedule is the hero.** Panels are quiet chrome; the table, the Gantt
  and the audit findings get the contrast and the space.
- **Squares with soft corners.** 4 px corner radius on interactive widgets,
  6 px on containers — engineering-tool solid, not playful.
- **Hairline structure.** 1 px separators and grid lines in a low-contrast
  stroke; colour is reserved for meaning (critical, variance, selection).
- **Data is monospaced.** Dates, floats and counts render in a monospaced
  face so columns align and digits don't jitter.

## Colour

Two themes, light and dark, built on a neutral slate/graphite ramp with one
restrained accent and semantic colours for schedule meaning:

| Token | Light | Dark | Meaning |
|---|---|---|---|
| `bg` | `#F4F5F7` | `#16181D` | window background |
| `panel` | `#FFFFFF` | `#1D2026` | panels, grids, popups |
| `text` | `#1F2933` | `#E3E7EB` | body text |
| `text-muted` | `#5F6B7A` | `#9AA5B1` | secondary text, units |
| `accent` | `#2F6FED` | `#2F6FED` | selection, links, focus (the `ACCENT` constant, AA-verified on both themes) |
| `critical` | `#B91C1C` | `#F87171` | critical bars, failures |
| `normal` | `#1D4ED8` | `#93C5FD` | non-critical bars |
| `ok` | `#15803D` | `#4ADE80` | passes, on-track |
| `warn` | `#B45309` | `#FBBF24` | warnings |
| `grid` | `#E5E7EB` | `#2A2E36` | hairlines, Gantt grid |

The accent must keep WCAG AA (4.5:1) against panel backgrounds in both themes,
enforced by the accessibility tests.

## Type

- **UI face: Inter** (SIL OFL 1.1, bundled in `assets/fonts/`, recorded in
  `ATTRIBUTION.md`) — a neutral, dense, highly legible UI face designed for
  screens. Sizes: 11 px body, 13 px strong, 16 px headings, 24 px window
  title. Small caps are not used; hierarchy comes from size and weight.
- **Numeric face: monospace** for every date, float and count column, at body
  size, slightly dimmed so words lead the eye.

## Spacing and rhythm

- 4 px base grid: 4, 8, 12, 16, 24. Widget inner padding 6×4. Item spacing
  8, button padding 8×4, grid spacing 12×4.
- One panel margin of 12 px; no double margins inside panels.

## Motion

- Reduced to near zero. Windows appear without animation; the only
  transitions are egui's native ~100 ms hover fades. The OS reduce-motion
  preference is respected by doing nothing custom.

## Iconography

- No icons in v1: text labels everywhere. An original SVG icon set is a later
  slice and would live in `assets/icons/` with an ATTRIBUTION entry.

## Voice

- Plain, specific, unhysterical. States are sentences ("Nothing is scheduled
  in this window yet"), never exclamations. Errors say what to do next.
