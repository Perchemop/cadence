# Attribution

Cadence uses the following third-party assets and libraries. All are
permissively licensed.

## Fonts, icons, and artwork

None yet. All visual design in Cadence is original.

## Software dependencies

Direct dependencies of the workspace crates:

| Crate | License | Used by | Purpose |
|-------|---------|---------|---------|
| `time` | MIT OR Apache-2.0 | core, commands, persistence, interchange, ui-egui, app | Date and calendar arithmetic |
| `serde` | MIT OR Apache-2.0 | core, commands | Serialisation derives on the model |
| `thiserror` | MIT OR Apache-2.0 | core, persistence | Error types |
| `rusqlite` (bundled SQLite) | MIT | persistence | SQLite project files |
| `egui` | MIT OR Apache-2.0 | ui-egui | Immediate-mode GUI |
| `eframe` | MIT OR Apache-2.0 | ui-egui | Windowing and the egui frame |

Development-only dependencies (tests, property tests):

| Crate | License | Used by | Purpose |
|-------|---------|---------|---------|
| `proptest` | MIT OR Apache-2.0 | core, interchange | Property-based tests |
| `serde_json` | MIT OR Apache-2.0 | core | Serde round-trip tests |

Transitive dependencies are recorded in `Cargo.lock`; `cargo tree` lists them
for any crate. None of them are required at runtime beyond the GUI stack
(`egui`/`eframe` and their platform crates) and SQLite.

This list is updated whenever a direct dependency is added.
