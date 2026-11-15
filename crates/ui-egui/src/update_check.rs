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
//!
//! Before an installer is ever handed to the operating system it is verified
//! end to end. Every release carries `SHA256SUMS.txt`, signed by the release
//! GPG key as `SHA256SUMS.txt.sig`; [`verify_update_installer`] checks that
//! signature against the release public key embedded in this module, then the
//! installer's own SHA-256 against the signed listing. Every step fails
//! closed (D30, guide §12.1).

use std::cmp::Ordering;
use std::io::Cursor;
use std::io::Read as _;
use std::path::PathBuf;
use std::time::Duration;

use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

/// The latest-release endpoint for the Cadence repository.
const RELEASES_API: &str = "https://api.github.com/repos/Perchemop/cadence/releases/latest";

/// The human-facing releases page, used when a release has no HTML URL.
const RELEASES_PAGE: &str = "https://github.com/Perchemop/cadence/releases/latest";

/// How long a successful answer is reused before the API is asked again.
const CACHE_TTL_SECONDS: i64 = 6 * 60 * 60;

/// Cap the response body so a hostile or broken endpoint cannot exhaust memory.
const MAX_RESPONSE_BYTES: u64 = 256 * 1024;

/// Release assets (checksums and their signature) sit beside the installers
/// under this base URL, one directory per tag.
const RELEASES_DOWNLOAD_BASE: &str = "https://github.com/Perchemop/cadence/releases/download/";

/// Cap an installer download so a hostile or broken endpoint cannot exhaust
/// memory; today's installers are tens of megabytes.
const MAX_INSTALLER_BYTES: u64 = 256 * 1024 * 1024;

/// Network budget for the checksums, their signature and the installer itself.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30);

/// The armoured OpenPGP public key of the release signing key — the trust
/// root for update verification, compiled into the application.
///
/// The matching private half lives only in the repository's GitHub Actions
/// secrets (`GPG_PRIVATE_KEY` and `GPG_PASSPHRASE`), which the release
/// workflow uses to sign `SHA256SUMS.txt`; it is never committed here.
///
/// OWNER ACTION (pending): generate the release key pair, store the armoured
/// secret key and its passphrase as those two repository secrets, and paste
/// the armoured public key (`gpg --armor --export <fingerprint>`) here. Until
/// this constant holds the real key, verification fails closed:
/// [`verify_signature`] returns `false` for every input and
/// [`verify_update_installer`] refuses every download.
pub const RELEASE_PUBLIC_KEY: &str = "";

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

/// Verify the installer a release download URL points at, end to end, before
/// the caller hands it to the operating system.
///
/// The checksum file and its detached signature are fetched from the same
/// release as the installer, and every step fails closed with a plain
/// `Err(String)`:
///
/// 1. the signature over `SHA256SUMS.txt` must verify against the embedded
///    release public key ([`RELEASE_PUBLIC_KEY`]);
/// 2. the installer at `url` is downloaded (bounded) and its SHA-256 must
///    match the entry the signed checksum file lists for it.
///
/// Nothing is written to disk and nothing is executed here; on success the
/// caller holds installer bytes whose provenance is verified and decides what
/// happens next.
pub fn verify_update_installer(url: &str) -> Result<(), String> {
    let base = release_download_base(url)?;
    let checksums_text = http_get_text(&format!("{base}/SHA256SUMS.txt"), MAX_RESPONSE_BYTES)?;
    let signature = http_get_bytes(&format!("{base}/SHA256SUMS.txt.sig"), MAX_RESPONSE_BYTES)?;
    verify_checksum_and_signature(url, &checksums_text, &signature)
}

/// Check an installer against a checksum file and that file's detached
/// signature.
///
/// The signature is verified **first**, before anything is downloaded: an
/// unsigned or forged release is rejected without spending the bandwidth, and
/// the checksum listing is never trusted for even one download unless it is
/// authentic. The installer at `update_url` is then downloaded, capped at
/// [`MAX_INSTALLER_BYTES`], and its SHA-256 compared with the signed entry
/// for its file name.
fn verify_checksum_and_signature(
    update_url: &str,
    checksums_text: &str,
    signature_bytes: &[u8],
) -> Result<(), String> {
    if !verify_signature(signature_bytes, checksums_text.as_bytes()) {
        return Err("the checksum file is not signed with the Cadence release key".to_string());
    }
    let asset_name = asset_name_from_url(update_url)?;
    let installer = download_installer(update_url)?;
    verify_checksum(checksums_text, asset_name, &installer)
}

/// Verify a detached GPG signature over `data` against the embedded release
/// public key.
///
/// The release workflow signs with `gpg --armor`, so the signature is
/// expected armoured; a raw binary signature packet is accepted too. A
/// missing release key ([`RELEASE_PUBLIC_KEY`] not yet set by the owner), a
/// malformed signature and a key mismatch all return `false` — never a
/// panic, never a guess.
#[must_use]
fn verify_signature(signature_bytes: &[u8], data: &[u8]) -> bool {
    verify_signature_with_key(RELEASE_PUBLIC_KEY, signature_bytes, data)
}

/// [`verify_signature`] against an explicit armoured public key, so the tests
/// can exercise the logic with throwaway key pairs instead of the embedded
/// release key.
#[must_use]
fn verify_signature_with_key(
    public_key_armour: &str,
    signature_bytes: &[u8],
    data: &[u8],
) -> bool {
    let key = match SignedPublicKey::from_armor_single(Cursor::new(public_key_armour)) {
        Ok((key, _headers)) => key,
        Err(_) => return false,
    };
    match parse_detached_signature(signature_bytes) {
        Some(signature) => signature.verify(&key, data).is_ok(),
        None => false,
    }
}

/// Parse a detached signature, armoured or raw binary. The release workflow
/// signs with `gpg --armor`; the binary form is accepted so a hand-made
/// signature is not rejected for its encoding alone.
fn parse_detached_signature(signature_bytes: &[u8]) -> Option<DetachedSignature> {
    DetachedSignature::from_armor_single(Cursor::new(signature_bytes))
        .map(|(signature, _headers)| signature)
        .or_else(|_| DetachedSignature::from_bytes(Cursor::new(signature_bytes)))
        .ok()
}

/// Compare the SHA-256 of `data` with the (already verified) checksum listing
/// for `asset_name`.
fn verify_checksum(checksums_text: &str, asset_name: &str, data: &[u8]) -> Result<(), String> {
    let expected = expected_checksum(checksums_text, asset_name)
        .ok_or_else(|| format!("the checksum file does not list {asset_name}"))?;
    let actual = sha256_hex(data);
    if expected != actual {
        return Err(format!("the downloaded {asset_name} does not match its signed checksum"));
    }
    Ok(())
}

/// Find the checksum for `asset_name` in `sha256sum` output
/// (`<64 hex>␠␠<name>`; binary mode writes ` *` before the name instead of a
/// second space).
fn expected_checksum(checksums_text: &str, asset_name: &str) -> Option<String> {
    checksums_text.lines().find_map(|line| {
        let (hex, name) = line.split_once(' ')?;
        let name = name.trim_start();
        let name = name.strip_prefix('*').unwrap_or(name);
        if name == asset_name
            && hex.len() == 64
            && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            Some(hex.to_ascii_lowercase())
        } else {
            None
        }
    })
}

/// SHA-256 of `data`, lowercase hex.
fn sha256_hex(data: &[u8]) -> String {
    use std::fmt::Write as _;
    let digest = Sha256::digest(data);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in &digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// The `<tag>/` directory part of a release download URL — where the
/// checksums and their signature live, next to the installer.
fn release_download_base(url: &str) -> Result<&str, String> {
    let rest = url
        .strip_prefix(RELEASES_DOWNLOAD_BASE)
        .ok_or_else(|| format!("the update URL is not a Cadence release download: {url}"))?;
    let (tag, asset) = rest
        .split_once('/')
        .ok_or_else(|| format!("the update URL has no installer name: {url}"))?;
    if tag.is_empty() || asset.is_empty() {
        return Err(format!("the update URL is malformed: {url}"));
    }
    Ok(&url[..url.len() - asset.len() - 1])
}

/// The file name at the end of an update URL, as the checksum file lists it.
fn asset_name_from_url(url: &str) -> Result<&str, String> {
    let name = url.rsplit('/').next().unwrap_or_default();
    if name.is_empty() {
        return Err(format!("the update URL has no installer name: {url}"));
    }
    Ok(name)
}

/// Download up to `max_bytes` from `url` with the shared user agent.
fn http_get_bytes(url: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(DOWNLOAD_TIMEOUT)
        .timeout_read(DOWNLOAD_TIMEOUT)
        .timeout_write(DOWNLOAD_TIMEOUT)
        .user_agent(concat!("Cadence/", env!("CARGO_PKG_VERSION")))
        .build();
    let response = agent
        .get(url)
        .call()
        .map_err(|error| format!("download failed: {error}"))?;
    let mut body = Vec::new();
    response
        .into_reader()
        .take(max_bytes)
        .read_to_end(&mut body)
        .map_err(|error| format!("cannot read the download: {error}"))?;
    Ok(body)
}

/// [`http_get_bytes`], decoded as UTF-8 text.
fn http_get_text(url: &str, max_bytes: u64) -> Result<String, String> {
    let bytes = http_get_bytes(url, max_bytes)?;
    String::from_utf8(bytes).map_err(|_| "the download is not valid UTF-8".to_string())
}

/// Download the installer itself: same client, the larger installer byte cap.
fn download_installer(url: &str) -> Result<Vec<u8>, String> {
    http_get_bytes(url, MAX_INSTALLER_BYTES)
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
    use pgp::composed::{
        ArmorOptions, DetachedSignature, KeyType, SecretKeyParamsBuilder, SignedPublicKey,
        SignedSecretKey,
    };
    use pgp::crypto::hash::HashAlgorithm;
    use pgp::types::Password;

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

    // ----- release verification (D30) -----

    /// A throwaway Ed25519 key pair, so the verification logic is exercised
    /// without the (owner-pending) embedded release key.
    fn test_key() -> (SignedPublicKey, SignedSecretKey) {
        let secret = SecretKeyParamsBuilder::default()
            .key_type(KeyType::Ed25519Legacy)
            .can_certify(false)
            .can_sign(true)
            .primary_user_id("Cadence test key <dev@example.com>".into())
            .build()
            .expect("key parameters build")
            .generate(rand::thread_rng())
            .expect("key generates");
        (secret.to_public_key(), secret)
    }

    fn armoured(public: &SignedPublicKey) -> String {
        public
            .to_armored_string(ArmorOptions::default())
            .expect("public key armours")
    }

    /// Sign `data` with `secret`; the armoured detached signature the release
    /// workflow produces with `gpg --armor`.
    fn sign_armoured(secret: &SignedSecretKey, data: &[u8]) -> Vec<u8> {
        DetachedSignature::sign_binary_data(
            rand::thread_rng(),
            &secret.primary_key,
            &Password::empty(),
            HashAlgorithm::Sha256,
            data,
        )
        .expect("test signature signs")
        .to_armored_string(ArmorOptions::default())
        .expect("test signature armours")
        .into_bytes()
    }

    /// Two `sha256sum` lines; both hashes are real SHA-256 vectors (of `abc`
    /// and of `foo`).
    const CHECKSUMS: &str = "\
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  cadence-linux-x86_64.deb
2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae  cadence-windows-x86_64.msi
";

    #[test]
    fn a_signature_from_the_release_key_verifies() {
        let (public, secret) = test_key();
        let data = CHECKSUMS.as_bytes();
        let signature = sign_armoured(&secret, data);
        assert!(verify_signature_with_key(&armoured(&public), &signature, data));
    }

    #[test]
    fn a_tampered_checksum_fails_signature_verification() {
        let (public, secret) = test_key();
        let signature = sign_armoured(&secret, CHECKSUMS.as_bytes());
        let tampered = CHECKSUMS.replace("ba7816", "ba7817");
        assert!(!verify_signature_with_key(&armoured(&public), &signature, tampered.as_bytes()));
    }

    #[test]
    fn a_signature_from_a_foreign_key_fails_verification() {
        let (_public, secret) = test_key();
        let (other_public, _other_secret) = test_key();
        let signature = sign_armoured(&secret, CHECKSUMS.as_bytes());
        assert!(!verify_signature_with_key(
            &armoured(&other_public),
            &signature,
            CHECKSUMS.as_bytes()
        ));
    }

    #[test]
    fn an_unparseable_signature_is_rejected() {
        let (public, _secret) = test_key();
        assert!(!verify_signature_with_key(&armoured(&public), b"not a signature", b"data"));
        // The embedded release key is still unset (owner action, see
        // RELEASE_PUBLIC_KEY), so the public entry point fails closed.
        assert!(!verify_signature(b"not a signature", b"data"));
    }

    #[test]
    fn sha256_hex_matches_the_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn the_checksum_listing_is_matched_by_asset_name() {
        let linux = "cadence-linux-x86_64.deb";
        let hex = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(expected_checksum(CHECKSUMS, linux).as_deref(), Some(hex));
        // sha256sum's binary mode writes ` *` before the name.
        let binary = format!("{hex} *{linux}\n");
        assert_eq!(expected_checksum(&binary, linux).as_deref(), Some(hex));
        assert!(expected_checksum(CHECKSUMS, "cadence-unknown.deb").is_none());
        assert!(expected_checksum(CHECKSUMS, "not a checksum line").is_none());
    }

    #[test]
    fn the_installer_checksum_is_enforced() {
        assert_eq!(verify_checksum(CHECKSUMS, "cadence-linux-x86_64.deb", b"abc"), Ok(()));

        let mismatch = verify_checksum(CHECKSUMS, "cadence-linux-x86_64.deb", b"xyz");
        assert_eq!(
            mismatch.unwrap_err(),
            "the downloaded cadence-linux-x86_64.deb does not match its signed checksum"
        );

        let unlisted = verify_checksum(CHECKSUMS, "cadence-unknown.deb", b"abc");
        assert_eq!(
            unlisted.unwrap_err(),
            "the checksum file does not list cadence-unknown.deb"
        );
    }

    #[test]
    fn the_release_base_is_derived_from_the_download_url() {
        let url = concat!(
            "https://github.com/Perchemop/cadence/releases/download/",
            "v0.13.0/cadence-linux-x86_64.deb"
        );
        assert_eq!(
            release_download_base(url),
            Ok("https://github.com/Perchemop/cadence/releases/download/v0.13.0")
        );
        assert_eq!(asset_name_from_url(url), Ok("cadence-linux-x86_64.deb"));
    }

    #[test]
    fn a_non_release_url_is_rejected() {
        let base = "https://github.com/Perchemop/cadence/releases/download";
        assert!(release_download_base("https://example.com/cadence.deb").is_err());
        // The base with a trailing slash but no tag/asset has no installer name.
        let bare = format!("{base}/");
        assert!(release_download_base(&bare).is_err());
        assert!(asset_name_from_url("https://example.com/").is_err());
    }

    #[test]
    fn an_unsigned_checksum_file_rejects_the_download_before_any_fetch() {
        // The signature check runs first: this URL is never fetched (and the
        // test never touches the network) because the garbage signature
        // already fails verification.
        let url = concat!(
            "https://github.com/Perchemop/cadence/releases/download/",
            "v0.13.0/cadence-linux-x86_64.deb"
        );
        let error = verify_checksum_and_signature(url, CHECKSUMS, b"not a signature")
            .expect_err("an unsigned checksum file must be rejected");
        assert_eq!(
            error,
            "the checksum file is not signed with the Cadence release key"
        );
    }
}
