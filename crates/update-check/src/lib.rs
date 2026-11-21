//! Shared suite crate: opt-in update checking against the public GitHub
//! Releases API (guide §12.1 suite standards, D27).
//!
//! The implementation lives in [`update_check`]; this crate is the suite-wide
//! home so a second application can consume the identical check without
//! depending on the GUI crate. `cadence-ui-egui` re-exports the module, so the
//! public path `cadence_ui_egui::update_check` is unchanged from when the code
//! was an inline module.

pub mod update_check;
