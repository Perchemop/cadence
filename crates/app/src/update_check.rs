//! Update checking for the Cadence binary.
//!
//! v0.1.0 ships the guide §6.1 *stub*: the current version is known and
//! reported, and the network check is an explicit placeholder that never
//! contacts anything. Wiring the GitHub Releases API is v0.2.x work and will
//! fail soft offline when it lands.

/// The running application's version, from the crate metadata.
#[must_use]
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The result of an update check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    /// The running version is the latest known.
    UpToDate,
    /// A newer release exists; the string is its version.
    UpdateAvailable(&'static str),
    /// The check is not wired up yet — the v0.1.0 stub.
    Stub,
}

/// Check for an update.
///
/// The stub never reaches the network and always reports [`UpdateStatus::Stub`]
/// so callers can render an honest "update checks arrive in a later release"
/// message instead of pretending to have asked.
#[must_use]
pub fn check_for_update() -> UpdateStatus {
    UpdateStatus::Stub
}
