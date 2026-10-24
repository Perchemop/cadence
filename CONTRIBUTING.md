# Contributing to Cadence

Thank you for considering a contribution. Cadence is a clean-room, free and
open-source scheduling application; a few rules keep it that way.

This document tells you how to report a problem, propose a feature, set up the
project, run the tests, and what to expect from review. Everything here is
written down so that a "no" is about the project's criteria rather than about
you.

## Ways to contribute

A merged patch is not the only contribution, and often not the most valuable
one:

- **Report a bug** with a reproduction — a minimal project file that shows the
  wrong result is the single most useful artefact anyone can send.
- **Report an accessibility problem** with the dedicated template.
- **Propose a feature** (after reading the goals and non-goals in
  [`VISION.md`](VISION.md)).
- **Improve the documentation** — the usage guide, the parity list, the README.
- **Triage** an existing issue: confirm a reproduction, narrow a version range,
  add the missing environment detail.
- **Write tests** for behaviour that is currently only exercised by hand.
- **Code** — see below.

Areas where an owner would be especially welcome: accessibility,
documentation, interchange formats (MSPDI/MPX), and security triage. If you
want to own one of those, say so; see
[`GOVERNANCE.md`](GOVERNANCE.md#how-to-become-a-maintainer).

## Before you start

1. Read [`VISION.md`](VISION.md). It lists the non-goals — cloud, web, mobile,
   ERP/macros, solvers, reverse-engineered formats — and most declined requests
   are declined against that list.
2. Search the issues. If your idea is already there, add to that thread rather
   than opening a duplicate.
3. For anything larger than a small fix, **open an issue first** and describe
   the problem you are solving. It is much cheaper to agree on an approach
   before writing code than after.
4. Look for the `good first issue` and `help wanted` labels if you want
   something scoped and ready.

## Setting up

Prerequisites: a [Rust toolchain](https://rustup.rs/) (stable) and a C compiler
for the bundled SQLite build. `git`, and nothing else, is required for the core
crates.

```bash
git clone https://github.com/Perchemop/cadence.git
cd cadence
cargo build --workspace          # build everything
cargo run -p cadence-app -- --help   # the CLI
cargo run -p cadence-app             # the desktop application (no file)
cargo run -p cadence-app -- path/to/project.sqlite   # open a project
```

The CLI commands, the file formats and the keyboard map are documented in
[`docs/usage.md`](docs/usage.md). The architecture is in
[`docs/architecture.md`](docs/architecture.md), and the development conventions
(commit identity, module layout, documentation rules) are in
[`docs/development.md`](docs/development.md).

## Running the tests

Run them locally; that is the normal way to work on Cadence:

```bash
cargo fmt --all -- --check                          # formatting
cargo clippy --workspace --all-targets -- -D warnings   # lints are errors
cargo test --workspace                              # unit + integration tests
cargo test -p cadence-core -- --nocapture           # one crate, with output
```

Property tests (the `proptest` suites) are part of `cargo test`; they are the
reason a scheduling change that is "obviously fine" sometimes is not.

**Note for this repository's maintainer only.** The maintainer's own machine is
memory-constrained, so *here* the authoritative result is GitHub Actions: the
maintainer pushes a branch and lets CI run the same commands on Linux, Windows
and macOS (see the badge and the workflow results). That is a statement about
one machine, not about the project — contributors are expected and encouraged to
run the suite locally, and a pull request that explains what was run locally is
easier to review.

## What you can expect from review

- **A response within seven days.** If it will take longer, you will get a note
  saying so rather than silence.
- **A few hours a week of maintainer time.** Cadence is a hobby-scale project
  with one maintainer; see
  [`GOVERNANCE.md`](GOVERNANCE.md#sustainability-and-boundaries).
- **Small pull requests get reviewed fastest.** One bounded change per pull
  request. Unrelated reformatting makes a diff hard to review and will be asked
  to be split out.
- **Tests are required for behaviour changes.** A bug fix without a regression
  test will be asked for a test; a new feature without tests is not ready.
- **The pull request template checklist is the review standard.** Fill it in,
  including the accessibility section for UI changes.
- **CI must be green.** `cargo fmt`, `clippy -D warnings` and the full test
  suite run on Linux, Windows and macOS, plus the security workflow.

## Reporting problems

| Kind of problem | Where it goes |
|---|---|
| Wrong result, crash, file will not open | **Bug report** issue template |
| Unusable with keyboard, screen reader or high contrast | **Accessibility report** issue template |
| A capability you are missing | **Feature request** issue template |
| A security vulnerability | **Privately**, per [`SECURITY.md`](SECURITY.md) — never a public issue |
| A code-of-conduct problem | **Privately**, per [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) |

Crash reports (when the in-app reporter is wired up) only ever send what you
explicitly consent to; see the README privacy note.

## Rules

- **Clean-room.** Never submit proprietary source code, decompiled material,
  copied UI trade dress, icons, fonts, string tables or internal algorithms
  from commercial products. Public documentation, specifications and your own
  original work are fine.
- **No secrets.** Never commit API keys, tokens, or credentials. The security
  workflow scans the full history; a secret in a commit is a leaked secret even
  after it is deleted.
- **Original expression.** User-facing strings, branding and UI are original;
  do not claim affiliation with or endorsement by any vendor.
- **Tests with changes.** New behaviour needs tests; CI (format, clippy with
  warnings denied, full suite on Linux/Windows/macOS) must be green.
- **Dependencies are a decision.** A new dependency needs a reason: its licence
  must be permissive (the licence check in CI enforces the allow-list), it is
  recorded in [`ATTRIBUTION.md`](ATTRIBUTION.md), and it must be justified in
  the pull request.
- **Accessibility is acceptance criteria.** A UI change is not done until the
  accessibility section of the pull request template is true, and any new
  limitation is recorded in [`ACCESSIBILITY.md`](ACCESSIBILITY.md).
- **Heavy builds stay in CI.** The maintainer works on a constrained machine;
  please let GitHub Actions do full builds and packaging. This does *not*
  restrict you: run whatever you need locally.

## Licence and contributor terms

By opening a pull request you agree that your contribution is licensed under
the repository's dual licence, **MIT OR Apache-2.0**, and that you have the
right to license it that way. GitHub's terms of service already make
"inbound = outbound" the default, so **no contributor licence agreement and no
signed-off-by line is required** — this statement is the terms. If your
employer claims rights over your work, see
[`VISION.md`](VISION.md) (open questions) before contributing.

## Saying no

Not every good idea fits Cadence. Declined contributions get a short reply that
thanks you, names the scope criterion from [`VISION.md`](VISION.md), and links
the relevant document — usually the non-goals list. That is a decision about the
project, not a judgement of the work. Repeated questions are answered by
improving the documentation, so the answer only has to be given once.
