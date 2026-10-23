# Research — update check

Status: draft, not human-approved.

## Behaviour wanted

"Check for updates" queries the GitHub Releases API, compares the latest
published version with the running one, and reports up-to-date / update
available with a link. Offline it fails soft and silently.

## Acceptance criteria

1. The check runs only when the user asks (no background polling in v0.2.0).
2. Offline or on any API error the UI shows "could not check" — never an
   error dialog, never a crash (the panic hook must not fire for this).
3. Version comparison is semantic: `v0.2.0` > `v0.1.9`, prereleases compare
   correctly.
4. The result links to the release page so the user can download the binary
   themselves (no auto-update in scope).

## Implementation direction (original notes)

- List the latest release: `GET /repos/{owner}/{repo}/releases/latest` —
  returns `tag_name` and `html_url` for non-draft, non-prerelease releases:
  https://docs.github.com/en/rest/releases/releases?apiVersion=2022-11-28#get-the-latest-release
- The comparison needs no dependency: split on `.`, compare numerically, treat
  a missing field as 0; prerelease suffix (`-rc.1`) sorts before the plain
  release.
- Network access from the GUI binary: the `ureq` crate (sync, small) or the
  `reqwest` crate (larger, async). Decision at implementation time; ureq is
  the better fit for the project's dependency-light stance. License MIT/Apache.
- Cache the last check time in the project directory so a user who restarts
  often is not rate-limited (GitHub allows 60 unauthenticated requests/hour per
  IP — the check is user-triggered, so abuse is unlikely, but the cache is
  polite).

## Sources

- REST API — get the latest release: https://docs.github.com/en/rest/releases/releases?apiVersion=2022-11-28#get-the-latest-release
- Semantic versioning: https://semver.org/
