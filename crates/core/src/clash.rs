//! Light location clash detection.
//!
//! Construction work is planned against physical locations — a floor, a room,
//! a zone — and two crews cannot usually work the same location on the same
//! day. This module flags that case: two activities that carry the same
//! free-text location and are scheduled to work on at least one common day.
//!
//! It is a light, dependency-free check, not a GIS layer: locations are
//! optional free text and compared case-insensitively after trimming. See
//! `docs/research/aphex-integration.md` for the public workflow behind it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::Date;

use crate::engine::Schedule;
use crate::model::{ActivityId, Project};

/// Two activities planned to work the same location on the same day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationClash {
    /// The shared location, as first written.
    pub location: String,
    /// First day both activities are scheduled to work.
    pub date: Date,
    /// Last day both activities are scheduled to work.
    pub last_date: Date,
    /// One activity in the clash.
    pub first: ActivityId,
    /// Name of the first activity.
    pub first_name: String,
    /// The other activity in the clash.
    pub second: ActivityId,
    /// Name of the second activity.
    pub second_name: String,
}

/// Find location clashes: pairs of open activities that share a location and
/// overlap on at least one working day.
///
/// The result is sorted by location then first day, and each unordered pair is
/// reported at most once. A missing location never clashes.
pub fn location_clashes(project: &Project, schedule: &Schedule) -> Vec<LocationClash> {
    // Group by the normalised location so only same-location activities are
    // compared.
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, activity) in project.activities.iter().enumerate() {
        let Some(location) = activity
            .location
            .as_deref()
            .map(str::trim)
            .filter(|location| !location.is_empty())
        else {
            continue;
        };
        groups
            .entry(location.to_lowercase())
            .or_default()
            .push(index);
    }

    let mut clashes = Vec::new();
    for indices in groups.values() {
        for (position, &left_index) in indices.iter().enumerate() {
            for &right_index in &indices[position + 1..] {
                let left = &project.activities[left_index];
                let right = &project.activities[right_index];
                let (Some(left_schedule), Some(right_schedule)) = (
                    schedule.activity(left.id),
                    schedule.activity(right.id),
                ) else {
                    continue;
                };
                // Skip finished work: a clash is about planned or ongoing work.
                if !crate::lookahead::is_open(left) || !crate::lookahead::is_open(right) {
                    continue;
                }
                let overlap_start = left_schedule
                    .scheduled_start
                    .max(right_schedule.scheduled_start);
                let overlap_end = left_schedule.scheduled_finish.min(right_schedule.scheduled_finish);
                if overlap_start > overlap_end {
                    continue;
                }
                clashes.push(LocationClash {
                    location: left.location.clone().unwrap_or_default(),
                    date: overlap_start,
                    last_date: overlap_end,
                    first: left.id,
                    first_name: left.name.clone(),
                    second: right.id,
                    second_name: right.name.clone(),
                });
            }
        }
    }

    clashes.sort_by(|a, b| {
        a.location
            .to_lowercase()
            .cmp(&b.location.to_lowercase())
            .then(a.date.cmp(&b.date))
            .then(a.first.cmp(&b.first))
            .then(a.second.cmp(&b.second))
    });
    clashes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::schedule;
    use crate::model::{Activity, RelationshipId};
    use time::Month;

    fn jan(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::January, day).expect("valid test date")
    }

    /// Start -> A (3d) -> B (3d), all in sequence.
    fn chain() -> Project {
        let mut project = Project::new("Clash test", jan(5));
        for (id, name, duration) in [(1u64, "A", 3u32), (2, "B", 3)] {
            project
                .activities
                .push(Activity::new(ActivityId::new(id), name, duration));
        }
        project.relationships.push(crate::model::Relationship::new(
            RelationshipId::new(1),
            ActivityId::new(1),
            ActivityId::new(2),
            crate::model::LinkType::Fs,
            0,
        ));
        project
    }

    #[test]
    fn activities_in_the_same_location_but_different_days_do_not_clash() {
        let mut project = chain();
        project.activities[0].location = Some("Level 1".to_string());
        project.activities[1].location = Some("Level 1".to_string());

        let schedule = schedule(&project).expect("schedule succeeds");
        assert!(location_clashes(&project, &schedule).is_empty());
    }

    #[test]
    fn overlapping_activities_in_the_same_location_clash() {
        let mut project = chain();
        // Run B in parallel with A by giving it no predecessor.
        project.relationships.clear();
        project.activities[0].location = Some("Level 1".to_string());
        project.activities[1].location = Some("level 1 ".to_string());

        let schedule = schedule(&project).expect("schedule succeeds");
        let clashes = location_clashes(&project, &schedule);
        assert_eq!(clashes.len(), 1);
        assert_eq!(clashes[0].date, jan(5));
    }

    #[test]
    fn different_locations_do_not_clash() {
        let mut project = chain();
        project.relationships.clear();
        project.activities[0].location = Some("Level 1".to_string());
        project.activities[1].location = Some("Level 2".to_string());

        let schedule = schedule(&project).expect("schedule succeeds");
        assert!(location_clashes(&project, &schedule).is_empty());
    }

    #[test]
    fn a_missing_location_never_clashes() {
        let mut project = chain();
        project.relationships.clear();
        project.activities[0].location = Some("Level 1".to_string());
        project.activities[1].location = None;

        let schedule = schedule(&project).expect("schedule succeeds");
        assert!(location_clashes(&project, &schedule).is_empty());
    }
}
