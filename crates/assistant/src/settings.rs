//! Assistant settings storage: one small file, off by default, key-less.
//!
//! The settings survive a restart so the user does not re-enter the provider
//! and model every session. The API key does **not** survive: [`SecretKey`]
//! serialises as the literal `"<redacted>"`, so a settings file cannot carry
//! a key — and this module honours that invariant rather than working around
//! it. The key is session-only by design (spec Q4 option (a)); the store
//! step drops it explicitly.
//!
//! The file lives in the platform configuration directory (the XDG config
//! directory or `HOME` on Linux, `APPDATA` on Windows, `HOME/Library/
//! Application Support` on macOS), inside a `cadence` subdirectory. The
//! lookup is hand-rolled — no new dependency. Loading never fails the
//! caller: a missing, unreadable or unparseable file yields the default
//! (off) configuration.

use std::path::{Path, PathBuf};

use crate::config::AssistantConfig;

/// The user-level settings file path, or `None` when the environment
/// provides no configuration directory.
pub fn settings_path() -> Option<PathBuf> {
    // Red-phase stub: `config_dir` always yields `None` until the green
    // phase, so the path is always `None`.
    config_dir(|name| std::env::var(name).ok())
        .map(|dir| dir.join("cadence").join("assistant-settings.json"))
}

/// Load the assistant settings; any missing file, read failure or parse
/// failure yields the default (off) configuration. Never fails the caller.
pub fn load_settings() -> AssistantConfig {
    // Red-phase stub: `load_settings_from` always yields the default.
    settings_path()
        .map(|path| load_settings_from(&path))
        .unwrap_or_default()
}

/// Store the assistant settings, **without the API key**.
///
/// The key is session-only: a stored config's `api_key` is written as
/// `None`, so the file provably cannot round-trip a key. The directory is
/// created when missing. Returns a plain-language error string for the
/// status bar on failure.
pub fn store_settings(config: &AssistantConfig) -> Result<(), String> {
    // Red-phase stub: `store_settings_at` always refuses until the green
    // phase.
    match settings_path() {
        Some(path) => store_settings_at(&path, config),
        None => Err("no settings directory is available".to_owned()),
    }
}

/// Resolve the platform configuration directory from an environment lookup,
/// so path rules are testable without touching the machine's real home.
fn config_dir(_lookup: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    // Red-phase stub: replaced by the real lookup in the green phase.
    None
}

/// Load and parse one settings file; any failure yields the default (off)
/// configuration.
fn load_settings_from(_path: &Path) -> AssistantConfig {
    // Red-phase stub: replaced by the real loader in the green phase.
    AssistantConfig::default()
}

/// Serialise and write one settings file (creating directories), with the
/// key removed.
fn store_settings_at(_path: &Path, _config: &AssistantConfig) -> Result<(), String> {
    // Red-phase stub: replaced by the real store step in the green phase.
    Err("not implemented yet".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecretKey;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A unique temporary directory for one test, hand-rolled so no new
    /// dev-dependency is needed. Every test writes only inside its own
    /// directory and removes it afterwards.
    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after the epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "cadence-assistant-test-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create the test directory");
        dir
    }

    fn config_with_key() -> AssistantConfig {
        AssistantConfig {
            enabled: true,
            provider_name: Some("openai".to_owned()),
            api_key: Some(SecretKey::new("sk-not-a-real-credential")),
            model: Some("test-model".to_owned()),
            base_url: Some("http://lm-studio.local:1234/v1".to_owned()),
        }
    }

    #[test]
    fn config_dir_uses_the_injected_environment_values() {
        let mut env = HashMap::new();
        let expected: PathBuf;
        if cfg!(target_os = "windows") {
            env.insert("APPDATA".to_owned(), String::from("C:\\AppData\\Roaming"));
            expected = PathBuf::from("C:\\AppData\\Roaming");
        } else if cfg!(target_os = "macos") {
            env.insert("HOME".to_owned(), String::from("/Users/planner"));
            expected = PathBuf::from("/Users/planner/Library/Application Support");
        } else {
            env.insert("XDG_CONFIG_HOME".to_owned(), String::from("/xdg/config"));
            expected = PathBuf::from("/xdg/config");
        };
        let lookup = move |name: &str| env.get(name).cloned();
        assert_eq!(config_dir(lookup), Some(expected));
    }

    #[test]
    fn config_dir_yields_none_without_an_environment_directory() {
        let lookup = |_name: &str| None::<String>;
        assert_eq!(config_dir(lookup), None);
    }

    #[test]
    fn an_empty_xdg_config_home_falls_back_to_home() {
        if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
            // XDG_CONFIG_HOME is not consulted on these platforms.
            return;
        }
        let mut env = HashMap::new();
        env.insert("XDG_CONFIG_HOME".to_owned(), String::from("   "));
        env.insert("HOME".to_owned(), String::from("/planner-home"));
        let lookup = move |name: &str| env.get(name).cloned();
        assert_eq!(config_dir(lookup), Some(PathBuf::from("/planner-home")));
    }

    #[test]
    fn store_then_load_round_trips_fields_and_drops_the_key() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("assistant-settings.json");
        let config = config_with_key();
        store_settings_at(&path, &config).expect("store succeeds");
        let loaded = load_settings_from(&path);
        assert!(loaded.enabled);
        assert_eq!(loaded.provider_name.as_deref(), Some("openai"));
        assert_eq!(loaded.model.as_deref(), Some("test-model"));
        assert_eq!(
            loaded.base_url.as_deref(),
            Some("http://lm-studio.local:1234/v1")
        );
        assert!(
            loaded.api_key.is_none(),
            "the key must never survive the file"
        );
        let raw = std::fs::read_to_string(&path).expect("read the stored file");
        assert!(
            !raw.contains("sk-not-a-real-credential"),
            "the raw file must not carry the key"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_legacy_file_with_the_redacted_placeholder_still_loads() {
        let dir = temp_dir("legacy");
        let path = dir.join("assistant-settings.json");
        std::fs::write(
            &path,
            r#"{"enabled":true,"provider_name":"openai","api_key":"<redacted>"}"#,
        )
        .expect("write the legacy file");
        let loaded = load_settings_from(&path);
        assert!(loaded.enabled);
        assert_eq!(loaded.provider_name.as_deref(), Some("openai"));
        assert_eq!(loaded.model, None);
        assert_eq!(loaded.base_url, None);
        assert!(
            !loaded.is_configured(),
            "a redacted placeholder is not a usable key"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_corrupt_file_loads_as_the_default_off_config() {
        let dir = temp_dir("corrupt");
        let path = dir.join("assistant-settings.json");
        std::fs::write(&path, "{ not json at all").expect("write the corrupt file");
        let loaded = load_settings_from(&path);
        assert!(!loaded.enabled);
        assert_eq!(loaded.provider_name, None);
        assert_eq!(loaded.api_key, None);
        assert_eq!(loaded.model, None);
        assert_eq!(loaded.base_url, None);
        assert!(!loaded.is_configured());
        // A missing file behaves the same way.
        let missing = load_settings_from(&dir.join("does-not-exist.json"));
        assert!(!missing.enabled);
        assert_eq!(missing.provider_name, None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn store_creates_missing_directories() {
        let dir = temp_dir("mkdirs");
        let path = dir.join("cadence").join("assistant-settings.json");
        store_settings_at(&path, &config_with_key()).expect("store creates the directory chain");
        assert!(path.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
