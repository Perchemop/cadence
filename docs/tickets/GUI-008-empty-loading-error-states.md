# Ticket: GUI-008 — Polish Empty/Loading/Error States

## Objective
Implement polished empty, loading, and error states for Cadence UI views, matching the pattern from Linear, Planera, and Airtable. This is a **medium priority** improvement identified in the GUI Inspiration Audit.

## Target Files
- `crates/design/src.rs` — Empty state constants, loading skeleton definitions, error component
- `crates/ui-egui/src/lib.rs` — Integration of improved states into all views

## Empty States (from research)
### Empty Project
- Display: "Create your first activity" + CTA button
- CTA: Opens/adds first activity
- Shows benefit/value of the application

### Empty Filtered Table
- Display: "No activities match filter" + clear filter button
- Clear filter: Removes filter, shows all activities
- Maintains context of what filter was active

### Empty Results (search, audit, etc.)
- Display: "No results found" with possible actions
- No dead controls; offers meaningful next step

## Loading States (from research)
- **Skeleton screens**: Match the exact shape of real content
- Subtle shimmer animation, not aggressive pulsing
- Used for: project load, update check, report generation, any async operation
- Three density variants compatible

## Error States (from research)
### Current: Status bar only
- Need: Inline + toast, never modal-only

### Improved: Inline + Toast
- **Inline error**: Directly below the control that errored, explains the problem
- **Toast error**: Top-right toast with error details and dismiss action
- **Never modal-only**: Error never blocks workflow without alternative

## Error Message Patterns (from research)
- **Validation error**: "Value must be a positive number"
- **Not found**: "No activities match the criteria"
- **Operation failed**: "Could not load project; check file format"
- **Network error**: "Could not check for updates; you are offline"

## Implementation Details (from design system gaps)
```rust
/// Empty state display type
pub enum EmptyStateType {
    EmptyProject,           // "Create your first activity"
    EmptyFilteredTable,     // "No activities match filter"
    NoSearchResults,        // "No results found"
    NoAuditFindings,        // "Program passes all checks"
}

/// Loading state component
pub struct LoadingState {
    pub r#type: LoadingType,
    pub message: Option<String>,  // optional custom message
}

/// Error state component
pub struct ErrorState {
    pub r#type: ErrorType,
    pub message: String,          // user-facing error message
    pub action: Option<String>,   // optional action (e.g., "Try Again")
    pub show_toast: bool,         // whether to also show a toast
}
```

## Blocking Dependencies
- GUI-002 (skeleton loaders - can be parallel)
- GUI-006 (toast notifications - can be parallel)

## Acceptance Criteria
- [ ] Empty project: "Create your first activity" CTA button displayed
- [ ] Empty filtered table: "No activities match filter" + clear filter button
- [ ] Loading states: skeleton screens for project load, update check, report generation
- [ ] Error states: inline messages below affected control
- [ ] Error states: toast notifications for global errors
- [ ] Error states: never modal-only; always has alternative workflow
- [ ] All states respect light/dark themes
- [ ] All states respect density mode

## Verification
- Headless test: Empty project on start → CTA displays → add activity → CTA removed
- UI test: Filter → no results → "No activities match" with clear filter → activities appear
- Integration: Empty/loading/error states work with skeleton loaders, density modes, tool palette
- Edge case: Very long loading → skeleton persists appropriately
- Edge case: Multiple error types → different inline/toast patterns for each