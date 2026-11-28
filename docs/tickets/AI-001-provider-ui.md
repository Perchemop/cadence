# Ticket: AI-001 — Provider Selection UI & Configuration Persistence

## Objective
Implement the provider selection UI in Settings and persist the selected provider configuration, incorporating all detailed configuration fields from the AI integration research.

## Target Files
- `crates/ui-egui/src/lib.rs` — Settings view: Assistant tab with provider dropdown, model selector, base URL, temperature, max tokens
- `crates/settings/src/lib.rs` — `AssistantConfig` struct with all provider fields
- `crates/commands/src/lib.rs` — `UpdateAssistantConfig` command (pattern to existing settings commands)
- `crates/core/src/model.rs` — `AssistantConfig` includes all provider selection fields

## AssistantConfig Structure (from AI integration research)
```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AssistantConfig {
    /// Whether the assistant feature is enabled
    pub enabled: bool,
    
    /// Selected provider: openai, anthropic, ollama, gemini, or custom
    pub provider: AssistantProvider,
    
    /// Model identifier (e.g., gpt-4o-mini, llama3:70b)
    pub model: String,
    
    /// Base URL for non-Ollama providers (e.g., https://api.openai.com/v1)
    /// Required for OpenAI, Anthropic, Gemini; optional for Custom
    pub base_url: Option<String>,
    
    /// Temperature control (0.0 to 2.0, default 0.7)
    pub temperature: f32,
    
    /// Maximum tokens in response (default 4096)
    pub max_tokens: u32,
    
    /// Whether the feature is active/configured
    pub active: bool,
}

/// Provider enum with all variants from research
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssistantProvider {
    OpenAI,
    Anthropic,
    Ollama,
    Gemini,
    Custom(String),  // custom server name
}
```

## Blocking Dependencies
- None (foundational)

## Acceptance Criteria
- [ ] Settings → Assistant tab shows provider dropdown with: OpenAI, Anthropic, Ollama, Gemini, Custom
- [ ] Model selector populates based on selected provider (or free-text for local/custom)
- [ ] Base URL field for Ollama/local endpoints and Custom providers
- [ ] Temperature slider (0.0 to 2.0, default 0.7) with display
- [ ] Max tokens input (positive integer, default 4096)
- [ ] API key input (hidden, stored in OS keyring via `keyring` crate, not in settings file)
- [ ] "Test Connection" button validates credentials before saving
- [ ] Configuration persists across app restarts (loaded from OS keystore)
- [ ] When no provider configured, Assistant view shows "Enable Assistant" prompt with link to Settings
- [ ] Default values: Assistant Enabled = false, Model = None, Base URL = None
- [ ] API key never written to project file or logs

## Verification
- Headless test: `UpdateAssistantConfig` command persists and restores config from OS keyring
- UI test: Settings tab renders all fields correctly, "Test Connection" works with mock provider
- Security test: API key never written to project file, logs, or git; keyring integration verified
- Default test: App starts with assistant disabled; all default values applied