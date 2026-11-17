//! HTTP model backends behind the [`AssistantProvider`] seam.
//!
//! This module is the network half of the assistant. It exists only behind an
//! explicitly configured [`AssistantConfig`] — the feature stays off until the
//! user enables it, names a provider, a model and (for hosted providers) an
//! API key, and the application only calls [`HttpProvider::complete`] when the
//! user explicitly asks for a proposal. Nothing here runs on its own.
//!
//! Four wire protocols are spoken, one per [`Wire`] variant:
//!
//! | Wire | Default endpoint | Auth |
//! |------|------------------|------|
//! | OpenAI (and OpenAI-compatible servers) | `…/v1/chat/completions` | `Authorization: Bearer` |
//! | Anthropic | `https://api.anthropic.com/v1/messages` | `x-api-key` + `anthropic-version` |
//! | Ollama (locally hosted) | `http://localhost:11434/api/chat` | none |
//! | Gemini | `…/v1beta/models/{model}:generateContent` | `x-goog-api-key` |
//!
//! The request and response shapes were written from the providers' published
//! API references (October 2026) and are covered by headless tests: request
//! building and response parsing are pure functions over data, so no test ever
//! opens a socket. The one step that needs a network — the `ureq` call itself
//! — is a thin, documented glue layer in [`HttpProvider::complete`].
//!
//! Errors never carry key material: transport failures map to
//! [`AssistantError::Network`] with the status code and, when the provider
//! supplies one, its own plain-language error message — nothing from the
//! request is ever echoed back.

use crate::config::{AssistantConfig, SecretKey};
use crate::provider::{AssistantError, AssistantProvider};
use serde_json::json;
use std::time::Duration;

/// How long connecting to the provider may take.
///
/// Short, like the update check: an unreachable server should fail fast rather
/// than hang the caller.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// How long a completion may take to arrive.
///
/// Generation is genuinely slow — a long prompt can legitimately take a
/// minute — so this is far more generous than the connect timeout.
const READ_TIMEOUT: Duration = Duration::from_secs(120);

/// How long writing the request may take.
const WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// The `anthropic-version` header value the Messages API requires.
///
/// Pinned to the version current at implementation time; the API reference
/// documents the header as required, and older servers still answer, so it
/// does not need to be configurable.
const ANTHROPIC_VERSION: &str = "2026-10-21";

/// The completion budget sent to providers that require one (Anthropic).
///
/// The Messages API rejects a request without `max_tokens`; four thousand
/// output tokens are ample for the proposal documents this assistant drafts.
const MAX_TOKENS: u32 = 4_000;

/// The wire protocol a backend speaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wire {
    /// OpenAI's chat-completions protocol.
    OpenAI,
    /// The same protocol on a user-named server (LM Studio, vLLM, OpenRouter,
    /// a self-hosted gateway…). Requires an explicit `base_url`.
    OpenAICompatible,
    /// Anthropic's Messages protocol.
    Anthropic,
    /// Ollama's native chat protocol on a locally hosted server.
    Ollama,
    /// Google's Gemini `generateContent` protocol.
    Gemini,
}

impl Wire {
    /// Map a configured provider name onto a wire.
    ///
    /// The four known names map to their own protocols; any other non-empty
    /// name is treated as an OpenAI-compatible server (the de-facto
    /// compatibility standard), which is why
    /// [`AssistantConfig::is_configured`] requires a server address for it.
    /// An empty name configures nothing.
    fn from_provider_name(name: &str) -> Option<Wire> {
        match name.trim().to_ascii_lowercase().as_str() {
            "openai" => Some(Wire::OpenAI),
            "anthropic" => Some(Wire::Anthropic),
            "ollama" => Some(Wire::Ollama),
            "gemini" => Some(Wire::Gemini),
            "" => None,
            _ => Some(Wire::OpenAICompatible),
        }
    }

    /// The endpoint used when the configuration names no server.
    fn default_base_url(self) -> &'static str {
        match self {
            Wire::OpenAI | Wire::OpenAICompatible => "https://api.openai.com/v1",
            Wire::Anthropic => "https://api.anthropic.com",
            Wire::Ollama => "http://localhost:11434",
            Wire::Gemini => "https://generativelanguage.googleapis.com/v1beta",
        }
    }

    /// The provider label shown in the UI.
    fn label(self) -> &'static str {
        match self {
            Wire::OpenAI => "openai",
            Wire::OpenAICompatible => "openai-compatible",
            Wire::Anthropic => "anthropic",
            Wire::Ollama => "ollama",
            Wire::Gemini => "gemini",
        }
    }
}

/// A request built and ready to be sent on the wire.
///
/// Pure data: the tests assert on these fields directly, so request shaping is
/// verified without any socket.
#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpRequest {
    /// The HTTP method (always POST today).
    method: &'static str,
    /// The absolute URL to call.
    url: String,
    /// Header name/value pairs in send order.
    headers: Vec<(&'static str, String)>,
    /// The serialised JSON body.
    body: String,
}

/// Build the wire request for one prompt.
///
/// `base_url` is already resolved (the configuration's override or the wire's
/// default); `key` is absent only for keyless servers such as a local Ollama.
/// The prompt is embedded with `serde_json`, so quoting is handled by the
/// serialiser, and the key never appears in the body or URL — only in its
/// authentication header.
fn build_request(
    wire: Wire,
    base_url: &str,
    model: &str,
    key: Option<&SecretKey>,
    prompt: &str,
) -> HttpRequest {
    let base = base_url.trim().trim_end_matches('/');
    let serialise = |value: serde_json::Value| value.to_string();
    let user_message = json!([{"role": "user", "content": prompt}]);
    let (url, body) = match wire {
        Wire::OpenAI | Wire::OpenAICompatible => (
            format!("{base}/chat/completions"),
            serialise(json!({
                "model": model,
                "messages": user_message,
            })),
        ),
        Wire::Anthropic => (
            format!("{base}/v1/messages"),
            serialise(json!({
                "model": model,
                "max_tokens": MAX_TOKENS,
                "messages": user_message,
            })),
        ),
        Wire::Ollama => (
            format!("{base}/api/chat"),
            serialise(json!({
                "model": model,
                "messages": user_message,
                "stream": false,
            })),
        ),
        Wire::Gemini => (
            format!("{base}/models/{model}:generateContent"),
            serialise(json!({
                "contents": [{"parts": [{"text": prompt}]}],
            })),
        ),
    };
    let mut headers = auth_headers(wire, key);
    // Every protocol body is JSON; the content-type header is shared.
    headers.push(("content-type", "application/json".to_owned()));
    HttpRequest {
        method: "POST",
        url,
        headers,
        body,
    }
}

/// The authentication headers for a wire, omitting every credential header
/// when no key is set.
///
/// A keyless server — a local Ollama, or an OpenAI-compatible server without
/// a key — must never carry an empty credential header such as
/// `x-api-key: `; the header is simply absent.
fn auth_headers(wire: Wire, key: Option<&SecretKey>) -> Vec<(&'static str, String)> {
    let key = match key {
        Some(key) => key.as_str().to_owned(),
        None => return Vec::new(),
    };
    match wire {
        Wire::OpenAI | Wire::OpenAICompatible => vec![("authorization", format!("Bearer {key}"))],
        Wire::Anthropic => vec![
            ("x-api-key", key),
            ("anthropic-version", ANTHROPIC_VERSION.to_owned()),
        ],
        Wire::Ollama => Vec::new(),
        Wire::Gemini => vec![("x-goog-api-key", key)],
    }
}

/// Extract the completion text from a successful provider response.
///
/// Each protocol nests its text differently; every failure to find text is an
/// [`AssistantError::MalformedOutput`] with a plain-language reason rather
/// than a parse dump.
fn parse_response(wire: Wire, body: &str) -> Result<String, AssistantError> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|_| {
        AssistantError::MalformedOutput("the provider's response was not valid JSON".to_owned())
    })?;
    let text: Option<String> = match wire {
        Wire::OpenAI | Wire::OpenAICompatible => value
            .pointer("/choices/0/message/content")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        Wire::Anthropic => text_blocks(&value, "/content"),
        Wire::Ollama => value
            .pointer("/message/content")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        Wire::Gemini => text_blocks(&value, "/candidates/0/content/parts"),
    };
    match text {
        Some(text) if !text.trim().is_empty() => Ok(text),
        _ => Err(AssistantError::MalformedOutput(
            "the response carried no completion text".to_owned(),
        )),
    }
}

/// Join the `text` fields of a response array (Anthropic content blocks,
/// Gemini parts), separating blocks with newlines.
///
/// Blocks without text (tool calls, thinking blocks) are skipped; only an
/// array with no text at all yields `None`.
fn text_blocks(value: &serde_json::Value, pointer: &str) -> Option<String> {
    let blocks = value.pointer(pointer)?.as_array()?;
    let parts: Vec<&str> = blocks
        .iter()
        .filter_map(|block| block.get("text").and_then(serde_json::Value::as_str))
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Map a non-success HTTP status to the plain-language assistant error.
///
/// Authentication and bad-request failures are [`AssistantError::Refused`]
/// (a configuration problem the user fixes); rate limits, timeouts and server
/// faults are [`AssistantError::Network`] (worth retrying). When the provider
/// supplies its own JSON error message, a short version of it becomes the
/// network detail; the raw body is never copied wholesale.
fn failure(status: u16, body: &str) -> AssistantError {
    match status {
        401 | 403 => AssistantError::Refused,
        408 | 429 | 500..=599 => AssistantError::Network(match error_detail(body) {
            Some(detail) => format!("status {status}: {detail}"),
            None => format!("status {status}"),
        }),
        // Other 4xx codes (bad request, unknown model, wrong path) are the
        // configuration's fault, not a transient network state.
        _ => AssistantError::Refused,
    }
}

/// The provider's own short error message, when the body is a JSON error.
///
/// Covers both `{"error": {"message": …}}` (OpenAI, Anthropic, Gemini) and
/// `{"error": "…"}` (Ollama). Anything else yields `None`, and the detail is
/// truncated so a chatty provider cannot flood a status line.
fn error_detail(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let message = value
        .pointer("/error/message")
        .and_then(serde_json::Value::as_str)
        .or_else(|| value.get("error").and_then(serde_json::Value::as_str))?;
    const MAX_DETAIL_CHARS: usize = 200;
    let truncated: String = message.chars().take(MAX_DETAIL_CHARS).collect();
    let detail = if message.chars().count() > MAX_DETAIL_CHARS {
        format!("{truncated}…")
    } else {
        truncated
    };
    Some(detail)
}

/// A model backend reached over HTTP, built from an [`AssistantConfig`].
///
/// Construct it with [`HttpProvider::from_config`]; an unconfigured or
/// partially configured assistant fails closed with
/// [`AssistantError::NotConfigured`] rather than reaching the network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpProvider {
    wire: Wire,
    base_url: String,
    model: String,
    key: Option<SecretKey>,
}

impl HttpProvider {
    /// Build the provider a configuration describes.
    ///
    /// Fails with [`AssistantError::NotConfigured`] when anything needed is
    /// missing — the feature disabled, no provider, no model, a missing key on
    /// a keyed provider, a key with control characters (which could never form
    /// a valid header value), or a custom server with no address.
    pub fn from_config(config: &AssistantConfig) -> Result<Self, AssistantError> {
        if !config.is_configured() {
            return Err(AssistantError::NotConfigured);
        }
        if let Some(key) = &config.api_key {
            let has_control_char = key.as_str().chars().any(char::is_control);
            if has_control_char {
                return Err(AssistantError::NotConfigured);
            }
        }
        let name = config.provider_name.as_deref().unwrap_or_default();
        let wire = Wire::from_provider_name(name).ok_or(AssistantError::NotConfigured)?;
        let base_url = config
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| wire.default_base_url().to_owned());
        Ok(Self {
            wire,
            base_url,
            model: config.model.clone().unwrap_or_default(),
            key: config.api_key.clone(),
        })
    }
}

impl AssistantProvider for HttpProvider {
    fn name(&self) -> &str {
        self.wire.label()
    }

    fn complete(&self, prompt: &str) -> Result<String, AssistantError> {
        let request = build_request(
            self.wire,
            &self.base_url,
            &self.model,
            self.key.as_ref(),
            prompt,
        );
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(CONNECT_TIMEOUT)
            .timeout_read(READ_TIMEOUT)
            .timeout_write(WRITE_TIMEOUT)
            .user_agent(concat!("Cadence/", env!("CARGO_PKG_VERSION")))
            .build();
        let mut call = agent.request(request.method, &request.url);
        for (name, value) in &request.headers {
            call = call.set(name, value);
        }
        match call.send_string(&request.body) {
            Ok(response) => {
                let body = response.into_string().map_err(|_| {
                    AssistantError::Network("could not read the provider's response".to_owned())
                })?;
                parse_response(self.wire, &body)
            }
            Err(ureq::Error::Status(status, response)) => {
                let body = response.into_string().unwrap_or_default();
                Err(failure(status, &body))
            }
            // Transport-level failures (DNS, refused connection, timeout,
            // bad URL): a fixed plain-language phrase. The underlying error
            // text can echo request context such as the endpoint URL, so it
            // is deliberately not copied into the error.
            Err(_) => Err(AssistantError::Network(
                "connection or address problem".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "not-a-real-credential";
    const PROMPT: &str = "Draft one activity for POURING-CONCRETE-L3.";

    fn key() -> SecretKey {
        SecretKey::new(KEY)
    }

    fn request_for(wire: Wire, base: Option<&str>, key: Option<SecretKey>) -> HttpRequest {
        build_request(
            wire,
            base.unwrap_or_else(|| wire.default_base_url()),
            "test-model",
            key.as_ref(),
            PROMPT,
        )
    }

    /// Whether the request carries exactly this header value.
    fn carries(request: &HttpRequest, name: &str, value: &str) -> bool {
        request
            .headers
            .iter()
            .any(|(header_name, header_value)| *header_name == name && header_value == value)
    }

    #[test]
    fn openai_request_carries_bearer_key_model_and_prompt() {
        let request = request_for(Wire::OpenAI, None, Some(key()));
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://api.openai.com/v1/chat/completions");
        let bearer = format!("Bearer {KEY}");
        assert!(carries(&request, "authorization", &bearer));
        assert!(request.body.contains("test-model"));
        assert!(request.body.contains(PROMPT));
        assert!(!request.body.contains(KEY));
    }

    #[test]
    fn anthropic_request_uses_api_key_header_and_pinned_version() {
        let request = request_for(Wire::Anthropic, None, Some(key()));
        assert_eq!(request.url, "https://api.anthropic.com/v1/messages");
        assert!(carries(&request, "x-api-key", KEY));
        assert!(carries(&request, "anthropic-version", ANTHROPIC_VERSION));
        assert!(request.body.contains("max_tokens"));
        assert!(!request.body.contains(KEY));
        assert!(!request.url.contains(KEY));
    }

    #[test]
    fn ollama_request_omits_auth_and_requests_a_single_response() {
        let request = request_for(Wire::Ollama, None, None);
        assert_eq!(request.url, "http://localhost:11434/api/chat");
        assert!(!request
            .headers
            .iter()
            .any(|(name, _)| *name == "authorization"));
        assert!(request.body.contains("\"stream\":false"));
        assert!(request.body.contains(PROMPT));
    }

    #[test]
    fn gemini_request_puts_model_in_url_and_key_in_header() {
        let request = request_for(Wire::Gemini, None, Some(key()));
        assert_eq!(
            request.url,
            "https://generativelanguage.googleapis.com/v1beta/models/test-model:generateContent"
        );
        assert!(carries(&request, "x-goog-api-key", KEY));
        assert!(request.body.contains(PROMPT));
        assert!(!request.body.contains(KEY));
        assert!(!request.url.contains(KEY));
    }

    #[test]
    fn base_url_override_replaces_the_default_and_tolerates_a_trailing_slash() {
        let request = request_for(Wire::Ollama, Some("http://box.local:11434/"), None);
        assert_eq!(request.url, "http://box.local:11434/api/chat");
    }

    #[test]
    fn an_openai_compatible_server_uses_the_chat_completions_path() {
        let request = request_for(
            Wire::OpenAICompatible,
            Some("http://lm-studio.local:1234/v1"),
            None,
        );
        assert_eq!(
            request.url,
            "http://lm-studio.local:1234/v1/chat/completions"
        );
    }

    #[test]
    fn the_key_never_leaves_the_auth_header_on_any_wire() {
        let wires = [
            Wire::OpenAI,
            Wire::OpenAICompatible,
            Wire::Anthropic,
            Wire::Gemini,
        ];
        for wire in wires {
            let request = request_for(wire, None, Some(key()));
            assert!(!request.body.contains(KEY), "{wire:?} body carried the key");
            assert!(!request.url.contains(KEY), "{wire:?} url carried the key");
            for (name, value) in &request.headers {
                let is_auth_header =
                    matches!(*name, "authorization" | "x-api-key" | "x-goog-api-key");
                assert!(
                    is_auth_header || !value.contains(KEY),
                    "{wire:?} carried the key in header {name}"
                );
            }
        }
    }

    #[test]
    fn parse_response_openai_extracts_the_message_content() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"Hello planner."}}]}"#;
        assert_eq!(
            parse_response(Wire::OpenAI, body),
            Ok("Hello planner.".to_owned())
        );
    }

    #[test]
    fn parse_response_anthropic_joins_its_text_blocks() {
        let body = r#"{"content":[
            {"type":"text","text":"First block."},
            {"type":"text","text":"Second block."}]}"#;
        assert_eq!(
            parse_response(Wire::Anthropic, body),
            Ok("First block.\nSecond block.".to_owned())
        );
    }

    #[test]
    fn parse_response_ollama_extracts_the_message_content() {
        let body = r#"{"message":{"role":"assistant","content":"Poured."},"done":true}"#;
        assert_eq!(parse_response(Wire::Ollama, body), Ok("Poured.".to_owned()));
    }

    #[test]
    fn parse_response_gemini_joins_its_parts() {
        let body = r#"{"candidates":[{"content":{"parts":[
            {"text":"Part one."},{"text":"Part two."}],"role":"model"}}]}"#;
        assert_eq!(
            parse_response(Wire::Gemini, body),
            Ok("Part one.\nPart two.".to_owned())
        );
    }

    #[test]
    fn non_text_blocks_are_skipped_when_text_exists() {
        let body = r#"{"content":[
            {"type":"text","text":"Answer."},
            {"type":"tool_use","id":"x"}]}"#;
        assert_eq!(
            parse_response(Wire::Anthropic, body),
            Ok("Answer.".to_owned())
        );
    }

    #[test]
    fn parse_response_rejects_non_json_as_malformed() {
        assert_eq!(
            parse_response(Wire::OpenAI, "<html>gateway timeout</html>"),
            Err(AssistantError::MalformedOutput(
                "the provider's response was not valid JSON".to_owned()
            ))
        );
    }

    #[test]
    fn parse_response_rejects_a_body_without_text_as_malformed() {
        let body = r#"{"choices":[]}"#;
        assert_eq!(
            parse_response(Wire::OpenAI, body),
            Err(AssistantError::MalformedOutput(
                "the response carried no completion text".to_owned()
            ))
        );
        let blank = r#"{"content":[{"type":"text","text":"  "}]}"#;
        assert!(matches!(
            parse_response(Wire::Anthropic, blank),
            Err(AssistantError::MalformedOutput(_))
        ));
    }

    #[test]
    fn authentication_failures_map_to_refused() {
        assert_eq!(failure(401, "{}"), AssistantError::Refused);
        assert_eq!(failure(403, "{}"), AssistantError::Refused);
    }

    #[test]
    fn transient_statuses_map_to_network_with_the_provider_message() {
        let body = r#"{"error":{"message":"Rate limit reached."}}"#;
        assert_eq!(
            failure(429, body),
            AssistantError::Network("status 429: Rate limit reached.".to_owned())
        );
        assert_eq!(
            failure(503, "{}"),
            AssistantError::Network("status 503".to_owned())
        );
    }

    #[test]
    fn other_client_errors_map_to_refused_without_detail() {
        let body = r#"{"error":{"message":"no such model"}}"#;
        assert_eq!(failure(400, body), AssistantError::Refused);
        assert_eq!(failure(404, body), AssistantError::Refused);
        assert_eq!(failure(422, body), AssistantError::Refused);
        // The configuration-problem class stays payload-less: the provider's
        // message is reserved for the retryable network detail.
        assert!(matches!(failure(400, body), AssistantError::Refused));
    }

    #[test]
    fn keyless_requests_carry_no_credential_headers_at_all() {
        for wire in [
            Wire::OpenAI,
            Wire::OpenAICompatible,
            Wire::Anthropic,
            Wire::Gemini,
            Wire::Ollama,
        ] {
            let request = request_for(wire, None, None);
            for (name, _) in &request.headers {
                assert!(
                    !matches!(*name, "authorization" | "x-api-key" | "x-goog-api-key"),
                    "{wire:?} emitted credential header {name} without a key"
                );
            }
        }
    }

    #[test]
    fn the_failure_detail_is_truncated_and_never_the_raw_body() {
        let long = format!(r#"{{"error":{{"message":"{}"}}}}"#, "x".repeat(500));
        match failure(500, &long) {
            AssistantError::Network(detail) => {
                assert!(detail.starts_with("status 500: "));
                assert!(detail.chars().count() < 220);
            }
            other => panic!("expected a network error, got {other:?}"),
        }
        // A non-JSON body yields no detail at all.
        assert_eq!(
            failure(500, "<html>oops</html>"),
            AssistantError::Network("status 500".to_owned())
        );
    }

    #[test]
    fn ollamas_flat_error_shape_is_understood() {
        let body = r#"{"error":"model 'test-model' not found"}"#;
        assert_eq!(
            failure(500, body),
            AssistantError::Network("status 500: model 'test-model' not found".to_owned())
        );
    }

    #[test]
    fn from_config_fails_closed_when_not_configured() {
        let error = HttpProvider::from_config(&AssistantConfig::default());
        assert_eq!(error, Err(AssistantError::NotConfigured));
    }

    #[test]
    fn from_config_builds_the_named_wire() {
        let config = AssistantConfig {
            enabled: true,
            provider_name: Some("Anthropic".to_owned()),
            api_key: Some(key()),
            model: Some("test-model".to_owned()),
            base_url: None,
        };
        let provider = HttpProvider::from_config(&config).expect("configured");
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn a_key_with_control_characters_fails_closed() {
        let config = AssistantConfig {
            enabled: true,
            provider_name: Some("openai".to_owned()),
            api_key: Some(SecretKey::new("bad\nkey")),
            model: Some("test-model".to_owned()),
            base_url: None,
        };
        let error = HttpProvider::from_config(&config);
        assert_eq!(error, Err(AssistantError::NotConfigured));
    }

    #[test]
    fn an_unknown_provider_is_openai_compatible_and_needs_its_own_address() {
        let without_address = AssistantConfig {
            enabled: true,
            provider_name: Some("my-own-server".to_owned()),
            api_key: Some(key()),
            model: Some("test-model".to_owned()),
            base_url: None,
        };
        assert!(!without_address.is_configured());
        let error = HttpProvider::from_config(&without_address);
        assert_eq!(error, Err(AssistantError::NotConfigured));

        let addressed = AssistantConfig {
            base_url: Some("http://lm-studio.local:1234/v1".to_owned()),
            ..without_address
        };
        assert!(addressed.is_configured());
        let provider = HttpProvider::from_config(&addressed).expect("configured");
        assert_eq!(provider.name(), "openai-compatible");
    }

    #[test]
    fn from_config_uses_the_configured_base_url_over_the_default() {
        let config = AssistantConfig {
            enabled: true,
            provider_name: Some("ollama".to_owned()),
            api_key: None,
            model: Some("test-model".to_owned()),
            base_url: Some("http://box.local:11434".to_owned()),
        };
        let provider = HttpProvider::from_config(&config).expect("configured");
        assert_eq!(provider.base_url, "http://box.local:11434");
    }
}
