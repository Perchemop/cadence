# Elite Tool Uplift – Investigation Summary

Date: 2026-11-08
Version: 0.12.1

## Findings

Parallel agent investigations completed:

* **explore**: F1-F25 gap analysis vs MS Project/P6/Asta/ProjectLibre/Aphex. Audit findings synthesized.
* **ui-dev**: `docs/design/paid-tool-ui-uplift.md` – iconography, widget layer, date picker, theme persistence, crash dialog, release notes.
* **domain-dev**: Parity analysis and clean-room sketches for mandatory constraints, drag/resize bars, WBS collapse, two-tier timescale, progress/status line.

Critical perception gaps:
* Non-working day shading (V5) – the single most recognizable professional Gantt texture
* Two-tier timescale (V6), WBS expand/collapse (V7)
* Drag-to-move/resize bars (V8)
* Iconography V1/V16, DatePicker V12, token colours V9/F12, monospace numerics V10/F20

Functional blockers still required: F1 Open guard, F13 dialog keyboard, F22 confirm deletes, F17 terminology.

## Prioritized backlog (top 10)

1. Non-working day shading from assigned calendar – V5
2. Token colours on canvas/grids – V9/F12
3. Phosphor iconography for toolbar/tabs/inspector – V1/V16
4. WBS expand/collapse outline – V7
5. Drag-to-move/resize Gantt bars – V8
6. Two-tier timescale – V6
7. Monospace numerics everywhere + right align – V10/F20
8. Status-bar wrap and zoom tooltip – V2/F14
9. DatePicker replacing raw YYYY-MM-DD – V12
10. Progress line + status-date line on chart – V8

## Next slices

* **P2-visual**: Non-working shading, token colours, monospace numerics, status-bar wrap
* **P2-chrome**: Iconography, DatePicker, WBS collapse UI state
* **P3-interaction**: Drag/resize bars, two-tier timescale, progress/status line
* **P3-domain**: Mandatory constraints, preference persistence

All recommendations clean-room. No proprietary UI/strings/icons copied.
