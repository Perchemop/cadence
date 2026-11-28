# Ticket: AI-005 — Broader Assistant Proposal Kinds

## Objective
Extend the proposal pipeline beyond the initial 4 change kinds to support additional assistant capabilities, incorporating all proposed command types from the AI integration research.

## Target Files
- `crates/assistant/src/lib.rs` — Extend `ProposedChange` enum with additional command types
- `crates/commands/src/lib.rs` — Handle new command types in `ApplyProposedChanges`
- `crates/ui-egui/src/lib.rs` — Review panel shows expanded change kinds

## ProposedChange Enum Extension (from AI integration research)
Extend the existing `ProposedChange` enum with these additional variants:

```rust
#[derive(Clone, Debug)]
pub enum ProposedChange {
    // Initial 4 kinds (from AI-002)
    CreateActivity {
        name: String,
        duration_days: u32,
        rationale: String,
    },
    CreateRelationship {
        predecessor: String,
        successor: String,
        rationale: String,
    },
    SetActivityDuration {
        activity: String,
        duration_days: u32,
        rationale: String,
    },
    SetActivityProgress {
        activity: String,
        percent: u8,
        rationale: String,
    },
    
    // Additional kinds from research
    SetActivityActualDates {
        activity: String,
        actual_start: Option<ChronologicalDate>,
        actual_finish: Option<ChronologicalDate>,
        rationale: String,
    },
    SetResourceAssignment {
        activity: String,
        resource: String,
        role: Option<String>,
        units: f64,
        rationale: String,
    },
    SetCalendar {
        activity: String,
        calendar: String,
        rationale: String,
    },
    CreateBaseline {
        baseline_name: String,
        description: String,
    },
    GenerateReport {
        report_type: ReportType,
        time_range: TimeRange,
        included_fields: Vec<String>,
    },
}
```

## ReportType Enum
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportType {
    StatusReport,    // narrative schedule status
    ProgressReport,  // percent complete and milestone progress
    VarianceReport,  // baseline vs. actual comparison
    AuditReport,     // schedule health check results
    Custom(String),  // user-specified report type
}
```

## TimeRange Enum
```rust
#[derive(Clone, Debug)]
pub struct TimeRange {
    pub start: Option<ChronologicalDate>,
    pub end: Option<ChronologicalDate>,
}
```

## Review Panel Extension (from research)
- Review panel renders before/after for all change kinds
- Each change kind has appropriate validation (e.g., resource assignment validates resource exists, calendar validates weekday pattern)
- "Apply All / Apply Selected / Discard" works uniformly across all kinds
- New change kinds have appropriate UI patterns in the review panel

## Blocking Dependencies
- AI-001, AI-002 (foundational proposal infrastructure)

## Acceptance Criteria
- [ ] `ProposedChange` enum includes: CreateActivity, CreateRelationship, SetActivityDuration, SetActivityProgress, SetActivityActualDates, SetResourceAssignment, SetCalendar, CreateBaseline, GenerateReport
- [ ] Review panel renders before/after for all change kinds
- [ ] "Apply All / Apply Selected / Discard" works uniformly across all kinds
- [ ] Each change kind has appropriate validation:
  - Resource assignment validates resource exists
  - Calendar validates weekday pattern
  - Actual dates validate finish >= start
  - Report generation has appropriate fields
- [ ] Applied changes appear in undo history for all kinds
- [ ] Command description names the assistant as origin for all kinds

## Verification
- Headless test: Create various proposal types → apply → verify command stack for each
- UI test: Review panel shows all change kinds, validation works for each
- Integration: All applied changes appear in undo history
- Edge case: Resource assignment with non-existent resource → validation error
- Edge case: Calendar with invalid pattern → validation error
- Edge case: Actual finish before start → validation error