//! Domain model for Cadence.
//!
//! The model is deliberately free of UI and I/O concerns. Everything here is
//! plain data plus a small amount of calendar arithmetic, which makes the
//! scheduling engine easy to test headlessly.
//!
//! Durations, lags and floats are measured in **working days**. Dates are
//! calendar dates (`time::Date`) and every date produced by the engine falls
//! on a working day under the activity's calendar (the project calendar in
//! this phase).

use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Date, Duration};

use crate::calendar_set::{CalendarId, CalendarSet, CalendarSetError};
use crate::wbs::{WbsId, WbsTree};

/// Stable identifier for an [`Activity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActivityId(pub u64);

impl ActivityId {
    /// Construct an activity identifier from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for ActivityId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Stable identifier for a [`Relationship`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RelationshipId(pub u64);

impl RelationshipId {
    /// Construct a relationship identifier from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for RelationshipId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Stable identifier for a [`Resource`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceId(pub u64);

impl ResourceId {
    /// Construct a resource identifier from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A resource that can be assigned to activities (e.g. labour, equipment, material).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Stable identifier.
    pub id: ResourceId,
    /// Human-readable name.
    pub name: String,
    /// Resource kind (labour, equipment, material).
    #[serde(default)]
    pub kind: ResourceKind,
    /// Maximum units the resource can supply on one working day.
    ///
    /// Every activity assignment consumes one unit per working day, so a
    /// capacity of two lets the resource work on two activities at the same
    /// time while a capacity of one forces them to be serialised.
    #[serde(default = "default_resource_capacity")]
    pub capacity: u32,
}

/// Default resource capacity: one unit per working day.
const fn default_resource_capacity() -> u32 {
    1
}

impl Resource {
    /// Create a resource with the given identifier and name.
    ///
    /// The capacity defaults to one unit per working day.
    pub fn new(id: ResourceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            kind: ResourceKind::Labour,
            capacity: default_resource_capacity(),
        }
    }

    /// Create a resource with an explicit capacity.
    ///
    /// A capacity below one is raised to one, because a resource that can
    /// supply nothing could never be scheduled.
    pub fn with_capacity(id: ResourceId, name: impl Into<String>, capacity: u32) -> Self {
        Self {
            id,
            name: name.into(),
            kind: ResourceKind::Labour,
            capacity: capacity.max(default_resource_capacity()),
        }
    }
}

/// Resource kind for categorisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResourceKind {
    /// Generic labour resource.
    #[default]
    Labour,
    /// Equipment or machine resource.
    Equipment,
    /// Material resource.
    Material,
}

/// Precedence relationship type between two activities.
///
/// The four public CPM link types are:
///
/// * `FS` — finish-to-start: the successor starts after the predecessor finishes.
/// * `SS` — start-to-start: the successor starts relative to the predecessor start.
/// * `FF` — finish-to-finish: the successor finishes relative to the predecessor finish.
/// * `SF` — start-to-finish: the successor finishes relative to the predecessor start.
///
/// Variant names use Rust's usual casing; serialised forms use the conventional
/// upper-case abbreviations (`"FS"`, `"SS"`, `"FF"`, `"SF"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LinkType {
    /// Finish-to-start.
    Fs,
    /// Start-to-start.
    Ss,
    /// Finish-to-finish.
    Ff,
    /// Start-to-finish.
    Sf,
}

impl LinkType {
    /// Conventional two-letter abbreviation.
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::Fs => "FS",
            Self::Ss => "SS",
            Self::Ff => "FF",
            Self::Sf => "SF",
        }
    }

    /// All relationship types, in documentation order.
    pub const ALL: [Self; 4] = [Self::Fs, Self::Ss, Self::Ff, Self::Sf];

    /// Parse a conventional abbreviation (`"FS"`, `"SS"`, `"FF"`, `"SF"`).
    ///
    /// Matching is case-insensitive. Returns `None` for any other value.
    pub fn from_abbreviation(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "FS" => Some(Self::Fs),
            "SS" => Some(Self::Ss),
            "FF" => Some(Self::Ff),
            "SF" => Some(Self::Sf),
            _ => None,
        }
    }
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.abbreviation())
    }
}

/// Errors produced by working-day calendar arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CalendarError {
    /// The calendar has no working weekdays, so working-day arithmetic is
    /// undefined.
    #[error("calendar has no working weekdays")]
    NoWorkingWeekdays,
    /// The requested arithmetic falls outside the range of representable dates.
    #[error("date arithmetic is outside the supported range")]
    DateOutOfRange,
}

/// A working-time calendar expressed as a weekly pattern plus holiday exceptions.
///
/// This is intentionally a value type rather than a trait so that calendars can
/// be serialised with a project and compared in tests. Projects hold a default
/// calendar plus a set of named calendars ([`crate::calendar_set::CalendarSet`]);
/// activities may select a named calendar and otherwise inherit the default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    /// Working flags indexed by day of week, Monday = 0 through Sunday = 6.
    working_weekdays: [bool; 7],
    /// Non-working exception dates, regardless of weekday.
    #[serde(default)]
    holidays: BTreeSet<Date>,
}

impl Calendar {
    /// Build a calendar from a Monday-first weekly pattern.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoWorkingWeekdays`] when every weekday is a
    /// non-working day.
    pub fn new(working_weekdays: [bool; 7]) -> Result<Self, CalendarError> {
        if working_weekdays.iter().any(|working| *working) {
            Ok(Self {
                working_weekdays,
                holidays: BTreeSet::new(),
            })
        } else {
            Err(CalendarError::NoWorkingWeekdays)
        }
    }

    /// The default Monday-to-Friday calendar with no holidays.
    pub fn weekend() -> Self {
        Self {
            working_weekdays: [true, true, true, true, true, false, false],
            holidays: BTreeSet::new(),
        }
    }

    /// Add a single non-working exception date.
    pub fn with_holiday(mut self, date: Date) -> Self {
        self.holidays.insert(date);
        self
    }

    /// Add several non-working exception dates.
    pub fn with_holidays<I>(mut self, dates: I) -> Self
    where
        I: IntoIterator<Item = Date>,
    {
        self.holidays.extend(dates);
        self
    }

    /// Monday-first weekly working pattern.
    pub fn working_weekdays(&self) -> [bool; 7] {
        self.working_weekdays
    }

    /// Exception dates configured on this calendar.
    pub fn holidays(&self) -> &BTreeSet<Date> {
        &self.holidays
    }

    /// Number of working weekdays per full week.
    pub fn working_days_per_week(&self) -> usize {
        self.working_weekdays.iter().filter(|day| **day).count()
    }

    /// Whether `date` is a working day under this calendar.
    pub fn is_working_day(&self, date: Date) -> bool {
        self.working_weekdays[weekday_index(date)] && !self.holidays.contains(&date)
    }

    /// First working day on or after `date`.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoWorkingWeekdays`] for a calendar with no
    /// working weekdays, or [`CalendarError::DateOutOfRange`] if the search
    /// runs past the representable date range.
    pub fn next_working_day_on_or_after(&self, date: Date) -> Result<Date, CalendarError> {
        if self.working_days_per_week() == 0 {
            return Err(CalendarError::NoWorkingWeekdays);
        }
        let mut current = date;
        loop {
            if self.is_working_day(current) {
                return Ok(current);
            }
            current = current
                .checked_add(Duration::days(1))
                .ok_or(CalendarError::DateOutOfRange)?;
        }
    }

    /// First working day strictly after `date`.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoWorkingWeekdays`] for a calendar with no
    /// working weekdays, or [`CalendarError::DateOutOfRange`] when the date is
    /// outside the representable range.
    pub fn next_working_day_after(&self, date: Date) -> Result<Date, CalendarError> {
        let next = date
            .checked_add(Duration::days(1))
            .ok_or(CalendarError::DateOutOfRange)?;
        self.next_working_day_on_or_after(next)
    }

    /// Last working day strictly before `date`.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoWorkingWeekdays`] for a calendar with no
    /// working weekdays, or [`CalendarError::DateOutOfRange`] when the date is
    /// outside the representable range.
    pub fn prev_working_day_before(&self, date: Date) -> Result<Date, CalendarError> {
        let previous = date
            .checked_sub(Duration::days(1))
            .ok_or(CalendarError::DateOutOfRange)?;
        self.prev_working_day_on_or_before(previous)
    }

    /// Last working day on or before `date`.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoWorkingWeekdays`] for a calendar with no
    /// working weekdays, or [`CalendarError::DateOutOfRange`] if the search
    /// runs past the representable date range.
    pub fn prev_working_day_on_or_before(&self, date: Date) -> Result<Date, CalendarError> {
        if self.working_days_per_week() == 0 {
            return Err(CalendarError::NoWorkingWeekdays);
        }
        let mut current = date;
        loop {
            if self.is_working_day(current) {
                return Ok(current);
            }
            current = current
                .checked_sub(Duration::days(1))
                .ok_or(CalendarError::DateOutOfRange)?;
        }
    }

    /// Move `days` working days from `date`.
    ///
    /// Positive values move forward and negative values move backward. Zero
    /// normalises a non-working date to the next working day on or after it.
    /// The anchor is always a working day; moving one working day forward from
    /// a Friday gives the following Monday, and moving one working day backward
    /// from a Monday gives the preceding Friday.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid calendar or when the result falls
    /// outside the representable date range.
    pub fn add_working_days(&self, date: Date, days: i64) -> Result<Date, CalendarError> {
        if self.working_days_per_week() == 0 {
            return Err(CalendarError::NoWorkingWeekdays);
        }
        match days.cmp(&0) {
            Ordering::Equal => self.next_working_day_on_or_after(date),
            Ordering::Greater => {
                let anchor = self.next_working_day_on_or_after(date)?;
                self.find_forward(anchor, days)
            }
            Ordering::Less => {
                let anchor = self.prev_working_day_on_or_before(date)?;
                let distance = days.checked_neg().ok_or(CalendarError::DateOutOfRange)?;
                self.find_backward(anchor, distance)
            }
        }
    }

    /// Signed number of working days between two dates.
    ///
    /// The count is taken over the half-open interval `[earlier, later)`, so a
    /// Friday to the following Monday is one working day and Monday to Friday
    /// is four. The function is intended for working-day anchors (the values
    /// produced by [`Calendar::add_working_days`]); non-working anchors still
    /// return a stable result.
    pub fn working_days_between(&self, from: Date, to: Date) -> i64 {
        match from.cmp(&to) {
            Ordering::Less => self.count_working_days_between(from, to),
            Ordering::Greater => -self.count_working_days_between(to, from),
            Ordering::Equal => 0,
        }
    }

    /// Count working days in `[start, end)`. `start` must be before `end`.
    fn count_working_days_between(&self, start: Date, end: Date) -> i64 {
        let weekdays = self.count_weekdays_between(start, end);
        let holiday_count = self
            .holidays
            .range(start..end)
            .filter(|date| self.working_weekdays[weekday_index(**date)])
            .count();
        // A calendar cannot hold more holidays than memory allows, which is far
        // below i64::MAX, so this conversion is lossless in practice.
        weekdays - holiday_count as i64
    }

    /// Count working weekdays in `[start, end)` ignoring holidays.
    fn count_weekdays_between(&self, start: Date, end: Date) -> i64 {
        let days = i64::from(end.to_julian_day()) - i64::from(start.to_julian_day());
        let per_week = self.working_days_per_week() as i64;
        let full_weeks = days / 7;
        let remainder = days % 7;
        let mut count = full_weeks * per_week;
        let first_weekday = weekday_index(start);
        for offset in 0..remainder {
            let index = (first_weekday + offset as usize) % 7;
            if self.working_weekdays[index] {
                count += 1;
            }
        }
        count
    }

    /// Extra calendar days that are guaranteed to contain `distance` working
    /// days even if every holiday in the calendar falls inside the window.
    fn search_window(&self, distance: i64) -> Result<i64, CalendarError> {
        let holiday_padding = i64::try_from(self.holidays.len())
            .ok()
            .and_then(|len| len.checked_mul(7))
            .ok_or(CalendarError::DateOutOfRange)?;
        distance
            .checked_mul(7)
            .and_then(|base| base.checked_add(holiday_padding))
            .and_then(|base| base.checked_add(7))
            .ok_or(CalendarError::DateOutOfRange)
    }

    /// Find the first date whose working-day distance from `anchor` is at least
    /// `distance`, then normalise it to a working day.
    fn find_forward(&self, anchor: Date, distance: i64) -> Result<Date, CalendarError> {
        let window = self.search_window(distance)?;
        let anchor_julian = i64::from(anchor.to_julian_day());
        let mut low = anchor_julian;
        let mut high = anchor_julian
            .checked_add(window)
            .ok_or(CalendarError::DateOutOfRange)?;

        let high_date = date_from_julian(high)?;
        if self.working_days_between(anchor, high_date) < distance {
            return Err(CalendarError::DateOutOfRange);
        }

        while low < high {
            let mid = low + (high - low) / 2;
            let mid_date = date_from_julian(mid)?;
            if self.working_days_between(anchor, mid_date) >= distance {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        let found = date_from_julian(low)?;
        self.next_working_day_on_or_after(found)
    }

    /// Find the latest date whose working-day distance back to `anchor` is at
    /// least `distance`, then normalise it to a working day.
    fn find_backward(&self, anchor: Date, distance: i64) -> Result<Date, CalendarError> {
        let window = self.search_window(distance)?;
        let anchor_julian = i64::from(anchor.to_julian_day());
        let mut low = anchor_julian
            .checked_sub(window)
            .ok_or(CalendarError::DateOutOfRange)?;
        let mut high = anchor_julian;

        let low_date = date_from_julian(low)?;
        if self.working_days_between(low_date, anchor) < distance {
            return Err(CalendarError::DateOutOfRange);
        }

        while low < high {
            let mid = low + (high - low + 1) / 2;
            let mid_date = date_from_julian(mid)?;
            if self.working_days_between(mid_date, anchor) >= distance {
                low = mid;
            } else {
                high = mid - 1;
            }
        }

        let found = date_from_julian(low)?;
        self.prev_working_day_on_or_before(found)
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::weekend()
    }
}

/// Convert a signed Julian-day number into a `time::Date`.
fn date_from_julian(julian_day: i64) -> Result<Date, CalendarError> {
    let julian_day = i32::try_from(julian_day).map_err(|_| CalendarError::DateOutOfRange)?;
    Date::from_julian_day(julian_day).map_err(|_| CalendarError::DateOutOfRange)
}

/// Monday-first index into a weekly calendar (Monday = 0, Sunday = 6).
fn weekday_index(date: Date) -> usize {
    usize::from(date.weekday().number_days_from_monday())
}

/// Scheduling constraint attached to an activity.
///
/// The default is [`Constraint::AsSoonAsPossible`]. Date constraints are
/// inclusive bounds interpreted on the activity's calendar: a lower bound
/// moves forward to the next working day and an upper bound moves backward to
/// the previous working day.
///
/// `AsLateAsPossible` delays the activity as far as its float and successors
/// allow, without moving any successor earlier. It never delays the project
/// finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Constraint {
    /// Schedule as early as the logic and lower-bound constraints allow.
    #[default]
    AsSoonAsPossible,
    /// Schedule as late as possible without delaying successors or the project.
    AsLateAsPossible,
    /// Start no earlier than the given date.
    StartNoEarlierThan(Date),
    /// Start no later than the given date.
    StartNoLaterThan(Date),
    /// Finish no earlier than the given date.
    FinishNoEarlierThan(Date),
    /// Finish no later than the given date.
    FinishNoLaterThan(Date),
}

impl Constraint {
    /// Stable lower-case code used by project files.
    pub const fn code(self) -> &'static str {
        match self {
            Self::AsSoonAsPossible => "asap",
            Self::AsLateAsPossible => "alap",
            Self::StartNoEarlierThan(_) => "snet",
            Self::StartNoLaterThan(_) => "snlt",
            Self::FinishNoEarlierThan(_) => "fnet",
            Self::FinishNoLaterThan(_) => "fnlt",
        }
    }

    /// Constraint date, when the variant carries one.
    pub const fn date(self) -> Option<Date> {
        match self {
            Self::StartNoEarlierThan(date)
            | Self::StartNoLaterThan(date)
            | Self::FinishNoEarlierThan(date)
            | Self::FinishNoLaterThan(date) => Some(date),
            Self::AsSoonAsPossible | Self::AsLateAsPossible => None,
        }
    }

    /// Rebuild a constraint from a project-file code and optional date.
    ///
    /// Returns `None` when the code is unknown, a date variant has no date, or
    /// a mode variant carries a date.
    pub fn from_parts(code: &str, date: Option<Date>) -> Option<Self> {
        let code = code.trim().to_ascii_lowercase();
        match (code.as_str(), date) {
            ("asap", None) => Some(Self::AsSoonAsPossible),
            ("alap", None) => Some(Self::AsLateAsPossible),
            ("snet", Some(date)) => Some(Self::StartNoEarlierThan(date)),
            ("snlt", Some(date)) => Some(Self::StartNoLaterThan(date)),
            ("fnet", Some(date)) => Some(Self::FinishNoEarlierThan(date)),
            ("fnlt", Some(date)) => Some(Self::FinishNoLaterThan(date)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AsSoonAsPossible => formatter.write_str("as soon as possible"),
            Self::AsLateAsPossible => formatter.write_str("as late as possible"),
            Self::StartNoEarlierThan(date) => {
                write!(formatter, "start no earlier than {date}")
            }
            Self::StartNoLaterThan(date) => write!(formatter, "start no later than {date}"),
            Self::FinishNoEarlierThan(date) => {
                write!(formatter, "finish no earlier than {date}")
            }
            Self::FinishNoLaterThan(date) => write!(formatter, "finish no later than {date}"),
        }
    }
}

/// A single scheduled activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    /// Stable identifier.
    pub id: ActivityId,
    /// Human-readable name.
    pub name: String,
    /// Duration in working days. Zero denotes a milestone.
    pub original_duration: u32,
    /// Optional work-breakdown-structure node this activity belongs to.
    #[serde(default)]
    pub wbs: Option<WbsId>,
    /// Optional resources assigned to this activity.
    #[serde(default)]
    pub resources: Vec<ResourceId>,
    /// Optional named calendar. `None` selects the project default calendar.
    #[serde(default)]
    pub calendar: Option<CalendarId>,
    /// Scheduling constraint. Defaults to as soon as possible.
    #[serde(default)]
    pub constraint: Constraint,
    /// Earliest start produced by the forward pass.
    #[serde(default)]
    pub early_start: Option<Date>,
    /// Earliest finish produced by the forward pass.
    #[serde(default)]
    pub early_finish: Option<Date>,
    /// Latest start produced by the backward pass.
    #[serde(default)]
    pub late_start: Option<Date>,
    /// Latest finish produced by the backward pass.
    #[serde(default)]
    pub late_finish: Option<Date>,
    /// Scheduled start: early for ASAP activities, late for ALAP activities.
    #[serde(default)]
    pub scheduled_start: Option<Date>,
    /// Scheduled finish: early for ASAP activities, late for ALAP activities.
    #[serde(default)]
    pub scheduled_finish: Option<Date>,
    /// Total float in working days.
    #[serde(default)]
    pub total_float: Option<i64>,
    /// Free float in working days.
    #[serde(default)]
    pub free_float: Option<i64>,
    /// Whether total float is zero or negative.
    #[serde(default)]
    pub critical: bool,
    /// Budget at completion: total planned cost for this activity, in minor
    /// currency units (e.g. cents).
    #[serde(default)]
    pub budget_at_completion: Option<i64>,
    /// Actual cost incurred to date, in minor currency units.
    #[serde(default)]
    pub actual_cost: Option<i64>,
    /// Percent complete, stored in basis points (0 to 10000).
    #[serde(default)]
    pub percent_complete: Option<i64>,
    /// Date work actually started, as recorded by the user.
    #[serde(default)]
    pub actual_start: Option<Date>,
    /// Date work actually finished, as recorded by the user.
    #[serde(default)]
    pub actual_finish: Option<Date>,
    /// Who is responsible for the activity (field-planning owner).
    #[serde(default)]
    pub owner: Option<String>,
    /// True when the activity is a promised near-term task in the lookahead.
    #[serde(default)]
    pub promised: bool,
    /// The reason recorded when a promised activity's dates moved.
    #[serde(default)]
    pub delay_reason: Option<String>,
    /// Optional free-text work location or area, used for clash detection.
    #[serde(default)]
    pub location: Option<String>,
}

impl Activity {
    /// Create an unscheduled activity.
    pub fn new(id: ActivityId, name: impl Into<String>, original_duration: u32) -> Self {
        Self {
            id,
            name: name.into(),
            original_duration,
            wbs: None,
            resources: Vec::new(),
            calendar: None,
            constraint: Constraint::AsSoonAsPossible,
            early_start: None,
            early_finish: None,
            late_start: None,
            late_finish: None,
            scheduled_start: None,
            scheduled_finish: None,
            total_float: None,
            free_float: None,
            critical: false,
            budget_at_completion: None,
            actual_cost: None,
            percent_complete: None,
            actual_start: None,
            actual_finish: None,
            owner: None,
            promised: false,
            delay_reason: None,
            location: None,
        }
    }

    /// Clear cached schedule results, for example after a duration change.
    pub fn clear_schedule(&mut self) {
        self.early_start = None;
        self.early_finish = None;
        self.late_start = None;
        self.late_finish = None;
        self.scheduled_start = None;
        self.scheduled_finish = None;
        self.total_float = None;
        self.free_float = None;
        self.critical = false;
    }
}

/// A precedence relationship between two activities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    /// Stable identifier.
    pub id: RelationshipId,
    /// Activity that must satisfy the relationship.
    pub predecessor: ActivityId,
    /// Activity constrained by the relationship.
    pub successor: ActivityId,
    /// Relationship type.
    pub link_type: LinkType,
    /// Lag in working days. Negative values are leads.
    pub lag: i64,
}

impl Relationship {
    /// Create a relationship.
    pub fn new(
        id: RelationshipId,
        predecessor: ActivityId,
        successor: ActivityId,
        link_type: LinkType,
        lag: i64,
    ) -> Self {
        Self {
            id,
            predecessor,
            successor,
            link_type,
            lag,
        }
    }
}

/// Root aggregate for a scheduling project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Project name.
    pub name: String,
    /// Calendar date on which the forward pass is anchored.
    pub start_date: Date,
    /// Activities in display order.
    #[serde(default)]
    pub activities: Vec<Activity>,
    /// Precedence relationships.
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    /// Default project calendar, used by activities that select no named
    /// calendar. Defaults to Monday-to-Friday.
    #[serde(default)]
    pub calendar: Calendar,
    /// Named calendars that activities may select instead of the default.
    #[serde(default)]
    pub calendars: CalendarSet,
    /// Work breakdown structure. Defaults to a single root node.
    #[serde(default)]
    pub wbs: WbsTree,
    /// Resource dictionary: resources available to assign to activities.
    #[serde(default)]
    pub resources: Vec<Resource>,
    /// Baselines: snapshots of the project schedule for earned value analysis.
    #[serde(default)]
    pub baselines: Vec<Baseline>,
    /// Lookahead snapshots: published, read-only views of the near-term plan.
    #[serde(default)]
    pub lookahead_snapshots: Vec<LookaheadSnapshot>,
    /// Daily notes: dated diary entries linked to activities or lookahead windows.
    #[serde(default)]
    pub daily_notes: Vec<DailyNote>,
    /// Production entries: dated actual progress recorded against activities.
    #[serde(default)]
    pub production_entries: Vec<ProductionEntry>,
}

impl Project {
    /// Create an empty project with the default weekend calendar.
    pub fn new(name: impl Into<String>, start_date: Date) -> Self {
        Self {
            name: name.into(),
            start_date,
            activities: Vec::new(),
            relationships: Vec::new(),
            calendar: Calendar::default(),
            calendars: CalendarSet::new(),
            wbs: WbsTree::new(),
            resources: Vec::new(),
            baselines: Vec::new(),
            lookahead_snapshots: Vec::new(),
            daily_notes: Vec::new(),
            production_entries: Vec::new(),
        }
    }

    /// Next unused activity identifier.
    ///
    /// The identifier is derived from the current maximum so it remains stable
    /// for a project loaded from disk. It saturates rather than overflowing if
    /// the maximum value is already `u64::MAX`.
    pub fn next_activity_id(&self) -> ActivityId {
        ActivityId(
            self.activities
                .iter()
                .map(|activity| activity.id.0)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
        )
    }

    /// Next unused relationship identifier.
    pub fn next_relationship_id(&self) -> RelationshipId {
        RelationshipId(
            self.relationships
                .iter()
                .map(|relationship| relationship.id.0)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
        )
    }

    /// Next unused WBS node identifier.
    pub fn next_wbs_id(&self) -> WbsId {
        self.wbs.next_id()
    }

    /// Next unused calendar identifier.
    pub fn next_calendar_id(&self) -> CalendarId {
        self.calendars.next_id()
    }

    /// Next unused baseline identifier.
    pub fn next_baseline_id(&self) -> BaselineId {
        BaselineId(
            self.baselines
                .iter()
                .map(|baseline| baseline.id.0)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
        )
    }

    /// Next unused lookahead snapshot identifier.
    pub fn next_lookahead_snapshot_id(&self) -> LookaheadSnapshotId {
        LookaheadSnapshotId(
            self.lookahead_snapshots
                .iter()
                .map(|snapshot| snapshot.id.0)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
        )
    }

    /// Next unused daily note identifier.
    pub fn next_daily_note_id(&self) -> DailyNoteId {
        DailyNoteId(
            self.daily_notes
                .iter()
                .map(|note| note.id.0)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
        )
    }

    /// Next unused production entry identifier.
    pub fn next_production_entry_id(&self) -> ProductionEntryId {
        ProductionEntryId(
            self.production_entries
                .iter()
                .map(|entry| entry.id.0)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
        )
    }

    /// Next unused resource identifier.
    pub fn next_resource_id(&self) -> ResourceId {
        ResourceId(
            self.resources
                .iter()
                .map(|resource| resource.id.0)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
        )
    }

    /// Look up a resource by identifier.
    pub fn resource(&self, id: ResourceId) -> Option<&Resource> {
        self.resources.iter().find(|resource| resource.id == id)
    }

    /// Look up a resource by identifier for modification.
    pub fn resource_mut(&mut self, id: ResourceId) -> Option<&mut Resource> {
        self.resources.iter_mut().find(|resource| resource.id == id)
    }

    /// Resolve an optional calendar selection to a calendar value.
    ///
    /// `None` selects the project default calendar. A named selection must
    /// exist in [`Project::calendars`].
    ///
    /// # Errors
    ///
    /// Returns [`CalendarSetError::UnknownCalendar`] when the named calendar is
    /// not present.
    pub fn resolve_calendar(
        &self,
        calendar: Option<CalendarId>,
    ) -> Result<&Calendar, CalendarSetError> {
        match calendar {
            None => Ok(&self.calendar),
            Some(id) => self.calendars.calendar(id),
        }
    }

    /// Look up an activity by identifier.
    pub fn activity(&self, id: ActivityId) -> Option<&Activity> {
        self.activities.iter().find(|activity| activity.id == id)
    }

    /// Mutably look up an activity by identifier.
    pub fn activity_mut(&mut self, id: ActivityId) -> Option<&mut Activity> {
        self.activities
            .iter_mut()
            .find(|activity| activity.id == id)
    }

    /// Look up a relationship by identifier.
    pub fn relationship(&self, id: RelationshipId) -> Option<&Relationship> {
        self.relationships
            .iter()
            .find(|relationship| relationship.id == id)
    }

    /// Mutably look up a relationship by identifier.
    pub fn relationship_mut(&mut self, id: RelationshipId) -> Option<&mut Relationship> {
        self.relationships
            .iter_mut()
            .find(|relationship| relationship.id == id)
    }
}

/// Stable identifier for a [`Baseline`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BaselineId(pub u64);

impl BaselineId {
    /// Construct a baseline identifier from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for BaselineId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A snapshot of one activity's planned values at the time a baseline is
/// captured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityBaselineSnapshot {
    /// Activity identifier.
    pub activity_id: ActivityId,
    /// Planned duration in working days at baseline time.
    pub baseline_duration: u32,
    /// Planned scheduled start at baseline time.
    pub baseline_start: Option<Date>,
    /// Planned scheduled finish at baseline time.
    pub baseline_finish: Option<Date>,
}

/// A baseline: a snapshot of the project schedule used for earned value
/// management and variance analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    /// Stable identifier.
    pub id: BaselineId,
    /// Human-readable name (e.g. "Baseline 1", "Board-approved").
    pub name: String,
    /// Date the baseline was captured.
    pub created_at: Date,
    /// Activity snapshots, in activity order.
    #[serde(default)]
    pub snapshots: Vec<ActivityBaselineSnapshot>,
}

impl Baseline {
    /// Create a baseline from explicit activity snapshots.
    ///
    /// The snapshots are stored in ascending activity order so that a
    /// baseline written twice compares equal.
    pub fn from_snapshots(
        id: BaselineId,
        name: impl Into<String>,
        created_at: Date,
        mut snapshots: Vec<ActivityBaselineSnapshot>,
    ) -> Self {
        snapshots.sort_by_key(|snapshot| snapshot.activity_id);
        Self {
            id,
            name: name.into(),
            created_at,
            snapshots,
        }
    }

    /// Look up the snapshot recorded for an activity.
    pub fn snapshot(&self, activity_id: ActivityId) -> Option<&ActivityBaselineSnapshot> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.activity_id == activity_id)
    }
}

/// Stable identifier for a [`LookaheadSnapshot`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LookaheadSnapshotId(pub u64);

impl LookaheadSnapshotId {
    /// Construct a lookahead snapshot identifier from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for LookaheadSnapshotId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A snapshot of one activity's values at the time a lookahead window is
/// published.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LookaheadSnapshotEntry {
    /// Activity identifier.
    pub activity_id: ActivityId,
    /// Activity name, copied for display.
    pub name: String,
    /// Scheduled start at publish time.
    pub scheduled_start: Date,
    /// Scheduled finish at publish time.
    pub scheduled_finish: Date,
    /// Field-planning owner at publish time.
    pub owner: Option<String>,
    /// True when the activity was promised at publish time.
    pub promised: bool,
    /// Percent complete in basis points at publish time.
    pub percent_complete: i64,
}

impl LookaheadSnapshotEntry {
    /// Construct a new lookahead snapshot entry.
    pub fn new(
        activity_id: ActivityId,
        name: String,
        scheduled_start: Date,
        scheduled_finish: Date,
        owner: Option<String>,
        promised: bool,
        percent_complete: i64,
    ) -> Self {
        Self {
            activity_id,
            name,
            scheduled_start,
            scheduled_finish,
            owner,
            promised,
            percent_complete,
        }
    }
}

/// A published lookahead window: a named, read-only snapshot of the near-term
/// plan that can be compared against later windows to detect drift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LookaheadSnapshot {
    /// Stable identifier.
    pub id: LookaheadSnapshotId,
    /// Human-readable name (e.g. "Week 1-2", "Pre-mobilisation").
    pub name: String,
    /// Date the snapshot was captured.
    pub created_at: Date,
    /// First day of the window that was published.
    pub window_start: Date,
    /// Length of the window in calendar weeks.
    pub window_weeks: u32,
    /// Activity snapshots, in activity order.
    #[serde(default)]
    pub entries: Vec<LookaheadSnapshotEntry>,
}

impl LookaheadSnapshot {
    /// Create a lookahead snapshot from explicit activity entries.
    ///
    /// The entries are stored in ascending activity order so that a snapshot
    /// written twice compares equal.
    pub fn from_entries(
        id: LookaheadSnapshotId,
        name: impl Into<String>,
        created_at: Date,
        window_start: Date,
        window_weeks: u32,
        mut entries: Vec<LookaheadSnapshotEntry>,
    ) -> Self {
        entries.sort_by_key(|entry| entry.activity_id);
        Self {
            id,
            name: name.into(),
            created_at,
            window_start,
            window_weeks,
            entries,
        }
    }

    /// Look up the snapshot entry recorded for an activity.
    pub fn entry(&self, activity_id: ActivityId) -> Option<&LookaheadSnapshotEntry> {
        self.entries
            .iter()
            .find(|entry| entry.activity_id == activity_id)
    }
}

/// Stable identifier for a [`DailyNote`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DailyNoteId(pub u64);

impl DailyNoteId {
    /// Construct a daily note identifier from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for DailyNoteId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A dated diary entry linked to an activity or a lookahead window.
///
/// Daily notes are site records — what happened, what was delayed, what needs
/// escalating — captured against a specific date. They are linked to an
/// activity when the note concerns that specific task, or to a lookahead
/// window when the note concerns the window as a whole.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DailyNote {
    /// Stable identifier.
    pub id: DailyNoteId,
    /// Date of the diary entry.
    pub date: Date,
    /// Note content.
    pub content: String,
    /// Optional activity the note concerns.
    pub activity_id: Option<ActivityId>,
    /// Optional lookahead window start the note concerns.
    pub lookahead_window_start: Option<Date>,
}

impl DailyNote {
    /// Construct a new daily note.
    pub fn new(
        id: DailyNoteId,
        date: Date,
        content: String,
        activity_id: Option<ActivityId>,
        lookahead_window_start: Option<Date>,
    ) -> Self {
        Self {
            id,
            date,
            content,
            activity_id,
            lookahead_window_start,
        }
    }
}

/// Stable identifier for a [`ProductionEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProductionEntryId(pub u64);

impl ProductionEntryId {
    /// Construct a production entry identifier from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for ProductionEntryId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// One day of recorded actual progress on an activity.
///
/// Production tracking records what was actually achieved each day against the
/// plan: the percent complete reached on that date, in basis points (0 to
/// 10 000). Recording an entry for an activity and date replaces any earlier
/// entry for the same pair, so the latest report for a day wins — matching how
/// site reports are corrected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProductionEntry {
    /// Stable identifier.
    pub id: ProductionEntryId,
    /// Activity the progress was recorded against.
    pub activity_id: ActivityId,
    /// Date the progress was recorded as of.
    pub date: Date,
    /// Percent complete reached on that date, in basis points (0 to 10 000).
    pub percent_complete: i64,
}

impl ProductionEntry {
    /// Construct a new production entry.
    pub fn new(
        id: ProductionEntryId,
        activity_id: ActivityId,
        date: Date,
        percent_complete: i64,
    ) -> Self {
        Self {
            id,
            activity_id,
            date,
            percent_complete,
        }
    }

    /// The planned share of the activity that should have been complete by this
    /// entry's date, in basis points, from the schedule.
    ///
    /// Whole working days between the scheduled start and the entry date,
    /// divided by the duration, following the same convention as the earned
    /// value planned percent.
    pub fn planned_percent(&self, activity: &Activity, calendar: &Calendar) -> i64 {
        let Some(scheduled_start) = activity.scheduled_start else {
            return 0;
        };
        if activity.original_duration == 0 {
            return if self.date >= scheduled_start {
                10_000
            } else {
                0
            };
        }
        let duration = i64::from(activity.original_duration);
        let elapsed = calendar
            .working_days_between(scheduled_start, self.date)
            .min(duration);
        i64::from(elapsed) * 10_000 / duration
    }
}

/// Earned value metrics for a project or activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvmMetrics {
    /// Budget at completion (planned total cost), in minor currency units.
    pub budget_at_completion: i64,
    /// Planned value: budgeted cost of work scheduled, in minor currency units.
    pub planned_value: i64,
    /// Earned value: budgeted cost of work performed, in minor currency units.
    pub earned_value: i64,
    /// Actual cost: actual cost of work performed, in minor currency units.
    pub actual_cost: i64,
    /// Cost variance: EV - AC.
    pub cost_variance: i64,
    /// Schedule variance: EV - PV.
    pub schedule_variance: i64,
    /// Cost performance index: EV / AC (None when AC is zero).
    pub cost_performance_index: Option<f64>,
    /// Schedule performance index: EV / PV (None when PV is zero).
    pub schedule_performance_index: Option<f64>,
}

/// Saturate an `i128` value into the `i64` range used by [`EvmMetrics`].
///
/// Earned-value arithmetic is computed in `i128` so that a large budget cannot
/// overflow. The public fields stay `i64`, so an out-of-range result is clamped
/// to the nearest representable value instead of wrapping or panicking.
pub(crate) fn saturate_i64(value: i128) -> i64 {
    value.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

impl EvmMetrics {
    /// Compute earned value metrics for one activity.
    ///
    /// `planned_fraction` is the share of the activity that the baseline
    /// schedules as complete at the status date, expressed in basis points
    /// (0 to 10000). Earned value comes from the activity's
    /// `percent_complete`, which uses the same basis-point scale.
    ///
    /// Cost and schedule performance indices are `None` when their denominator
    /// is zero, so an activity with no budget or no planned work does not
    /// report a misleading ratio.
    pub fn for_activity(activity: &Activity, planned_fraction: i64) -> Self {
        let bac = activity.budget_at_completion.unwrap_or(0);
        let ac = activity.actual_cost.unwrap_or(0);
        let pct = activity.percent_complete.unwrap_or(0).clamp(0, 10_000);
        let fraction = planned_fraction.clamp(0, 10_000);
        // Compute in `i128`: `bac * 10000` overflows `i64` for any budget above
        // roughly 9.2e14, which a large project can easily reach.
        let pv = saturate_i64(i128::from(bac) * i128::from(fraction) / 10_000);
        let ev = saturate_i64(i128::from(bac) * i128::from(pct) / 10_000);
        let cv = ev.saturating_sub(ac);
        let sv = ev.saturating_sub(pv);
        let cpi = if ac != 0 {
            Some(ev as f64 / ac as f64)
        } else {
            None
        };
        let spi = if pv != 0 {
            Some(ev as f64 / pv as f64)
        } else {
            None
        };

        Self {
            budget_at_completion: bac,
            planned_value: pv,
            earned_value: ev,
            actual_cost: ac,
            cost_variance: cv,
            schedule_variance: sv,
            cost_performance_index: cpi,
            schedule_performance_index: spi,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid test date")
    }

    fn jan(day: u8) -> Date {
        date(2026, Month::January, day)
    }

    #[test]
    fn weekend_calendar_knows_working_days() {
        let calendar = Calendar::weekend();
        assert!(calendar.is_working_day(jan(5))); // Monday
        assert!(calendar.is_working_day(jan(9))); // Friday
        assert!(!calendar.is_working_day(jan(10))); // Saturday
        assert!(!calendar.is_working_day(jan(11))); // Sunday
        assert_eq!(calendar.working_days_per_week(), 5);
    }

    #[test]
    fn calendar_adds_working_days_across_weekends() {
        let calendar = Calendar::weekend();
        let monday = jan(5);

        assert_eq!(calendar.add_working_days(monday, 0), Ok(monday));
        assert_eq!(calendar.add_working_days(monday, 1), Ok(jan(6)));
        assert_eq!(calendar.add_working_days(monday, 4), Ok(jan(9)));
        assert_eq!(calendar.add_working_days(monday, 5), Ok(jan(12)));
        assert_eq!(calendar.add_working_days(jan(9), 1), Ok(jan(12)));
        assert_eq!(calendar.add_working_days(jan(9), 2), Ok(jan(13)));
    }

    #[test]
    fn calendar_subtracts_working_days_across_weekends() {
        let calendar = Calendar::weekend();

        assert_eq!(calendar.add_working_days(jan(12), -1), Ok(jan(9)));
        assert_eq!(calendar.add_working_days(jan(12), -5), Ok(jan(5)));
        assert_eq!(calendar.add_working_days(jan(9), -1), Ok(jan(8)));
        assert_eq!(calendar.add_working_days(jan(5), -1), Ok(jan(2)));
        assert_eq!(calendar.add_working_days(jan(9), 0), Ok(jan(9)));
    }

    #[test]
    fn calendar_normalises_non_working_zero_offsets() {
        let calendar = Calendar::weekend();

        assert_eq!(calendar.add_working_days(jan(10), 0), Ok(jan(12)));
        assert_eq!(calendar.add_working_days(jan(11), 0), Ok(jan(12)));
        assert_eq!(calendar.next_working_day_on_or_after(jan(10)), Ok(jan(12)));
        assert_eq!(calendar.prev_working_day_on_or_before(jan(11)), Ok(jan(9)));
    }

    #[test]
    fn calendar_skips_holidays() {
        let calendar = Calendar::weekend().with_holiday(jan(5));
        assert!(!calendar.is_working_day(jan(5)));
        assert_eq!(calendar.add_working_days(jan(2), 1), Ok(jan(6)));
        assert_eq!(calendar.add_working_days(jan(6), -1), Ok(jan(2)));
    }

    #[test]
    fn calendar_counts_working_days_between_dates() {
        let calendar = Calendar::weekend();

        assert_eq!(calendar.working_days_between(jan(5), jan(9)), 4);
        assert_eq!(calendar.working_days_between(jan(9), jan(12)), 1);
        assert_eq!(calendar.working_days_between(jan(5), jan(12)), 5);
        assert_eq!(calendar.working_days_between(jan(12), jan(9)), -1);
        assert_eq!(calendar.working_days_between(jan(5), jan(5)), 0);
    }

    #[test]
    fn calendar_rejects_empty_week() {
        assert_eq!(
            Calendar::new([false; 7]),
            Err(CalendarError::NoWorkingWeekdays)
        );
    }

    #[test]
    fn identifiers_display_as_raw_values() {
        assert_eq!(ActivityId::new(7).to_string(), "7");
        assert_eq!(RelationshipId::new(9).to_string(), "9");
    }

    #[test]
    fn link_types_use_conventional_abbreviations() {
        assert_eq!(LinkType::Fs.to_string(), "FS");
        assert_eq!(LinkType::Ss.to_string(), "SS");
        assert_eq!(LinkType::Ff.to_string(), "FF");
        assert_eq!(LinkType::Sf.to_string(), "SF");
        assert_eq!(LinkType::ALL.len(), 4);
    }

    #[test]
    fn link_types_parse_from_abbreviations_case_insensitively() {
        for link_type in LinkType::ALL {
            assert_eq!(
                LinkType::from_abbreviation(link_type.abbreviation()),
                Some(link_type)
            );
            assert_eq!(
                LinkType::from_abbreviation(&link_type.abbreviation().to_lowercase()),
                Some(link_type)
            );
        }
        assert_eq!(LinkType::from_abbreviation("FS "), Some(LinkType::Fs));
        assert_eq!(LinkType::from_abbreviation("XX"), None);
        assert_eq!(LinkType::from_abbreviation(""), None);
    }

    #[test]
    fn project_allocates_sequential_identifiers() {
        let mut project = Project::new("Test", jan(5));
        assert_eq!(project.next_activity_id(), ActivityId(1));
        project
            .activities
            .push(Activity::new(ActivityId(1), "A", 1));
        project
            .activities
            .push(Activity::new(ActivityId(4), "B", 1));
        assert_eq!(project.next_activity_id(), ActivityId(5));

        assert_eq!(project.next_relationship_id(), RelationshipId(1));
        project.relationships.push(Relationship::new(
            RelationshipId(3),
            ActivityId(1),
            ActivityId(4),
            LinkType::Fs,
            0,
        ));
        assert_eq!(project.next_relationship_id(), RelationshipId(4));
    }

    #[test]
    fn activity_clear_schedule_removes_derived_values() {
        let mut activity = Activity::new(ActivityId(1), "A", 2);
        activity.early_start = Some(jan(5));
        activity.early_finish = Some(jan(6));
        activity.late_start = Some(jan(5));
        activity.late_finish = Some(jan(6));
        activity.scheduled_start = Some(jan(5));
        activity.scheduled_finish = Some(jan(6));
        activity.total_float = Some(0);
        activity.free_float = Some(0);
        activity.critical = true;

        activity.clear_schedule();

        assert_eq!(activity.early_start, None);
        assert_eq!(activity.early_finish, None);
        assert_eq!(activity.late_start, None);
        assert_eq!(activity.late_finish, None);
        assert_eq!(activity.scheduled_start, None);
        assert_eq!(activity.scheduled_finish, None);
        assert_eq!(activity.total_float, None);
        assert_eq!(activity.free_float, None);
        assert!(!activity.critical);
    }

    #[test]
    fn project_round_trips_through_json() {
        let calendar = Calendar::weekend().with_holiday(jan(6));
        let mut project = Project::new("Serde project", jan(5));
        project.calendar = calendar;

        let mut activity = Activity::new(ActivityId(1), "Excavate", 3);
        activity.early_start = Some(jan(5));
        activity.early_finish = Some(jan(7));
        activity.late_start = Some(jan(5));
        activity.late_finish = Some(jan(8));
        activity.scheduled_start = Some(jan(5));
        activity.scheduled_finish = Some(jan(7));
        activity.total_float = Some(1);
        activity.free_float = Some(0);
        activity.critical = false;
        activity.constraint = Constraint::StartNoEarlierThan(jan(5));
        project.activities.push(activity);

        project.relationships.push(Relationship::new(
            RelationshipId(1),
            ActivityId(1),
            ActivityId(2),
            LinkType::Ss,
            -1,
        ));

        let json = serde_json::to_string(&project).expect("serialise project");
        let decoded: Project = serde_json::from_str(&json).expect("deserialise project");
        assert_eq!(decoded, project);
    }

    #[test]
    fn link_type_round_trips_through_json() {
        for link_type in LinkType::ALL {
            let json = serde_json::to_string(&link_type).expect("serialise link type");
            assert_eq!(json, format!("\"{}\"", link_type.abbreviation()));
            let decoded: LinkType = serde_json::from_str(&json).expect("deserialise link type");
            assert_eq!(decoded, link_type);
        }
    }

    #[test]
    fn activities_default_to_asap() {
        let activity = Activity::new(ActivityId(1), "A", 1);
        assert_eq!(activity.constraint, Constraint::AsSoonAsPossible);
        assert_eq!(activity.constraint.code(), "asap");
        assert_eq!(activity.constraint.date(), None);
    }

    #[test]
    fn constraints_round_trip_codes_and_json() {
        let constraints = [
            Constraint::AsSoonAsPossible,
            Constraint::AsLateAsPossible,
            Constraint::StartNoEarlierThan(jan(5)),
            Constraint::StartNoLaterThan(jan(9)),
            Constraint::FinishNoEarlierThan(jan(12)),
            Constraint::FinishNoLaterThan(jan(16)),
        ];

        for constraint in constraints {
            let decoded =
                Constraint::from_parts(constraint.code(), constraint.date()).expect("known code");
            assert_eq!(decoded, constraint);

            let json = serde_json::to_string(&constraint).expect("serialise constraint");
            let from_json: Constraint =
                serde_json::from_str(&json).expect("deserialise constraint");
            assert_eq!(from_json, constraint);
        }

        assert_eq!(
            Constraint::from_parts(" SNET ", Some(jan(5))),
            Some(Constraint::StartNoEarlierThan(jan(5)))
        );
        assert_eq!(Constraint::from_parts("asap", Some(jan(5))), None);
        assert_eq!(Constraint::from_parts("snet", None), None);
        assert_eq!(Constraint::from_parts("mfo", Some(jan(5))), None);
    }

    #[test]
    fn baselines_sort_their_snapshots_by_activity() {
        let baseline = Baseline::from_snapshots(
            BaselineId(1),
            "Baseline 1",
            jan(5),
            vec![
                ActivityBaselineSnapshot {
                    activity_id: ActivityId(3),
                    baseline_duration: 1,
                    baseline_start: Some(jan(7)),
                    baseline_finish: Some(jan(7)),
                },
                ActivityBaselineSnapshot {
                    activity_id: ActivityId(1),
                    baseline_duration: 2,
                    baseline_start: Some(jan(5)),
                    baseline_finish: Some(jan(6)),
                },
            ],
        );

        assert_eq!(baseline.snapshots.len(), 2);
        assert_eq!(baseline.snapshots[0].activity_id, ActivityId(1));
        assert_eq!(baseline.snapshots[1].activity_id, ActivityId(3));
        assert_eq!(
            baseline
                .snapshot(ActivityId(3))
                .map(|s| s.baseline_duration),
            Some(1)
        );
        assert!(baseline.snapshot(ActivityId(2)).is_none());
    }

    #[test]
    fn evm_metrics_derive_variances_from_budget_and_progress() {
        let mut activity = Activity::new(ActivityId(1), "A", 4);
        activity.budget_at_completion = Some(1_000);
        activity.actual_cost = Some(400);
        activity.percent_complete = Some(2_500);

        // A quarter of the budget is planned and a quarter is earned.
        let metrics = EvmMetrics::for_activity(&activity, 2_500);
        assert_eq!(metrics.budget_at_completion, 1_000);
        assert_eq!(metrics.planned_value, 250);
        assert_eq!(metrics.earned_value, 250);
        assert_eq!(metrics.actual_cost, 400);
        assert_eq!(metrics.cost_variance, -150);
        assert_eq!(metrics.schedule_variance, 0);
        let cpi = metrics
            .cost_performance_index
            .expect("cost performance index");
        assert!((cpi - 0.625).abs() < 1e-9, "cpi was {cpi}");
        let spi = metrics
            .schedule_performance_index
            .expect("schedule performance index");
        assert!((spi - 1.0).abs() < 1e-9, "spi was {spi}");
    }

    #[test]
    fn evm_metrics_omit_indices_without_a_denominator() {
        let activity = Activity::new(ActivityId(1), "A", 4);

        let metrics = EvmMetrics::for_activity(&activity, 0);
        assert_eq!(metrics.budget_at_completion, 0);
        assert_eq!(metrics.planned_value, 0);
        assert_eq!(metrics.earned_value, 0);
        assert_eq!(metrics.cost_performance_index, None);
        assert_eq!(metrics.schedule_performance_index, None);
    }

    #[test]
    fn evm_metrics_clamp_the_planned_fraction() {
        let mut activity = Activity::new(ActivityId(1), "A", 4);
        activity.budget_at_completion = Some(1_000);

        let over = EvmMetrics::for_activity(&activity, 50_000);
        assert_eq!(over.planned_value, 1_000);

        let under = EvmMetrics::for_activity(&activity, -5_000);
        assert_eq!(under.planned_value, 0);
    }

    #[test]
    fn evm_metrics_saturate_on_a_huge_budget_without_panicking() {
        let mut activity = Activity::new(ActivityId(1), "A", 4);
        activity.budget_at_completion = Some(i64::MAX);
        activity.actual_cost = Some(i64::MIN);
        activity.percent_complete = Some(10_000);

        let metrics = EvmMetrics::for_activity(&activity, 10_000);

        assert_eq!(metrics.planned_value, i64::MAX);
        assert_eq!(metrics.earned_value, i64::MAX);
        // Cost variance saturates rather than overflowing.
        assert_eq!(metrics.cost_variance, i64::MAX);
        assert!(metrics.cost_performance_index.is_some());
    }

    #[test]
    fn resources_default_to_labour_and_keep_their_name() {
        let resource = Resource::new(ResourceId(1), "Crew A");
        assert_eq!(resource.id, ResourceId(1));
        assert_eq!(resource.name, "Crew A");
        assert_eq!(resource.kind, ResourceKind::Labour);
    }
}
