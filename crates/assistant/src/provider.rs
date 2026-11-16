//! The provider seam: one trait every model backend implements.
//!
//! A *provider* turns a prompt into a completion. Everything else in this crate
//! is provider-agnostic, so a later slice can add an HTTP backend (OpenAI,
//! Anthropic, Ollama, Gemini, a local binary) without touching the proposal or
//! config types. This crate deliberately ships no network code: it defines the
//! contract and a deterministic test double.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// An assistant failure, carrying enough detail for a plain-language message.
///
/// The variants never carry key material: a provider that cannot run returns
/// [`AssistantError::NotConfigured`] rather than echoing its credentials.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum AssistantError {
    /// The assistant is disabled or has no provider configured.
    #[error("the assistant is not configured")]
    NotConfigured,
    /// The provider refused the request (for example, a policy decline).
    #[error("the provider refused the request")]
    Refused,
    /// The provider returned output that could not be understood.
    #[error("the provider returned malformed output: {0}")]
    MalformedOutput(String),
    /// A transport-level failure reaching the provider.
    #[error("the provider could not be reached: {0}")]
    Network(String),
}

/// A named large-language-model backend.
///
/// Implement this trait to add a provider. Implementations must be cheap to
/// clone behind an `Arc` if shared across threads.
pub trait AssistantProvider: Send + Sync {
    /// Produce a completion for `prompt`.
    fn complete(&self, prompt: &str) -> Result<String, AssistantError>;

    /// The provider's human-readable name, for display.
    fn name(&self) -> &str;
}

/// The provider a user has selected.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssistantProviderName {
    /// OpenAI's chat completions API.
    OpenAI,
    /// Anthropic's messages API.
    Anthropic,
    /// A local Ollama server.
    Ollama,
    /// Google's Gemini API.
    Gemini,
    /// A provider not covered by the variants above.
    Custom(String),
}

/// A provider that always refuses.
///
/// Used when the feature is disabled: the UI hides the assistant, and any
/// accidental call fails closed rather than reaching the network.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopProvider;

impl AssistantProvider for NoopProvider {
    fn complete(&self, _prompt: &str) -> Result<String, AssistantError> {
        Err(AssistantError::NotConfigured)
    }

    fn name(&self) -> &str {
        "disabled"
    }
}

/// A deterministic provider for tests.
///
/// It returns a canned string and never performs I/O, so workflow tests remain
/// headless and reproducible.
#[derive(Clone, Debug)]
pub struct MockProvider {
    response: String,
}

impl MockProvider {
    /// Create a mock provider that always returns `response`.
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

impl AssistantProvider for MockProvider {
    fn complete(&self, _prompt: &str) -> Result<String, AssistantError> {
        Ok(self.response.clone())
    }

    fn name(&self) -> &str {
        "mock"
    }
}
