# Contributing to Cadence

Thank you for considering a contribution. Cadence is a clean-room, free and
open-source scheduling application; a few rules keep it that way.

## Licence

By opening a pull request you agree that your contribution is licensed under
the repository's dual licence, **MIT OR Apache-2.0**, and that you have the
right to license it that way. No contributor licence agreement (CLA) is
required; this statement is the terms.

## How to propose a change

1. Open an issue or pick one from `ROADMAP.md`.
2. Fork, branch, and open a pull request using the repository's pull request
   template. The template's checklist is the review standard.
3. Keep changes small and reviewable — one bounded slice per pull request.

## Rules

- **Clean-room.** Never submit proprietary source code, decompiled material,
  copied UI trade dress, icons, fonts, string tables or internal algorithms
  from commercial products. Public documentation, specifications and your own
  original work are fine.
- **No secrets.** Never commit API keys, tokens, or credentials.
- **Original expression.** User-facing strings, branding and UI are original;
  do not claim affiliation with or endorsement by any vendor.
- **Tests with changes.** New behaviour needs tests; CI (format, clippy with
  warnings denied, full suite on Linux/Windows/macOS) must be green.
- **Heavy builds stay in CI.** The maintainers work on constrained machines;
  please let GitHub Actions do full builds and packaging.

## Reporting problems

Use GitHub Issues. Crash reports (when the in-app reporter is wired up) only
ever send what you explicitly consent to; see the README privacy note.
