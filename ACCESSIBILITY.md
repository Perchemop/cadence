# Accessibility

Cadence ships as a desktop application (egui/eframe) and a command-line tool.
Accessibility work here splits into two different problems, and this document
states where each one stands.

## Target standard

**WCAG 2.2 AA**, applied to what an egui desktop application can meaningfully
support. Web-native requirements that have no equivalent in a native window are
recorded as not applicable rather than ignored silently.

## Priorities, in order

1. **Keyboard support.** Every task is completable without a pointer.
   Currently: view switching with `Ctrl+1`–`Ctrl+5`, standard editing and
   navigation inside statistics, grids and input fields.
2. **Screen-reader semantics.** Button and control purposes readable from text,
   error messages announced, no information carried by colour alone.
3. **Contrast.** Text and structural elements at AA contrast against the
   default theme, at default font sizes.
4. **Screen-reader friendly output.** The CLI's plain-text reports are already
   screen-reader usable by construction.

## Known limitations and current work

Documented honestly, including what is not done:

- **Colour-only signals.** The Gantt view marks critical activities, and the
  Resources view marks over-allocated days, by colour. These meanings are also
  expressed in text in the same views, but the Gantt colour-coding itself is not
  yet fully annotated in the exported PDF. Tracked as a11y work.
- **egui limits.** egui does not expose a full accessibility tree, so
  assistive-technology support is limited to what the platform provides for the
  toolkit. This is a toolkit constraint, not a design choice, and it is the
  reason screen-reader support is not claimed as complete.
- **Focus order in complex panels.** Some compound panels order their focusable
  controls by construction order, which is usually but not guaranteed to be the
  reading order.

## Supported environments

- Linux, Windows, macOS desktop builds published from releases.
- The CLI on the same three platforms.
- No web or mobile build is planned; no claims are made for those.

## Reporting an accessibility problem

Use the **Accessibility report** issue template (label `accessibility`), not the
general bug template. The template asks for what matters here: what you were
trying to do, which assistive technology and OS, and what Cadence did instead.

Accessibility reports are treated as correctness reports, not opinions. If a
feature is unusable by a whole group of people, that is a defect in the feature.
