//! Assistant settings and secret handling.
//!
//! The assistant is **off by default**: until the user enables the feature and
//! configures a provider, nothing here can start a network call and the UI is
//! expected to hide the feature entirely. The API key never appears in `Debug`
//! output or in serialised data.

use serde::{Deserialize, Serialize};

/// An API key that redacts itself in `Debug` and serialised output.
///
/// * `Debug` prints `SecretKey("**redacted**")`.
/// * `Serialize` writes the literal string `"<redacted>"`, so a settings file,
///   a log line or a crash report can never carry the key.
/// * `Deserialize` accepts any string. A key read back from serialised data is
///   therefore only a placeholder; the real key must be loaded from the OS
///   keystore or a user-managed file instead.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct SecretKey(String);

impl SecretKey {
    /// Wrap a raw key value.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Borrow the raw key.
    ///
    /// Callers must not log or serialise the returned value; the whole point of
    /// this type is that the key only ever leaves it for an explicit API call.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never leak the key; always show the redacted view.
        formatter.write_str("SecretKey(\"**redacted**\")")
    }
}

impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Always serialise as the literal placeholder.
        serializer.serialize_str("<redacted>")
    }
}

impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self(value))
    }
}

/// The optional assistant configuration.
///
/// The default is **off**: `enabled` is `false`, so [`AssistantConfig::is_configured`]
/// returns `false` regardless of the other fields. The UI must hide the
/// assistant feature until the user explicitly enables it and configures a
/// provider and an API key.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AssistantConfig {
    /// Whether the assistant feature is active at all.
    pub enabled: bool,
    /// The configured provider, if any.
    pub provider_name: Option<String>,
    /// The configured API key, if any.
    pub api_key: Option<SecretKey>,
}

impl AssistantConfig {
    /// Whether the assistant is enabled **and** fully configured.
    ///
    /// The UI must hide the assistant feature when this returns `false`.
    pub fn is_configured(&self) -> bool {
        self.enabled && self.provider_name.is_some() && self.api_key.is_some()
    }
}
