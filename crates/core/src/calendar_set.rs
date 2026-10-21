//! Named calendars.
//!
//! A project has a default [`Calendar`](crate::model::Calendar) plus a set of
//! named calendars. Activities may select a named calendar; when they do not,
//! the project default applies. Named calendar names are unique
//! case-insensitively so that users cannot create confusing duplicates.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::Calendar;

/// Stable identifier for a named calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CalendarId(pub u64);

impl std::fmt::Display for CalendarId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// A calendar with a user-facing name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedCalendar {
    /// Identifier assigned when the calendar was created.
    pub id: CalendarId,
    /// Unique display name (never empty).
    pub name: String,
    /// Working-time definition.
    pub calendar: Calendar,
}

/// Errors produced by named-calendar mutations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CalendarSetError {
    /// The referenced calendar does not exist.
    #[error("calendar {0} does not exist")]
    UnknownCalendar(CalendarId),
    /// A calendar with the same identifier already exists.
    #[error("calendar {0} already exists")]
    DuplicateCalendar(CalendarId),
    /// A calendar with the same name already exists.
    #[error("a calendar named \"{0}\" already exists")]
    DuplicateName(String),
    /// A calendar name was empty or whitespace-only.
    #[error("calendar name must not be empty")]
    EmptyName,
}

/// The named calendars belonging to a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CalendarSet {
    #[serde(default)]
    calendars: Vec<NamedCalendar>,
    #[serde(default)]
    next: u64,
}

impl CalendarSet {
    /// Create an empty calendar set.
    pub fn new() -> Self {
        Self::default()
    }

    /// All named calendars in creation order.
    pub fn calendars(&self) -> &[NamedCalendar] {
        &self.calendars
    }

    /// Number of named calendars.
    pub fn len(&self) -> usize {
        self.calendars.len()
    }

    /// Whether the set has no named calendars.
    pub fn is_empty(&self) -> bool {
        self.calendars.is_empty()
    }

    /// Look up a named-calendar entry.
    pub fn named(&self, id: CalendarId) -> Option<&NamedCalendar> {
        self.calendars.iter().find(|entry| entry.id == id)
    }

    /// Resolve a named calendar's working-time definition.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarSetError::UnknownCalendar`] when `id` is not present.
    pub fn calendar(&self, id: CalendarId) -> Result<&Calendar, CalendarSetError> {
        self.named(id)
            .map(|entry| &entry.calendar)
            .ok_or(CalendarSetError::UnknownCalendar(id))
    }

    /// Next unused calendar identifier.
    pub fn next_id(&self) -> CalendarId {
        let highest = self
            .calendars
            .iter()
            .map(|entry| entry.id.0)
            .max()
            .unwrap_or(0);
        CalendarId(highest.max(self.next.saturating_sub(1)).saturating_add(1))
    }

    /// Add a named calendar.
    ///
    /// # Errors
    ///
    /// Rejects duplicate identifiers, duplicate names (case-insensitive), and
    /// blank names.
    pub fn add(
        &mut self,
        id: CalendarId,
        name: impl Into<String>,
        calendar: Calendar,
    ) -> Result<(), CalendarSetError> {
        if self.named(id).is_some() {
            return Err(CalendarSetError::DuplicateCalendar(id));
        }
        let name = normalise_name(name.into())?;
        if self.name_taken(&name, None) {
            return Err(CalendarSetError::DuplicateName(name));
        }
        self.calendars.push(NamedCalendar { id, name, calendar });
        self.next = self.next.max(id.0.saturating_add(1));
        Ok(())
    }

    /// Rename a calendar, returning the previous name.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarSetError::UnknownCalendar`], [`CalendarSetError::EmptyName`],
    /// or [`CalendarSetError::DuplicateName`].
    pub fn rename(
        &mut self,
        id: CalendarId,
        name: impl Into<String>,
    ) -> Result<String, CalendarSetError> {
        let name = normalise_name(name.into())?;
        if self.name_taken(&name, Some(id)) {
            return Err(CalendarSetError::DuplicateName(name));
        }
        let entry = self
            .calendars
            .iter_mut()
            .find(|entry| entry.id == id)
            .ok_or(CalendarSetError::UnknownCalendar(id))?;
        Ok(std::mem::replace(&mut entry.name, name))
    }

    /// Remove a named calendar, returning it for undo.
    ///
    /// The caller is responsible for checking that no activity still selects
    /// this calendar.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarSetError::UnknownCalendar`] when `id` is not present.
    pub fn remove(&mut self, id: CalendarId) -> Result<NamedCalendar, CalendarSetError> {
        let position = self
            .calendars
            .iter()
            .position(|entry| entry.id == id)
            .ok_or(CalendarSetError::UnknownCalendar(id))?;
        Ok(self.calendars.remove(position))
    }

    /// Restore a removed calendar snapshot at its original position.
    ///
    /// Used by undo. Rejects identifier or name collisions.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarSetError::DuplicateCalendar`] or
    /// [`CalendarSetError::DuplicateName`] when the snapshot cannot be restored.
    pub fn restore(
        &mut self,
        entry: NamedCalendar,
        position: usize,
    ) -> Result<(), CalendarSetError> {
        if self.named(entry.id).is_some() {
            return Err(CalendarSetError::DuplicateCalendar(entry.id));
        }
        if self.name_taken(&entry.name, None) {
            return Err(CalendarSetError::DuplicateName(entry.name));
        }
        let position = position.min(self.calendars.len());
        self.next = self.next.max(entry.id.0.saturating_add(1));
        self.calendars.insert(position, entry);
        Ok(())
    }

    /// Whether a name is already used, ignoring `except`.
    fn name_taken(&self, name: &str, except: Option<CalendarId>) -> bool {
        self.calendars
            .iter()
            .filter(|entry| Some(entry.id) != except)
            .any(|entry| entry.name.eq_ignore_ascii_case(name))
    }
}

fn normalise_name(name: String) -> Result<String, CalendarSetError> {
    let name = name.trim();
    if name.is_empty() {
        Err(CalendarSetError::EmptyName)
    } else {
        Ok(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn six_day_week() -> Calendar {
        Calendar::new([true, true, true, true, true, true, false]).expect("valid calendar")
    }

    #[test]
    fn add_and_lookup_calendar() {
        let mut set = CalendarSet::new();
        set.add(CalendarId(1), "Six day", six_day_week())
            .expect("add calendar");
        assert_eq!(set.len(), 1);
        assert_eq!(set.next_id(), CalendarId(2));
        assert_eq!(
            set.calendar(CalendarId(1))
                .expect("calendar")
                .working_days_per_week(),
            6
        );
        assert_eq!(set.named(CalendarId(1)).expect("named").name, "Six day");
    }

    #[test]
    fn add_rejects_duplicates_and_blank_names() {
        let mut set = CalendarSet::new();
        set.add(CalendarId(1), "Site", Calendar::weekend())
            .expect("add");
        assert_eq!(
            set.add(CalendarId(1), "Other", Calendar::weekend()),
            Err(CalendarSetError::DuplicateCalendar(CalendarId(1)))
        );
        assert_eq!(
            set.add(CalendarId(2), "site", Calendar::weekend()),
            Err(CalendarSetError::DuplicateName("site".to_string()))
        );
        assert_eq!(
            set.add(CalendarId(3), "  ", Calendar::weekend()),
            Err(CalendarSetError::EmptyName)
        );
    }

    #[test]
    fn rename_returns_previous_name_and_rejects_collision() {
        let mut set = CalendarSet::new();
        set.add(CalendarId(1), "Site", Calendar::weekend())
            .expect("add");
        set.add(CalendarId(2), "Office", Calendar::weekend())
            .expect("add");
        assert_eq!(set.rename(CalendarId(1), "Field").expect("rename"), "Site");
        assert_eq!(
            set.rename(CalendarId(1), "office"),
            Err(CalendarSetError::DuplicateName("office".to_string()))
        );
        assert_eq!(
            set.rename(CalendarId(1), "Site").expect("rename back"),
            "Field"
        );
    }

    #[test]
    fn rename_to_own_name_is_allowed() {
        let mut set = CalendarSet::new();
        set.add(CalendarId(1), "Site", Calendar::weekend())
            .expect("add");
        assert_eq!(set.rename(CalendarId(1), "Site").expect("rename"), "Site");
    }

    #[test]
    fn remove_returns_entry_and_unknown_is_error() {
        let mut set = CalendarSet::new();
        set.add(CalendarId(1), "Site", Calendar::weekend())
            .expect("add");
        let removed = set.remove(CalendarId(1)).expect("remove");
        assert_eq!(removed.id, CalendarId(1));
        assert!(set.is_empty());
        assert_eq!(
            set.remove(CalendarId(1)),
            Err(CalendarSetError::UnknownCalendar(CalendarId(1)))
        );
    }

    #[test]
    fn next_id_survives_serde_roundtrip() {
        let mut set = CalendarSet::new();
        set.add(CalendarId(7), "Site", Calendar::weekend())
            .expect("add");
        let json = serde_json::to_string(&set).expect("serialize");
        let restored: CalendarSet = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.next_id(), CalendarId(8));
        assert_eq!(restored.calendars().len(), 1);
    }
}
