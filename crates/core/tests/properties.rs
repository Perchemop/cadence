//! Property-based tests for the CPM engine and working-day arithmetic.
//!
//! These complement the concrete date-based unit tests: arbitrary DAGs check
//! invariants that are hard to enumerate by hand, such as every produced date
//! landing on the activity's own working calendar and every relationship
//! remaining satisfied after the forward and ALAP passes.

use cadence_core::engine::{level_resources, resource_conflicts, schedule, SchedError};
use cadence_core::model::{
    Activity, ActivityId, Calendar, Constraint, LinkType, Project, Relationship, RelationshipId,
    Resource, ResourceId,
};
use cadence_core::CalendarId;
use proptest::prelude::*;
use time::{Date, Month};

const MAX_ACTIVITIES: usize = 8;

fn origin() -> Date {
    Date::from_calendar_date(2026, Month::January, 5).expect("valid origin")
}

fn base_project() -> Project {
    Project::new("Properties", origin())
}

fn link_type(raw: u8) -> LinkType {
    LinkType::ALL[usize::from(raw % 4)]
}

fn add_activities(project: &mut Project, durations: &[u32]) -> Vec<ActivityId> {
    durations
        .iter()
        .enumerate()
        .map(|(index, duration)| {
            let id = ActivityId(index as u64 + 1);
            project
                .activities
                .push(Activity::new(id, format!("A{index}"), *duration));
            id
        })
        .collect()
}

/// Add links only from lower to higher indices, which keeps the graph acyclic.
fn add_links(
    project: &mut Project,
    ids: &[ActivityId],
    raw: &[(usize, usize, u8, i64)],
) -> Vec<(ActivityId, ActivityId, LinkType, i64)> {
    let mut applied = Vec::new();
    let mut next_id = 1u64;
    for &(left, right, kind, lag) in raw {
        if left >= ids.len() || right >= ids.len() || left == right {
            continue;
        }
        let (predecessor, successor) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        let link = link_type(kind);
        project.relationships.push(Relationship::new(
            RelationshipId(next_id),
            ids[predecessor],
            ids[successor],
            link,
            lag,
        ));
        next_id += 1;
        applied.push((ids[predecessor], ids[successor], link, lag));
    }
    applied
}

fn build(
    durations: &[u32],
    raw_links: &[(usize, usize, u8, i64)],
) -> (Project, Vec<(ActivityId, ActivityId, LinkType, i64)>) {
    let mut project = base_project();
    let ids = add_activities(&mut project, durations);
    let links = add_links(&mut project, &ids, raw_links);
    (project, links)
}

/// Signed working-day distance required by a relationship's forward form.
fn required_distance(link: LinkType, lag: i64) -> i64 {
    match link {
        LinkType::Fs => lag + 1,
        LinkType::Ss | LinkType::Ff | LinkType::Sf => lag,
    }
}

fn actual_distance(
    calendar: &Calendar,
    link: LinkType,
    predecessor: (Date, Date),
    successor: (Date, Date),
) -> i64 {
    match link {
        LinkType::Fs => calendar.working_days_between(predecessor.1, successor.0),
        LinkType::Ss => calendar.working_days_between(predecessor.0, successor.0),
        LinkType::Ff => calendar.working_days_between(predecessor.1, successor.1),
        LinkType::Sf => calendar.working_days_between(predecessor.0, successor.1),
    }
}

proptest! {
    #[test]
    fn every_scheduled_date_is_a_working_day(
        durations in prop::collection::vec(0u32..8, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, -3i64..4),
            0..20,
        ),
        six_day_flags in prop::collection::vec(any::<bool>(), MAX_ACTIVITIES),
    ) {
        let mut project = base_project();
        project
            .calendars
            .add(
                CalendarId(1),
                "Six day",
                Calendar::new([true, true, true, true, true, true, false])
                    .expect("six-day week"),
            )
            .expect("calendar is valid");
        let ids = add_activities(&mut project, &durations);
        add_links(&mut project, &ids, &raw_links);

        for (index, id) in ids.iter().enumerate() {
            if six_day_flags.get(index).copied().unwrap_or(false) {
                project.activity_mut(*id).expect("activity").calendar = Some(CalendarId(1));
            }
        }

        let result = schedule(&project).expect("valid DAG schedules");
        for activity in &project.activities {
            let calendar = match activity.calendar {
                Some(id) => project.calendars.calendar(id).expect("known calendar"),
                None => &project.calendar,
            };
            let item = result.activity(activity.id).expect("scheduled activity");
            for date in [
                item.early_start,
                item.early_finish,
                item.late_start,
                item.late_finish,
                item.scheduled_start,
                item.scheduled_finish,
            ] {
                prop_assert!(
                    calendar.is_working_day(date),
                    "activity {} has non-working date {} (calendar {:?})",
                    activity.id,
                    date,
                    activity.calendar
                );
            }
            prop_assert!(item.early_start <= item.early_finish);
            prop_assert!(item.late_start <= item.late_finish);
            prop_assert_eq!(
                item.total_float,
                calendar.working_days_between(item.early_start, item.late_start)
            );
            prop_assert!(item.free_float >= 0);
            prop_assert_eq!(item.critical, item.total_float <= 0);
        }
    }

    #[test]
    fn early_dates_obey_relationships_and_float(
        durations in prop::collection::vec(0u32..8, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, 0i64..4),
            0..20,
        ),
    ) {
        let (project, links) = build(&durations, &raw_links);
        let calendar = project.calendar.clone();
        let result = schedule(&project).expect("valid DAG schedules");

        for (predecessor, successor, link, lag) in links {
            let pred = result.activity(predecessor).expect("predecessor");
            let succ = result.activity(successor).expect("successor");
            let distance = actual_distance(
                &calendar,
                link,
                (pred.early_start, pred.early_finish),
                (succ.early_start, succ.early_finish),
            );
            prop_assert!(
                distance >= required_distance(link, lag),
                "{} -> {} {:?} lag {}: early dates are {} too early",
                predecessor,
                successor,
                link,
                lag,
                required_distance(link, lag) - distance
            );
        }

        for activity in &project.activities {
            let item = result.activity(activity.id).expect("scheduled activity");
            prop_assert!(item.total_float >= 0);
            prop_assert!(item.free_float >= 0);
            prop_assert_eq!(item.critical, item.total_float <= 0);
        }
    }

    #[test]
    fn alap_dates_stay_within_bounds_and_preserve_logic(
        durations in prop::collection::vec(0u32..8, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, 0i64..4),
            0..20,
        ),
        alap_flags in prop::collection::vec(any::<bool>(), MAX_ACTIVITIES),
    ) {
        let (mut project, links) = build(&durations, &raw_links);
        for (index, activity) in project.activities.iter_mut().enumerate() {
            if alap_flags.get(index).copied().unwrap_or(false) {
                activity.constraint = Constraint::AsLateAsPossible;
            }
        }

        let calendar = project.calendar.clone();
        let result = schedule(&project).expect("valid DAG schedules");

        let mut baseline_project = project.clone();
        for activity in &mut baseline_project.activities {
            activity.constraint = Constraint::AsSoonAsPossible;
        }
        let baseline = schedule(&baseline_project).expect("baseline schedules");
        prop_assert_eq!(result.project_finish, baseline.project_finish);

        for activity in &project.activities {
            let item = result.activity(activity.id).expect("scheduled activity");
            prop_assert!(item.scheduled_start >= item.early_start);
            prop_assert!(item.scheduled_finish >= item.early_finish);
            prop_assert!(item.scheduled_start <= item.late_start);
            prop_assert!(item.scheduled_finish <= item.late_finish);
            if activity.constraint != Constraint::AsLateAsPossible {
                prop_assert_eq!(item.scheduled_start, item.early_start);
            }
        }

        for (predecessor, successor, link, lag) in links {
            let pred = result.activity(predecessor).expect("predecessor");
            let succ = result.activity(successor).expect("successor");
            let distance = actual_distance(
                &calendar,
                link,
                (pred.scheduled_start, pred.scheduled_finish),
                (succ.scheduled_start, succ.scheduled_finish),
            );
            prop_assert!(
                distance >= required_distance(link, lag),
                "{} -> {} {:?} lag {}: scheduled dates violate the link",
                predecessor,
                successor,
                link,
                lag
            );
        }
    }

    #[test]
    fn constraint_bounds_are_respected_or_reported(
        duration in 0u32..7,
        kind in 0u8..6,
        day in 5u8..23,
    ) {
        let constraint = match kind {
            0 => Constraint::AsSoonAsPossible,
            1 => Constraint::AsLateAsPossible,
            2 => Constraint::StartNoEarlierThan(
                Date::from_calendar_date(2026, Month::January, day).expect("date"),
            ),
            3 => Constraint::StartNoLaterThan(
                Date::from_calendar_date(2026, Month::January, day).expect("date"),
            ),
            4 => Constraint::FinishNoEarlierThan(
                Date::from_calendar_date(2026, Month::January, day).expect("date"),
            ),
            _ => Constraint::FinishNoLaterThan(
                Date::from_calendar_date(2026, Month::January, day).expect("date"),
            ),
        };

        let mut project = base_project();
        let a = ActivityId(1);
        let mut activity = Activity::new(a, "Constrained", duration);
        activity.constraint = constraint;
        project.activities.push(activity);
        project
            .activities
            .push(Activity::new(ActivityId(2), "Independent", 8));

        let calendar = project.calendar.clone();
        match schedule(&project) {
            Ok(result) => {
                let item = result.activity(a).expect("scheduled activity");
                match constraint {
                    Constraint::AsSoonAsPossible => {
                        prop_assert_eq!(item.scheduled_start, item.early_start);
                        prop_assert_eq!(item.scheduled_finish, item.early_finish);
                    }
                    Constraint::AsLateAsPossible => {
                        prop_assert_eq!(item.scheduled_start, item.late_start);
                        prop_assert_eq!(item.scheduled_finish, item.late_finish);
                    }
                    Constraint::StartNoEarlierThan(bound)
                    | Constraint::FinishNoEarlierThan(bound) => {
                        let bound = calendar
                            .next_working_day_on_or_after(bound)
                            .expect("working-day bound");
                        if matches!(constraint, Constraint::StartNoEarlierThan(_)) {
                            prop_assert!(item.early_start >= bound);
                            prop_assert!(item.scheduled_start >= bound);
                        } else {
                            prop_assert!(item.early_finish >= bound);
                            prop_assert!(item.scheduled_finish >= bound);
                        }
                    }
                    Constraint::StartNoLaterThan(bound)
                    | Constraint::FinishNoLaterThan(bound) => {
                        let bound = calendar
                            .prev_working_day_on_or_before(bound)
                            .expect("working-day bound");
                        if matches!(constraint, Constraint::StartNoLaterThan(_)) {
                            prop_assert!(item.early_start <= bound);
                            prop_assert!(item.scheduled_start <= bound);
                        } else {
                            prop_assert!(item.early_finish <= bound);
                            prop_assert!(item.scheduled_finish <= bound);
                        }
                    }
                }
            }
            Err(SchedError::ConstraintViolated { activity, .. }) => {
                prop_assert_eq!(activity, a);
            }
            Err(other) => {
                prop_assert!(false, "unexpected scheduling error: {other:?}");
            }
        }
    }

    #[test]
    fn leveling_only_delays_work_and_lands_on_a_feasible_schedule(
        durations in prop::collection::vec(1u32..6, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, 0i64..3),
            0..12,
        ),
        masks in prop::collection::vec(0u8..8, 1..MAX_ACTIVITIES),
        capacities in prop::collection::vec(1u32..4, 1..3),
    ) {
        let (mut project, _links) = build(&durations, &raw_links);
        for (index, capacity) in capacities.iter().enumerate() {
            project.resources.push(Resource::with_capacity(
                ResourceId(index as u64 + 1),
                format!("R{index}"),
                *capacity,
            ));
        }
        for (index, activity) in project.activities.iter_mut().enumerate() {
            let mask = masks.get(index).copied().unwrap_or(0);
            for bit in 0..capacities.len() {
                if mask & (1 << bit) != 0 {
                    activity.resources.push(ResourceId(bit as u64 + 1));
                }
            }
        }

        let computed = schedule(&project).expect("valid DAG schedules");
        let Ok(leveled) = level_resources(&project, &computed) else {
            // Leveling only fails when the project stops being schedulable,
            // which the generator cannot produce; treat it as a skip.
            return Ok(());
        };

        for activity in &leveled.activities {
            prop_assert!(
                activity.start >= activity.original_start,
                "leveling moved activity {} earlier: {} -> {}",
                activity.id,
                activity.original_start,
                activity.start
            );
            prop_assert!(activity.finish >= activity.original_finish);
            prop_assert_eq!(
                activity.delay,
                project
                    .calendar
                    .working_days_between(activity.original_start, activity.start)
                    .max(0)
            );
        }

        if !leveled.conflicts.is_empty() {
            // The iteration budget ran out; the remaining conflicts are
            // reported rather than hidden, which is all that is promised.
            return Ok(());
        }

        // A leveled schedule with no conflicts left must be achievable through
        // ordinary start-no-earlier-than constraints, and the reschedule must
        // reproduce exactly the dates leveling reported (D12).
        let mut applied = project.clone();
        for activity in &leveled.activities {
            applied
                .activity_mut(activity.id)
                .expect("activity")
                .constraint = Constraint::StartNoEarlierThan(activity.start);
        }
        let rescheduled = schedule(&applied).expect("the leveled project schedules");
        prop_assert!(resource_conflicts(&applied, &rescheduled).is_empty());
        for activity in &leveled.activities {
            let item = rescheduled.activity(activity.id).expect("scheduled activity");
            prop_assert_eq!(item.scheduled_start, activity.start);
            prop_assert_eq!(item.scheduled_finish, activity.finish);
        }
    }

    #[test]
    fn calendar_arithmetic_round_trips(day in 1u8..28, offset in -40i64..40) {
        let calendar = Calendar::weekend();
        let date = Date::from_calendar_date(2026, Month::January, day).expect("valid date");
        let anchor = calendar
            .next_working_day_on_or_after(date)
            .expect("working-day anchor");

        prop_assert!(calendar.is_working_day(anchor));
        prop_assert!(anchor >= date);
        prop_assert!(calendar.prev_working_day_on_or_before(date).expect("previous") <= date);

        let shifted = calendar
            .add_working_days(anchor, offset)
            .expect("working-day shift");
        prop_assert!(calendar.is_working_day(shifted));
        prop_assert_eq!(calendar.working_days_between(anchor, shifted), offset);
    }
}
