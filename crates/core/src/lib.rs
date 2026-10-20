//! Cadence core: scheduling domain model and Critical Path Method engine.
//!
//! This crate is pure, dependency-light domain logic. It must remain usable
//! headlessly from tests, the CLI, and (later) the GUI without any UI or I/O
//! dependencies.

pub mod engine;
pub mod model;
