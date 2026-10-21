//! Cadence core: scheduling domain model and Critical Path Method engine.
//!
//! This crate is pure, dependency-light domain logic. It must remain usable
//! headlessly from tests, the CLI, and (later) the GUI without any UI or I/O
//! dependencies.

pub mod calendar_set;
pub mod engine;
pub mod model;
pub mod wbs;

pub use calendar_set::{CalendarId, CalendarSet, CalendarSetError, NamedCalendar};
pub use model::{Activity, ActivityId, Calendar, CalendarError, LinkType, Project, Relationship};
pub use wbs::{WbsError, WbsId, WbsNode, WbsTree};
