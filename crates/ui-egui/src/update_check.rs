//! Update checking against the public GitHub Releases API.
//!
//! The check is deliberately small and fail-soft: it asks for the latest
//! release, compares its tag with the running version, and caches a successful
//! answer for a few hours so a GUI restart does not hit the API again. Any
//! network, HTTP or parsing failure becomes [`UpdateStatus::Unavailable`] and
//! the application keeps working offline.
//!
//! No account, token or identifying header is sent. The request carries only a
//! `Cadence/<version>` user agent, which GitHub requires for any API client.

use std::cmp::Ordering;
use std::io::Read as _;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// The latest-release endpoint for the Cadence repository.
const RELEASES_API: &str = "https://api.github.com/repos/Perchemop/cadence/releases/latest";

/// The human-facing releases page, used when a release has no HTML URL.
const RELEASES_PAGE: &str = "https://github.com/Perchemop/cadence/releases/latest";

/// How long a successful answer is reused before the API is asked again.
const CACHE_TTL_SECONDS: i64 = 6 * 60 * 60;

/// Cap the response body so a hostile or broken endpoint cannot exhaust memory.
const MAX_RESPONSE_BYTES: u64 = 256 * 1024;

/// The result of an update check, as rendered by the GUI and the CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// No check has run yet and there is no fresh cache entry.
    NotChecked,
    /// The running version is the latest known release.
    UpToDate,
    /// A newer release exists; `version` is its tag without the leading `v`.
    UpdateAvailable { version: String, url: String },
    /// The check could not complete — offline, HTTP error, or a bad response.
    Unavailable(String),
}

/// The running application's version, from the crate metadata.
#[must_use]
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The cache location: the system temporary directory, next to the crash log.
#[must_use]
pub fn cache_path() -> PathBuf {
    std::env::temp_dir().join("cadence-update-check.json")
}

/// Return a cached answer when it is still fresh, otherwise `None`.
#[must_use]
pub fn cached_status() -> Option<UpdateStatus> {
    let text = std::fs::read_to_string(cache_path()).ok()?;
    let cached: CachedStatus = serde_json::from_str(&text).ok()?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    if now < cached.checked_at || now - cached.checked_at >= CACHE_TTL_SECONDS {
        return None;
    }
    cached.into_status()
}

/// Ask GitHub for the latest release and compare it with the running version.
///
/// A successful answer is cached. Failures are reported, not cached, so a
/// later attempt can recover when the network returns.
#[must_use]
pub fn check_for_update() -> UpdateStatus {
    match fetch_latest_release() {
        Ok(release) => {
            let version = strip_v(&release.tag_name);
            let status = if is_newer(version, current_version()) {
                UpdateStatus::UpdateAvailable {
                    version: version.to_string(),
                    url: release.html_url,
                }
            } else {
                UpdateStatus::UpToDate
            };
            write_cache(&status);
            status
        }
        Err(message) => UpdateStatus::Unavailable(message),
    }
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedStatus {
    checked_at: i64,
    up_to_date: bool,
    version: Option<String>,
    url: Option<String>,
}

impl CachedStatus {
    fn into_status(self) -> Option<UpdateStatus> {
        if self.up_to_date {
            Some(UpdateStatus::UpToDate)
        } else {
            let version = self.version?;
            let url = self.url.unwrap_or_else(|| RELEASES_PAGE.to_string());
            Some(UpdateStatus::UpdateAvailable { version, url })
        }
    }
}

fn fetch_latest_release() -> Result<GitHubRelease, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(4))
        .timeout_read(Duration::from_secs(4))
        .timeout_write(Duration::from_secs(4))
        .user_agent(concat!("Cadence/", env!("CARGO_PKG_VERSION")))
        .build();
    let response = agent
        .get(RELEASES_API)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| format!("update check failed: {error}"))?;
    let mut body = String::new();
    response
        .into_reader()
        .take(MAX_RESPONSE_BYTES)
        .read_to_string(&mut body)
        .map_err(|error| format!("cannot read the release response: {error}"))?;
    serde_json::from_str(&body)
        .map_err(|error| format!("the release response is not readable: {error}"))
}

fn write_cache(status: &UpdateStatus) {
    let checked_at = OffsetDateTime::now_utc().unix_timestamp();
    let cached = match status {
        UpdateStatus::UpToDate => CachedStatus {
            checked_at,
            up_to_date: true,
            version: None,
            url: None,
        },
        UpdateStatus::UpdateAvailable { version, url } => CachedStatus {
            checked_at,
            up_to_date: false,
            version: Some(version.clone()),
            url: Some(url.clone()),
        },
        UpdateStatus::NotChecked | UpdateStatus::Unavailable(_) => return,
    };
    if let Ok(text) = serde_json::to_string(&cached) {
        let _ = std::fs::write(cache_path(), text);
    }
}

fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag)
}

fn is_newer(latest: &str, current: &str) -> bool {
    let Some(latest) = Version::parse(latest) else {
        return false;
    };
    let Some(current) = Version::parse(current) else {
        return false;
    };
    latest > current
}

#[derive(Debug, PartialEq, Eq)]
struct Version {
    core: [u64; 3],
    pre: Option<String>,
}

impl Version {
    fn parse(text: &str) -> Option<Self> {
        let text = strip_v(text.trim());
        let text = text.split_once('+').map_or(text, |(core, _)| core);
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) if !pre.is_empty() => (core, Some(pre.to_string())),
            Some(_) => return None,
            None => (text, None),
        };
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            core: [major, minor, patch],
            pre,
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let core = self.core.cmp(&other.core);
        if core != Ordering::Equal {
            return core;
        }
        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(left), Some(right)) => compare_prerelease(left, right),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compare SemVer pre-release strings: numeric identifiers sort before
/// alphanumeric ones, and a shorter identifier list sorts first when the
/// shared prefix is equal.
fn compare_prerelease(left: &str, right: &str) -> Ordering {
    let mut left = left.split('.');
    let mut right = right.split('.');
    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a), Some(b)) => {
                let ordering = match (a.parse::<u64>(), b.parse::<u64>()) {
                    (Ok(a), Ok(b)) => a.cmp(&b),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => a.cmp(b),
                };
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_version_is_reported() {
        assert!(!current_version().is_empty());
    }

    #[test]
    fn newer_release_tags_are_detected() {
        assert!(is_newer("v0.2.0", "0.1.1"));
        assert!(is_newer("0.1.2", "0.1.1"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.2.0", "v0.1.1"));
    }

    #[test]
    fn older_and_equal_release_tags_are_not_updates() {
        assert!(!is_newer("0.1.1", "0.1.1"));
        assert!(!is_newer("v0.1.0", "0.1.1"));
        assert!(!is_newer("0.1.9", "0.1.10"));
    }

    #[test]
    fn a_release_beats_its_own_prerelease() {
        assert!(is_newer("0.2.0", "0.2.0-beta.1"));
        assert!(!is_newer("0.2.0-beta.1", "0.2.0"));
        assert!(is_newer("0.2.0-beta.2", "0.2.0-beta.1"));
    }

    #[test]
    fn malformed_tags_do_not_claim_an_update() {
        assert!(!is_newer("not-a-version", "0.1.1"));
        assert!(!is_newer("0.2", "0.1.1"));
        assert!(!is_newer("0.2.0.1", "0.1.1"));
        assert!(!is_newer("0.2.0-", "0.1.1"));
    }

    #[test]
    fn a_cache_entry_round_trips_through_json() {
        let cached = CachedStatus {
            checked_at: 1_700_000_000,
            up_to_date: false,
            version: Some("0.2.0".to_string()),
            url: Some("https://github.com/Perchemop/cadence/releases/tag/v0.2.0".to_string()),
        };
        let text = serde_json::to_string(&cached).expect("cache serialises");
        let parsed: CachedStatus = serde_json::from_str(&text).expect("cache parses");
        assert_eq!(
            parsed.into_status(),
            Some(UpdateStatus::UpdateAvailable {
                version: "0.2.0".to_string(),
                url: "https://github.com/Perchemop/cadence/releases/tag/v0.2.0".to_string(),
            })
        );
    }

    #[test]
    fn an_up_to_date_cache_entry_needs_no_version_or_url() {
        let cached = CachedStatus {
            checked_at: 1_700_000_000,
            up_to_date: true,
            version: None,
            url: None,
        };
        assert_eq!(cached.into_status(), Some(UpdateStatus::UpToDate));
    }
}
