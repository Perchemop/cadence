//! Lookahead planning: the rolling short-interval window over a scheduled
//! programme.
//!
//! Construction delivery teams work from a *lookahead* — a rolling window of
//! the near future (typically a few weeks) extracted from the master
//! schedule, so the coming work can be coordinated, resourced and promised
//! without wading through the whole programme. This module extracts that
//! window: the activities whose scheduled span intersects the window, in
//! scheduled-start order. It is a pure view over an existing schedule — no
//! duplicate data, no second engine (see
//! `docs/research/aphex-integration.md` for the public sources behind the
//! workflow).
//!
//! The window is measured in **calendar weeks** from a start date and
//! normalised onto the project calendar: an activity is in the window when it
//! is scheduled to work on any day between the window start (inclusive) and
//! the window end (exclusive), where the end is `weeks` calendar weeks after
//! the start. Activities already finished before the window never appear;
//! activities spanning the window appear once.

use serde::{Deserialize, Serialize};
use time::Date;

use crate::engine::Schedule;
use crate::model::{Activity, ActivityId, Project};

/// One activity in the lookahead window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookaheadEntry {
    /// Activity identifier.
    pub id: ActivityId,
    /// Activity name, copied for display.
    pub name: String,
    /// Scheduled start.
    pub scheduled_start: Date,
    /// Scheduled finish.
    pub scheduled_finish: Date,
    /// True when the activity is on the critical set.
    pub critical: bool,
    /// True when the activity has already started by the window start
    /// (started before the window begins but still working inside it).
    pub in_progress: bool,
    /// The field-planning owner, copied for display.
    pub owner: Option<String>,
    /// True when the activity is a promised near-term task.
    pub promised: bool,
}

/// Extract the lookahead window: activities scheduled to work between `from`
/// (inclusive) and `from` plus `weeks` calendar weeks (exclusive), in
/// scheduled-start then identifier order.
///
/// An unschedulable project yields an empty window; callers that need to
/// distinguish that from a genuinely empty window can check
/// [`Schedule::activity`] themselves or schedule first.
pub fn lookahead(
    project: &Project,
    schedule: &Schedule,
    from: Date,
    weeks: u32,
) -> Vec<LookaheadEntry> {
    let calendar = &project.calendar;
    let window_start = calendar.next_working_day_on_or_after(from);
    let window_end = from
        .checked_add(time::Duration::weeks(i64::from(weeks)))
        .unwrap_or(from);
    let Ok(window_start) = window_start else {
        return Vec::new();
    };

    let mut entries: Vec<LookaheadEntry> = project
        .activities
        .iter()
        .filter_map(|activity| {
            let scheduled = schedule.activity(activity.id)?;
            let starts_in_or_before = scheduled.scheduled_start < window_end;
            let finishes_on_or_after = scheduled.scheduled_finish >= window_start;
            (starts_in_or_before && finishes_on_or_after).then(|| LookaheadEntry {
                id: activity.id,
                name: activity.name.clone(),
                scheduled_start: scheduled.scheduled_start,
                scheduled_finish: scheduled.scheduled_finish,
                critical: scheduled.critical,
                in_progress: scheduled.scheduled_start < window_start,
                owner: activity.owner.clone(),
                promised: activity.promised,
            })
        })
        .collect();
    entries.sort_by_key(|entry| (entry.scheduled_start, entry.id));
    entries
}

/// True when the activity is open (not finished) — the lookahead is about
/// upcoming and ongoing work, so callers can filter out finished work.
pub fn is_open(activity: &Activity) -> bool {
    activity.actual_finish.is_none() && activity.percent_complete.unwrap_or(0) < 10_000
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::schedule;
    use crate::model::RelationshipId;
    use time::Month;

    fn jan(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::January, day).expect("valid test date")
    }

    /// Start (0d) -> A (3d) -> B (3d) -> C (10d) -> Finish (0d), all critical.
    fn chain() -> Project {
        let mut project = Project::new("Lookahead test", jan(5));
        for (id, name, duration) in [
            (1u64, "Start", 0u32),
            (2, "A", 3),
            (3, "B", 3),
            (4, "C", 10),
            (5, "Finish", 0),
        ] {
            project
                .activities
                .push(Activity::new(ActivityId::new(id), name, duration));
        }
        for (id, predecessor, successor) in [(1u64, 1u64, 2u64), (2, 2, 3), (3, 3, 4), (4, 4, 5)] {
            project.relationships.push(crate::model::Relationship::new(
                RelationshipId::new(id),
                ActivityId::new(predecessor),
                ActivityId::new(successor),
                crate::model::LinkType::Fs,
                0,
            ));
        }
        project
    }

    fn scheduled(project: &Project) -> Schedule {
        schedule(project).expect("schedule succeeds")
    }

    #[test]
    fn a_two_week_window_contains_only_the_near_work() {
        let project = chain();
        let schedule = scheduled(&project);

        let entries = lookahead(&project, &schedule, jan(5), 2);

        // Two weeks from Mon 5 Jan ends Mon 19 Jan: Start, A, B and the start
        // of C are inside; C finishes 10 working days after B, well after.
        let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, vec!["Start", "A", "B", "C"]);
        assert!(entries.iter().all(|entry| entry.critical));
    }

    #[test]
    fn a_window_past_the_programme_is_empty() {
        let project = chain();
        let schedule = scheduled(&project);

        let entries = lookahead(&project, &schedule, jan(5) + time::Duration::weeks(30), 2);

        assert!(entries.is_empty());
    }

    #[test]
    fn activities_started_before_the_window_are_marked_in_progress() {
        let project = chain();
        let schedule = scheduled(&project);

        // C spans jan13-26; a one-week window from jan19 (ending jan26)
        // contains C alone, already in progress. The finish milestone (jan27)
        // starts on the window's end day and is correctly excluded.
        let entries = lookahead(&project, &schedule, jan(19), 1);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "C");
        assert!(entries[0].in_progress);
    }

    #[test]
    fn the_window_is_deterministic_and_sorted_by_start() {
        let project = chain();
        let schedule = scheduled(&project);

        let first = lookahead(&project, &schedule, jan(5), 4);
        let second = lookahead(&project, &schedule, jan(5), 4);

        assert_eq!(first, second);
        let starts: Vec<Date> = first.iter().map(|entry| entry.scheduled_start).collect();
        let mut sorted = starts.clone();
        sorted.sort();
        assert_eq!(starts, sorted);
    }

    #[test]
    fn an_unschedulable_programme_yields_an_empty_window() {
        let mut project = chain();
        // A cycle makes the project unschedulable.
        project.relationships.push(crate::model::Relationship::new(
            RelationshipId::new(9),
            ActivityId::new(2),
            ActivityId::new(1),
            crate::model::LinkType::Fs,
            0,
        ));
        let broken = schedule(&project);
        assert!(broken.is_err(), "the fixture is cyclic");
        // The caller passes whatever schedule it has; with none there is no
        // window, so the empty result is the documented behaviour.
        let schedule = Schedule {
            project_start: jan(5),
            project_finish: jan(5),
            activities: Vec::new(),
            critical_path: Vec::new(),
        };
        let entries = lookahead(&project, &schedule, jan(5), 2);
        assert!(entries.is_empty());
        let _ = broken;
    }
}
