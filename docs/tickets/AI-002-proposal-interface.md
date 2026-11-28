# Ticket: AI-002 — Proposal Submission Interface

## Objective
Build the Assistant view chat interface for submitting questions and receiving proposed changes, incorporating the complete prompt/command structure from the AI integration research.

## Target Files
- `crates/ui-egui/src/lib.rs` — Assistant view (Ctrl+0): chat history, input field, send button, proposal review panel
- `crates/assistant/src/lib.rs` — `AssistantClient` trait, `ProposedChange` enum, proposal validation logic
- `crates/commands/src/lib.rs` — `SubmitAssistantRequest` command, `ApplyProposedChanges` command
- `crates/core/src/model.rs` — `AssistantMessage`, `ProposedChange`, `ProposalReviewState`

## ProposedChange Enum (from AI integration research)
```rust
#[derive(Clone, Debug)]
pub enum ProposedChange {
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
}
```

## AssistantMessage Enum
```rust
#[derive(Clone, Debug)]
pub enum AssistantMessage {
    User(String),
    Assistant {
        prose: String,
        changes: Vec<ProposedChange>,
        citations: Vec<String>,  // activity IDs or document page references
    },
}
```

## AssistantClient Trait (from AI integration research)
```rust
pub trait AssistantClient: Send + Sync + 'static {
    /// Send a request to the configured provider
    fn send_request(
        &self,
        prompt: String,
        project_snapshot: ProjectSnapshot,  // bounded to 500 activities
    ) -> Result<AssistantResponse, AssistantError>;
    
    /// Check if assistant is configured
    fn is_configured(&self) -> bool;
    
    /// Get the current provider configuration
    fn config(&self) -> &AssistantConfig;
}
```

## AssistantResponse Structure (from research)
```rust
#[derive(Clone, Debug)]
pub struct AssistantResponse {
    /// Plain-language explanation
    pub prose: String,
    
    /// Array of proposed changes (may be empty)
    pub changes: Vec<ProposedChange>,
    
    /// Citations to activity IDs or document pages
    pub citations: Vec<String>,
    
    /// Whether the response contains valid completion text
    pub has_completion: bool,
}
```

## SubmitAssistantRequest Command (from research)
- Accepts: natural language question + current project context
- Returns: structured proposal with prose + changes + citations
- Validation: typed-command validation prevents prompt injection
- Never executes without user approval

## ApplyProposedChanges Command
- Applies selected changes through the existing command stack
- Makes changes undoable (one undo step restores pre-assistant state)
- Command history records "Assistant" as origin
- Review panel shows before/after per field with highlighting

## Blocking Dependencies
- AI-001 (provider configuration)

## Acceptance Criteria
- [ ] Assistant view shows chat history with user/assistant messages
- [ ] Input field accepts natural language questions (placeholder: "Type your question or request")
- [ ] Send button dispatches `SubmitAssistantRequest` with current project context
- [ ] Response renders as: prose answer + list of proposed changes (if any)
- [ ] Each proposed change shows: command type, target activity/field, before/after values
- [ ] "Apply All" / "Apply Selected" / "Discard" buttons for proposals
- [ ] Applying goes through `ApplyProposedChanges` → command stack → undoable
- [ ] Applied changes appear in undo history with "Assistant" origin
- [ ] Error handling: malformed proposals shown as review errors, not crashes
- [ ] "Not found in material" response when no citations available
- [ ] Provider disabled UI when no configuration present

## Verification
- Headless test: Submit request → get structured proposals → apply → verify command stack
- UI test: Chat renders, proposals show before/after, apply/discard works
- Integration: Undo restores pre-assistant state exactly
- Edge case: No provider configured → button disabled with "enable AI" prompt
- Edge case: Large document sets → chunked processing with progress state
- Edge case: Scanned PDF → "no text layer detected" message