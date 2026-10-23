# Cadence

**Cadence** is a free, open-source construction and capital-project scheduling
application written in pure Rust.

Professional-grade scheduling capabilities — hierarchical WBS, full dependency
logic (FS/SS/FF/SF with lag/lead), a from-scratch Critical Path Method engine,
interactive Gantt charts, resources, calendars, baselines, and earned-value
metrics — completely offline, with no telemetry and open file formats.

## Status

Phases 0–4 are complete and Phase 5 is complete except for two decided
deferrals (native print dialogs and the screen-reader audit) recorded in
[`docs/implementation-report.md`](docs/implementation-report.md), which also
lists the verification evidence. See [`docs/usage.md`](docs/usage.md) for
commands, file formats and the keyboard map,
[`docs/parity.md`](docs/parity.md) for the living capability list, and
[`docs/architecture.md`](docs/architecture.md) for the design.

## Building and verifying

All heavy work happens in CI, not on your machine — the repository is the
source of truth for builds and tests:

- `cargo test --workspace` locally is fine if you have the toolchain, but the
  authoritative check is GitHub Actions, which runs fmt, clippy (warnings
  denied) and the full suite on Linux, Windows and macOS and packages the
  release binary.
- [`docs/development.md`](docs/development.md) documents the workflow,
  conventions and expectations.

## Privacy

Cadence works fully offline and sends nothing anywhere. If it crashes, a
technical entry (version, operating system, panic message, backtrace) is
appended to a crash log in your system temporary directory; that file stays on
your machine and is never uploaded. Update checks make no network requests —
`cadence version` reports the running version and says so. If crash
submission or online update checks are added later, they will be opt-in and
documented here.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
