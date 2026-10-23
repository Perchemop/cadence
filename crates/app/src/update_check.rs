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
///
/// The full API shape is fixed now so wiring the GitHub Releases API in v0.2.0
/// cannot change what callers handle; the stub only ever returns
/// [`UpdateStatus::Stub`], which is why the other variants are allowed dead
/// code until then.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stub_reports_the_version_and_makes_no_request() {
        assert!(!current_version().is_empty());
        assert_eq!(check_for_update(), UpdateStatus::Stub);
    }

    #[test]
    fn an_available_update_carries_the_new_version() {
        // The variant is the v0.2.0 API shape; the stub never produces it, but
        // the shape is fixed here so wiring the Releases API later cannot
        // change what callers must handle.
        let status = UpdateStatus::UpdateAvailable("v0.2.0");
        assert!(matches!(status, UpdateStatus::UpdateAvailable(version) if version == "v0.2.0"));
        assert_ne!(status, UpdateStatus::Stub);
    }
}
