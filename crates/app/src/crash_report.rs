//! Crash reporting for the Cadence binary.
//!
//! v0.1.0 ships the guide §6.1 *stub*: a panic hook that appends a crash
//! entry to a local log file and points the user at it. Nothing leaves the
//! machine; submitting crash reports to GitHub Issues is v0.2.x work and will
//! only ever send what the user consents to.

use std::fmt::Write as _;
use std::io::Write as _;
use std::panic::PanicHookInfo;
use std::path::PathBuf;

/// The crash log location: the shared file the GUI also reads.
fn crash_log_path() -> PathBuf {
    cadence_ui_egui::crash::log_path()
}

/// Install the panic hook. Call once, before anything that can panic.
///
/// The previous hook (if any) is chained so a test harness or embedded runner
/// keeps its own reporting.
pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info: &PanicHookInfo<'_>| {
        previous(info);
        if let Err(error) = append_entry(info) {
            eprintln!("cadence: could not write the crash log: {error}");
        }
        eprintln!(
            "cadence: crashed — details in {} (nothing was sent anywhere)",
            crash_log_path().display()
        );
    }));
}

/// Append one crash entry to the local log.
fn append_entry(info: &PanicHookInfo<'_>) -> std::io::Result<()> {
    let mut entry = String::new();
    let _ = writeln!(entry, "=== crash {} ===", timestamp());
    let _ = writeln!(entry, "version: {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(entry, "os: {}", std::env::consts::OS);
    if let Some(location) = info.location() {
        let _ = writeln!(
            entry,
            "panic at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    let message = payload_message(info);
    let _ = writeln!(entry, "panic message: {message}");
    let _ = writeln!(
        entry,
        "backtrace (RUST_BACKTRACE=1 for full):\n{}",
        std::backtrace::Backtrace::force_capture()
    );

    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(crash_log_path())?;
    log.write_all(entry.as_bytes())
}

/// The panic payload as text, when it is a string.
fn payload_message(info: &PanicHookInfo<'_>) -> String {
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

/// A wall-clock timestamp for the log entry.
fn timestamp() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown time".to_string())
}
