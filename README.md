# Cadence

**Cadence** is a free, offline scheduling application for construction and
capital-project work, written in pure Rust.

It does what professional scheduling tools do — work breakdown structures,
dependency logic, a from-scratch Critical Path Method engine, an interactive
Gantt chart, resources, calendars, baselines and earned-value metrics — without
a subscription, without an account, and without sending your project anywhere.
It exists so that a planner whose employer will not buy or host a commercial
tool can still produce a defensible schedule, on their own laptop, in a file
they own.

- **Genuinely offline.** No telemetry, no account, no network call in the
  scheduling, persistence, reporting or interchange paths.
- **Open file formats.** A documented, versioned SQLite project file; MSPDI
  (Microsoft Project XML) and MPX import/export; CSV and a lossless JSON format;
  Markdown and PDF reports.
- **Original and open source.** A clean-room implementation under
  MIT OR Apache-2.0, so it can be audited, extended and embedded.

## Getting started

```bash
cargo build --workspace                # build everything
cargo run -p cadence-app               # open the demo project in the desktop app
cargo run -p cadence-app -- project.sqlite   # open a project file
```

The command line does the same work without a window:

```bash
cadence                          # schedule the built-in demo and print it
cadence save project.sqlite      # write the demo to a project file
cadence load project.sqlite      # load a file and print its schedule
cadence report project.sqlite    # a Markdown report (schedule, resources, EVM)
cadence print  project.sqlite report.pdf   # the same report as a PDF
cadence export-xml project.sqlite out.xml  # hand the schedule to another tool
```

The report opens with a schedule table, so another person can read the result
without Cadence installed:

```
| ID | Activity | Days | Start | Finish | Float | Critical | Resources |
```

Prebuilt binaries for Linux (x86-64 and ARM64), Windows and macOS are attached
to each [release](https://github.com/Perchemop/cadence/releases), with SHA-256
checksums and a CycloneDX bill of materials.

## Where to get help, and where things are

| I want to… | Go to |
|---|---|
| Use the commands, formats and keyboard map | [`docs/usage.md`](docs/usage.md) |
| Know what Cadence can do today | [`docs/parity.md`](docs/parity.md) |
| Understand the design | [`docs/architecture.md`](docs/architecture.md) |
| See what is planned and how ready it is | [`ROADMAP.md`](ROADMAP.md) |
| Know what Cadence is for (and is not) | [`VISION.md`](VISION.md) |
| Contribute, or run the tests | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| Report a bug or request a feature | [Issues](https://github.com/Perchemop/cadence/issues/new/choose) |
| Report an accessibility problem | [Accessibility report](https://github.com/Perchemop/cadence/issues/new?template=accessibility_report.yml) |
| Report a security vulnerability | [`SECURITY.md`](SECURITY.md) — privately, never a public issue |
| Know who decides what | [`GOVERNANCE.md`](GOVERNANCE.md) |
| See who has contributed | [`CONTRIBUTORS.md`](CONTRIBUTORS.md) |
| Check the accessibility commitment | [`ACCESSIBILITY.md`](ACCESSIBILITY.md) |

## Status

Phases 0–4 are complete. Phase 5 is complete apart from manual,
platform-specific screen-reader passes: the code-level accessibility audit and
AccessKit wiring are done, and native print dialogs are implemented. The
remaining checks and the verification evidence are recorded in
[`docs/implementation-report.md`](docs/implementation-report.md) and
[`ACCESSIBILITY.md`](ACCESSIBILITY.md). `v0.1.0` was the first functional
release; [`ROADMAP.md`](ROADMAP.md) carries the living readiness percentages
and the v0.2.0 infrastructure work.

## Building and verifying

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Those commands are what CI runs, on Linux, Windows and macOS, alongside a
security workflow (secret scanning over the full history, dependency licence
and advisory checks, dependency review, CodeQL static analysis and SBOM
generation). Contributors should run them locally before opening a pull
request; see [`CONTRIBUTING.md`](CONTRIBUTING.md#running-the-tests) for the
detail and [`docs/development.md`](docs/development.md) for the workflow.

## Contributing

Contributions are welcome, and not only code: a bug report with a reproduction,
a triage, a documentation fix, or a test all count. Start with
[`CONTRIBUTING.md`](CONTRIBUTING.md), check the non-goals in
[`VISION.md`](VISION.md), and look for the `good first issue` and `help wanted`
labels.

Expectations, stated up front so they are easy to point at later:

- **A reply within seven days**, even if it is only an acknowledgement.
- **A few hours a week** of maintainer time — a hobby-scale project with one
  maintainer (see [`GOVERNANCE.md`](GOVERNANCE.md)).
- **Tests and CI must pass.** One bounded change per pull request.
- **Small, focused pull requests** are reviewed fastest.

Accessibility reports are treated as correctness reports, not as opinions.

## Privacy

Cadence works fully offline unless you choose to open a crash report in your
browser. If it crashes, a technical entry (version, operating system, panic
message, backtrace) is appended to a crash log in your system temporary
directory; that file stays on your machine and is never uploaded. On the next
launch the GUI shows the log, lets you add notes, and only opens a pre-filled
GitHub issue when you press **Open GitHub issue** — nothing is sent
automatically, and no token is embedded in the application. Update checks are
opt-in: `cadence version` uses only the last cached answer and never reaches
the network, while `cadence check-updates` or the GUI's **Updates → Check now**
sends one GET request to the public GitHub Releases API. The request includes
no project data, account, identifier or token; the answer is cached for a few
hours and failures are reported without blocking the application.

Badges do not lie here: nothing in this project measures you. The only metrics
recorded are the ones GitHub reports publicly (release downloads, clones,
issues), for the maintainer's own context — see
[`GOVERNANCE.md`](GOVERNANCE.md#metrics-this-project-tracks).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. The "MIT OR Apache-2.0" expression is deliberate: it is the
Rust-ecosystem convention, and the Apache-2.0 half carries the express patent
grant that larger organisations look for.

Third-party assets and dependencies, and the licences that cover them, are
listed in [`ATTRIBUTION.md`](ATTRIBUTION.md).
