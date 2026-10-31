//! Crash-report helpers shared by the GUI.
//!
//! The panic hook in the application crate writes a local crash log and sends
//! nothing anywhere. This module reads that log when the GUI starts, builds
//! the preview the user must approve, and turns the preview into a pre-filled
//! GitHub "new issue" URL. Opening the URL is the only network-adjacent step,
//! and it happens in the user's browser after they press the button; no token
//! is embedded in the binary and no request is made by Cadence itself.
//!
//! The repository URL is deliberately a constant: the application is a public
//! project and the issue tracker is the documented place to report a crash.

use std::path::PathBuf;

/// The repository where crash reports go.
const ISSUES_URL: &str = "https://github.com/Perchemop/cadence/issues/new";

/// The crash log location: the same file the application's panic hook writes.
#[must_use]
pub fn log_path() -> PathBuf {
    std::env::temp_dir().join("cadence-crash.log")
}

/// Read the crash log, if it exists.
#[must_use]
pub fn read_log() -> Option<String> {
    std::fs::read_to_string(log_path()).ok()
}

/// Remove the crash log after the user has acted on it.
pub fn clear_log() {
    let _ = std::fs::remove_file(log_path());
}

/// Build the issue title from the log, falling back to a generic title.
#[must_use]
pub fn issue_title(log: &str) -> String {
    if log.trim().is_empty() {
        return "Bug report: Cadence".to_string();
    }
    let Some(message) = log
        .lines()
        .find_map(|line| line.strip_prefix("panic message: "))
    else {
        return "Cadence crashed".to_string();
    };
    let message = message.lines().next().unwrap_or(message).trim();
    if message.is_empty() {
        "Cadence crashed".to_string()
    } else {
        format!("Crash report: {message}")
    }
}

/// Build the issue body preview shown to the user before anything is sent.
///
/// The log already contains the version, OS, panic location, panic message
/// and a backtrace. `notes` is whatever the user typed in the dialog.
#[must_use]
pub fn issue_body(log: &str, notes: &str) -> String {
    let notes = notes.trim();
    let log = log.trim_end();
    if log.is_empty() {
        if notes.is_empty() {
            "## Bug report\n\nNo information was sent automatically.\n".to_string()
        } else {
            format!("## What happened\n\n{notes}\n\nNo information was sent automatically.\n")
        }
    } else if notes.is_empty() {
        format!(
            "## Crash report\n\nCadence wrote this entry to its local crash log:\n\n```\n{log}\n```\n\nNo information was sent automatically.\n"
        )
    } else {
        format!(
            "## What happened\n\n{notes}\n\n## Crash log\n\n```\n{log}\n```\n\nNo information was sent automatically.\n"
        )
    }
}

/// A pre-filled GitHub new-issue URL for the crash report.
#[must_use]
pub fn issue_url(log: &str, notes: &str) -> String {
    format!(
        "{ISSUES_URL}?title={}&body={}",
        encode(&issue_title(log)),
        encode(&issue_body(log, notes))
    )
}

/// Percent-encode a query-string component.
///
/// Only the unreserved characters from RFC 3986 are left alone. This is small
/// enough to keep in-crate, and avoids pulling a URL-encoding dependency into
/// the build for one dialog.
fn encode(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                output.push(byte as char);
            }
            _ => {
                output.push('%');
                output.push(hex(byte >> 4));
                output.push(hex(byte & 0x0F));
            }
        }
    }
    output
}

fn hex(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        _ => char::from(b'A' + nibble - 10),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
=== crash 2026-10-26T00:00:00Z ===
version: 0.1.1
os: linux
panic at src/main.rs:1:1
panic message: index out of bounds
backtrace (RUST_BACKTRACE=1 for full):
   0: cadence
";

    #[test]
    fn title_uses_the_panic_message() {
        assert_eq!(issue_title(SAMPLE), "Crash report: index out of bounds");
    }

    #[test]
    fn title_falls_back_when_the_log_has_no_message() {
        assert_eq!(issue_title("version: 0.1.1\n"), "Cadence crashed");
    }

    #[test]
    fn body_contains_the_log_and_notes() {
        let body = issue_body(SAMPLE, "Clicked Print Gantt");
        assert!(body.contains("Clicked Print Gantt"));
        assert!(body.contains("index out of bounds"));
        assert!(body.contains("No information was sent automatically"));
    }

    #[test]
    fn body_without_notes_is_still_usable() {
        let body = issue_body(SAMPLE, "   ");
        assert!(body.contains("== crash 2026-10-26T00:00:00Z =="));
        assert!(!body.contains("What happened"));
    }

    #[test]
    fn issue_url_percent_encodes_the_title_and_body() {
        let url = issue_url(SAMPLE, "line 1\nline 2");
        assert!(url.starts_with(ISSUES_URL));
        assert!(url.contains("title=Crash%20report%3A%20index%20out%20of%20bounds"));
        assert!(url.contains("%0A"));
        assert!(!url.contains('\n'));
        assert!(!url.contains(' '));
    }

    #[test]
    fn unreserved_characters_are_not_encoded() {
        assert_eq!(encode("Aa0-._~"), "Aa0-._~");
        assert_eq!(encode("a b"), "a%20b");
    }
}
