//! Open project interchange formats for Cadence.
//!
//! Cadence stores projects in its own SQLite file. This crate reads and writes
//! two open, text-based formats so that a project can leave and re-enter the
//! application without a proprietary toolchain:
//!
//! * **Microsoft Project XML** (MSPDI) — the XML dialect written by
//!   Microsoft Project. See [`mspdi`].
//! * **MPX** — the older Microsoft Project Exchange record format. See
//!   [`mpx`].
//!
//! Both modules implement a documented subset of their format. Everything that
//! Cadence models — the activity list, durations, constraints, relationships
//! with lag, the resource dictionary, assignments and baselines — is written
//! and read back. Fields that Cadence does not model (cost rates, calendars,
//! custom fields, levelling results) are ignored on import and omitted on
//! export rather than silently invented.
//!
//! [`csv`] is the exception to that scope: a CSV table carries one row per
//! activity, so it exchanges the activity list, its constraints and links, and
//! its assignments, but not baselines (see the module documentation for the
//! exact column list). [`json`] is the opposite: it is Cadence's own schema, so
//! it carries everything Cadence models — including the WBS tree, named
//! calendars and activity membership — and converts nothing. The other three
//! formats have no way to express the WBS or named calendars and leave them
//! empty on import. Outside the interchange family, [`schedule`] writes the
//! *derived* schedule of a project (dates, float, criticality) as JSON for
//! scripts and assistive technology; it is the only document this crate
//! writes that carries derived dates, and it is never read back.

pub mod csv;
pub mod json;
pub mod mpx;
pub mod mspdi;
pub mod schedule;
pub mod xml;

use std::fmt;

use time::{Date, Month};

/// Errors returned by the interchange readers and writers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterchangeError {
    /// The XML document is not well formed.
    Xml {
        /// What went wrong.
        message: String,
    },
    /// A required field is missing.
    MissingField(String),
    /// A field holds a value the format does not allow.
    InvalidField {
        /// Field path, for example `Task/ConstraintType`.
        field: String,
        /// The offending value.
        value: String,
    },
    /// The document uses a feature Cadence cannot represent.
    Unsupported(String),
    /// The project could not be scheduled while exporting.
    Schedule(String),
}

impl fmt::Display for InterchangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { message } => write!(formatter, "malformed XML: {message}"),
            Self::MissingField(field) => write!(formatter, "missing required field {field}"),
            Self::InvalidField { field, value } => {
                write!(formatter, "invalid value \"{value}\" for {field}")
            }
            Self::Unsupported(message) => write!(formatter, "unsupported document: {message}"),
            Self::Schedule(message) => write!(formatter, "project cannot be scheduled: {message}"),
        }
    }
}

impl std::error::Error for InterchangeError {}

/// Parse a calendar date written as `YYYY-MM-DD`.
pub fn parse_date(value: &str) -> Option<Date> {
    let mut parts = value.trim().split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u8>().ok()?;
    let day = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let month = Month::try_from(month).ok()?;
    Date::from_calendar_date(year, month, day).ok()
}

/// Parse the date part of an ISO 8601 date-time such as `2026-01-05T08:00:00`.
///
/// Microsoft Project writes local times without an offset, so only the date
/// part is significant for Cadence.
pub fn parse_date_prefix(value: &str) -> Option<Date> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("NA") {
        return None;
    }
    let date = trimmed.split('T').next().unwrap_or(trimmed).trim();
    parse_date(date)
}

/// Format a calendar date as `YYYY-MM-DD`.
pub fn format_date(date: Date) -> String {
    date.to_string()
}

/// Parse a decimal amount into minor currency units.
pub(crate) fn parse_money(text: &str) -> Option<i64> {
    let value: f64 = text.parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some((value * 100.0).round() as i64)
}

/// Format money held in minor units as a decimal amount.
pub(crate) fn format_money(minor_units: i64) -> String {
    let value = minor_units as f64 / 100.0;
    format!("{value:.2}")
}

/// The date used when a format carries no project start date at all.
pub(crate) fn default_start_date() -> Date {
    time::OffsetDateTime::now_utc().date()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip() {
        let date = Date::from_calendar_date(2026, Month::January, 5).expect("date");
        assert_eq!(format_date(date), "2026-01-05");
        assert_eq!(parse_date("2026-01-05"), Some(date));
    }

    #[test]
    fn date_times_are_read_by_their_date_part() {
        let date = Date::from_calendar_date(2026, Month::March, 9).expect("date");
        assert_eq!(parse_date_prefix("2026-03-09T08:00:00"), Some(date));
        assert_eq!(parse_date_prefix("2026-03-09"), Some(date));
        assert_eq!(parse_date_prefix("NA"), None);
        assert_eq!(parse_date_prefix(""), None);
    }

    #[test]
    fn malformed_dates_are_rejected() {
        assert_eq!(parse_date("2026-13-01"), None);
        assert_eq!(parse_date("2026-01"), None);
        assert_eq!(parse_date("2026-01-05-01"), None);
        assert_eq!(parse_date("not a date"), None);
    }
}
