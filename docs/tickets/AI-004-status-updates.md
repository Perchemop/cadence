# Ticket: AI-004 — AI Status Updates in Diary

## Objective
Implement AI-generated progress updates in diary entries with source tracking, incorporating the status update structure from the AI integration research.

## Target Files
- `crates/ui-egui/src/lib.rs` — Diary view (Ctrl+8): AI suggestion button per note, progress update dialog
- `crates/assistant/src/lib.rs` — `ProposedProgressUpdate` struct, source citation model
- `crates/commands/src/lib.rs` — `CreateProposedProgress` command, integrates with `CreateDailyNote` flow
- `crates/core/src/model.rs` — `DiaryNote` already has `lookahead_window_start`; add `ai_source: Option<String>` field

## ProposedProgressUpdate Struct (from research)
```rust
#[derive(Clone, Debug)]
pub struct ProposedProgressUpdate {
    /// Activity ID being updated
    pub activity: String,
    
    /// Percent complete (0-100)
    pub percent: u8,
    
    /// Actual start date (if provided)
    pub actual_start: Option<ChronologicalDate>,
    
    /// Actual finish date (if provided)
    pub actual_finish: Option<ChronologicalDate>,
    
    /// Delay reason (required for promised activities)
    pub delay_reason: Option<String>,
    
    /// Source tracking: which document or project data contributed
    pub ai_source: String,  // e.g., "diary-note-001", "document-pdf-003"
    
    /// Source type for tracking
    pub source_type: ProgressSourceType,
}

/// Progress source type enum
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressSourceType {
    DiaryNote,      // from existing diary entry
    Document,       // from ingested document
    ProjectData,    // from project model (logic, float, constraints)
    Manual,         // user-entered manually
}
```

## Diary Note Modification (from research)
- `DiaryNote` already has `lookahead_window_start`; add `ai_marked: bool` flag
- Clicking "AI Suggest Progress" generates a proposed progress update with:
  - percent complete, actual start/finish dates, delay reason
  - Proposal cites source document(s) or project model data
- User reviews and can accept/reject the proposal
- Accepted update creates/updates `DailyNote` with `ai_marked: bool` flag
- Applied update appears in undo history with "Assistant" origin
- Rejected update leaves note unchanged
- Source tracking: every AI progress update records which document/project data contributed

## CreateProposedProgress Command (from research)
- Integrates with `CreateDailyNote` flow
- Generates proposed progress based on available data sources
- Returns structured proposal for user review
- Validation: percent outside 0-100% shown as review error
- Finish before start rejected
- Unknown activity ID rejected

## Blocking Dependencies
- AI-001 (provider config), AI-002 (proposal pipeline)

## Acceptance Criteria
- [ ] Diary note dialog has "AI Suggest Progress" button
- [ ] Clicking generates a proposed progress update with: percent complete, actual start/finish dates, delay reason
- [ ] Proposal cites source document(s) or project model data
- [ ] User reviews and can accept/reject the proposal
- [ ] Accepted update creates/updates `DailyNote` with `ai_marked: bool` flag
- [ ] Applied update appears in undo history with "Assistant" origin
- [ ] Rejected update leaves note unchanged
- [ ] Source tracking: every AI progress update records which document/project data contributed
- [ ] Button disabled when no provider configured, shows "enable AI" prompt
- [ ] Edge case: No data available → shows "cannot determine progress from available material"

## Verification
- Headless test: AI suggestion → accept → verify DailyNote updated with AI flag → undo restores
- UI test: Diary dialog renders, AI suggestion shows plausible values, accept/discard works
- Integration: Undo history includes assistant-origin entries
- Edge case: No provider configured → button disabled with "enable AI" prompt
- Edge case: Very large document sets → chunking, size cap, progress state
- Edge case: Scanned PDFs → "no text layer detected" rather than invented content