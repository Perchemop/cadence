# Attribution

Cadence uses the following third-party assets and libraries. All are
permissively licensed. This file is part of the release checklist: it is
updated whenever a dependency or an asset is added.

## Licensing policy

Every dependency and asset must fall in one of these classes, and the class is
the reason the entry is acceptable:

| Class | Accepted? | Why |
|---|---|---|
| Permissive (MIT, Apache-2.0, ISC, BSD, Zlib, Unlicense/CC0) | Yes | No obligation beyond attribution |
| Weak / limited copyleft (MPL-2.0, LGPL) | Yes, if isolated to a library boundary | Extra rules are followed; none are used directly today |
| Strong copyleft (GPL, AGPL) | **No** | It would change the licence of the whole project |
| Source-available, non-open-source (BUSL, SSPL) | **No** | Usage and business-model restrictions; would stop Cadence being open source |
| Non-commercial (CC-BY-NC) | **No** | Incompatible with a tool meant to be freely usable |

The same list is enforced mechanically by `deny.toml` in the
[security workflow](.github/workflows/security.yml), so a disallowed licence
fails CI rather than being discovered later. Nothing in this project is
"source-available" rather than open source.

## Fonts, icons, and artwork

All visual design in Cadence is original — original shapes drawn with the GUI
toolkit's own primitives and original colour choices. No commercial product's
icons, string tables or trade dress have been copied, and none will be.

Fonts are the GUI toolkit's bundled defaults: the `epaint_default_fonts` crate
(embedded by `eframe`) ships third-party font assets, and they carry their own
licences, which are separate from the code licence:

| Asset | Licence | Source |
|---|---|---|
| Ubuntu (default proportional font) | Ubuntu Font Licence 1.0 | embedded by `epaint_default_fonts` |
| Noto Emoji | SIL Open Font License 1.1 | embedded by `epaint_default_fonts` |

Both are permissive, OSI-approved font licences, allow-listed in `deny.toml`
for exactly this purpose and checked by the security workflow. Any further
non-code asset is listed here with its licence before it ships.

## Software dependencies

Direct dependencies of the workspace crates:

| Crate | License | Used by | Purpose |
|-------|---------|---------|---------|
| `time` | MIT OR Apache-2.0 | core, commands, persistence, interchange, ui-egui, app | Date and calendar arithmetic |
| `serde` | MIT OR Apache-2.0 | core, commands | Serialisation derives on the model |
| `thiserror` | MIT OR Apache-2.0 | core, persistence | Error types |
| `rusqlite` (bundled SQLite) | MIT (SQLite itself is public domain) | persistence | SQLite project files |
| `egui` | MIT OR Apache-2.0 | ui-egui | Immediate-mode GUI |
| `eframe` | MIT OR Apache-2.0 | ui-egui | Windowing and the egui frame |
| `rfd` | MIT | app, ui-egui | Native save dialog for the PDF report (D16) |

Development-only dependencies (tests, property tests):

| Crate | License | Used by | Purpose |
|-------|---------|---------|---------|
| `proptest` | MIT OR Apache-2.0 | core, interchange | Property-based tests |
| `serde_json` | MIT OR Apache-2.0 | core | Serde round-trip tests |

Transitive dependencies are recorded in `Cargo.lock`; `cargo tree` lists them
for any crate. None of them are required at runtime beyond the GUI stack
(`egui`/`eframe` and their platform crates) and SQLite. Two transitive notes,
recorded because they are the parts of the tree that are not plain MIT/Apache:
the GUI stack pulls in `MPL-2.0` components (allowed, and used at the library
boundary only), and the bundled SQLite is public domain.

## Tools, models and services used to build Cadence

The build process itself uses tooling whose terms matter to the provenance of
this repository. Recorded here so the story is auditable:

| Item | Terms | How it is used |
|---|---|---|
| Rust toolchain, `cargo`, `clippy`, `rustfmt` | MIT OR Apache-2.0 | Compiling, linting and formatting |
| GitHub Actions and its published actions | Vendor terms (see each action) | Building, testing and releasing; actions are pinned to commit SHAs |
| AI coding models used by the maintainer's build process | Vendor/model terms, free tiers | Writing code and documentation from the maintainer's direction |

Provenance rules followed: original code, public specifications and standards
(CPM, MSPDI, MPX, SQLite file format, WCAG), and capability-level inspiration
only. No proprietary or reverse-engineered material is included, and no model
output is committed where its terms would forbid redistribution.

## Bill of materials

Each release carries a CycloneDX SBOM listing every component, version and
licence in the dependency set. Current release assets are on the
[releases page](https://github.com/Perchemop/cadence/releases); the
[security workflow](.github/workflows/security.yml) generates one per run, and
the [release workflow](.github/workflows/release.yml) attaches one to the
release itself.
