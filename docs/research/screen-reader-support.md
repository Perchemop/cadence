# Research — screen-reader support (AccessKit)

Status: draft, not human-approved.

## Behaviour wanted

Screen readers announce the window, the toolbar buttons, the activity table
cells and the dialogs by role and label. Today the egui UI is keyboard
navigable in part, but accessibility-tree integration depends on eframe's
AccessKit feature being enabled at build time.

## Acceptance criteria

1. Enabling the feature does not change the visual UI.
2. On Windows (Narrator), macOS (VoiceOver) and Linux (Orca) the toolbar
   buttons and window title are announced.
3. Table rows announce activity name and key dates when focused.
4. CI stays green; the feature is compile-time and adds no behavioural risk to
   tests.

## Implementation direction (original notes)

- eframe/egui integrate with AccessKit behind a cargo feature; enabling it on
  the `eframe` dependency turns on the accessibility tree without code
  changes for standard widgets. Feature flag documentation:
  https://docs.rs/eframe/latest/eframe/#feature-flags
- AccessKit project overview (what the tree is, which platforms are covered):
  https://accesskit.dev/
- Custom-painted widgets (the Gantt canvas) need explicit nodes with roles and
  bounds; that is the real work, and it is bounded to the Gantt view because
  every other surface uses standard widgets.
- Verification is interactive by nature (screen readers on three platforms),
  so this is a human-verified slice even though the code change is small.

## Sources

- eframe feature flags: https://docs.rs/eframe/latest/eframe/#feature-flags
- AccessKit: https://accesskit.dev/
- egui accessibility discussion (context, not an instruction source):
  https://github.com/emilk/egui/issues/orthogonal — replaced during approval
  review with the current upstream tracking issue.
