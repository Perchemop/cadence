//! Critical Path Method engine.
//!
//! The engine is a pure function of a [`Project`]: it validates the
//! relationship graph, performs a forward pass (earliest dates), a backward
//! pass (latest dates), and derives total float, free float, and the critical
//! activities. It never mutates its input and never blocks or hangs on a
//! cyclic graph.
//!
//! # Relationship semantics
//!
//! Let `ES`/`EF` be the early start/finish of the predecessor and `dur` the
//! successor duration, all in working days:
//!
//! | Type | Forward constraint |
//! |------|--------------------|
//! | FS   | `ES_succ >= EF_pred + 1 + lag` |
//! | SS   | `ES_succ >= ES_pred + lag` |
//! | FF   | `EF_succ >= EF_pred + lag` |
//! | SF   | `EF_succ >= ES_pred + lag` |
//!
//! The `+ 1` in FS means a zero-lag finish-to-start relationship consumes the
//! next working day, the conventional CPM interpretation. Negative lags are
//! leads. Durations are inclusive: an activity of duration 1 starting on Monday
//! finishes on Monday, and a duration of 0 is a milestone.
//!
//! # Multiple calendars
//!
//! Each activity may select a named calendar and otherwise inherits the project
//! default. When a relationship crosses calendars, the **successor's** calendar
//! governs the forward pass constraint (when the successor may start or
//! finish), while the **predecessor's** calendar governs the backward pass
//! (how late it may finish). This keeps every activity's dates on its own
//! working days. See `DECISIONS.md` D6.
//!
//! # Constraints
//!
//! Each activity carries a [`Constraint`]. Date constraints are inclusive
//! bounds on the activity's own calendar: a lower bound moves forward to the
//! next working day and an upper bound moves backward to the previous working
//! day. `StartNoEarlierThan` and `FinishNoEarlierThan` raise the forward-pass
//! dates; `StartNoLaterThan` and `FinishNoLaterThan` cap the backward-pass
//! dates. An upper bound that the earliest feasible schedule already misses is
//! a [`SchedError::ConstraintViolated`], not negative float by stealth.
//!
//! [`ActivitySchedule`] always exposes the CPM bounds (`early_*`, `late_*`)
//! and additionally reports `scheduled_*`. ASAP activities report their early
//! dates. ALAP activities report their late dates, pulled back only where a
//! successor's scheduled date would otherwise be violated; precedence always
//! wins over the ALAP preference.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Date, Duration};

use crate::calendar_set::CalendarId;
use crate::model::{
    ActivityBaselineSnapshot, ActivityId, Baseline, BaselineId, Calendar, CalendarError,
    Constraint, EvmMetrics, LinkType, Project, RelationshipId, ResourceId,
};
use crate::wbs::WbsId;

/// Errors returned by [`schedule`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SchedError {
    /// Two activities share an identifier.
    #[error("duplicate activity id {0}")]
    DuplicateActivity(ActivityId),
    /// Two relationships share an identifier.
    #[error("duplicate relationship id {0}")]
    DuplicateRelationship(RelationshipId),
    /// A relationship refers to an activity that is not in the project.
    #[error("relationship {relationship} references unknown activity {activity}")]
    UnknownActivity {
        /// Relationship containing the bad reference.
        relationship: RelationshipId,
        /// Missing activity identifier.
        activity: ActivityId,
    },
    /// The relationship graph contains a cycle.
    #[error("relationship cycle detected: {path:?}")]
    Cycle {
        /// Cycle path, with the repeated starting activity at both ends.
        path: Vec<ActivityId>,
    },
    /// Lag arithmetic overflowed the supported integer range.
    #[error("arithmetic overflow while scheduling")]
    ArithmeticOverflow,
    /// The project calendar could not perform the requested arithmetic.
    #[error(transparent)]
    Calendar(#[from] CalendarError),
    /// An activity selects a named calendar that does not exist.
    #[error("activity {activity} selects unknown calendar {calendar}")]
    UnknownCalendar {
        /// Activity with the bad selection.
        activity: ActivityId,
        /// Missing calendar identifier.
        calendar: CalendarId,
    },
    /// An activity references a WBS node that does not exist.
    #[error("activity {activity} references unknown WBS node {wbs}")]
    UnknownWbs {
        /// Activity with the bad reference.
        activity: ActivityId,
        /// Missing WBS node identifier.
        wbs: WbsId,
    },
    /// A hard upper-bound constraint cannot be met by the earliest schedule.
    #[error("activity {activity} cannot satisfy its constraint: {constraint}")]
    ConstraintViolated {
        /// Activity carrying the unsatisfiable constraint.
        activity: ActivityId,
        /// Constraint that could not be met.
        constraint: Constraint,
    },
}

/// Scheduled dates and float for one activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivitySchedule {
    /// Activity identifier.
    pub id: ActivityId,
    /// Earliest start produced by the forward pass.
    pub early_start: Date,
    /// Earliest finish produced by the forward pass.
    pub early_finish: Date,
    /// Latest start produced by the backward pass.
    pub late_start: Date,
    /// Latest finish produced by the backward pass.
    pub late_finish: Date,
    /// Start date used for reporting: early for ASAP, late for ALAP.
    pub scheduled_start: Date,
    /// Finish date used for reporting: early for ASAP, late for ALAP.
    pub scheduled_finish: Date,
    /// Total float in working days (`late_start - early_start`).
    pub total_float: i64,
    /// Free float in working days; equals total float for terminal activities.
    pub free_float: i64,
    /// True when total float is zero or negative.
    pub critical: bool,
}

/// Result of a complete CPM calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    /// First working day of the project.
    pub project_start: Date,
    /// Latest early finish in the project.
    pub project_finish: Date,
    /// One result per activity, in project order.
    pub activities: Vec<ActivitySchedule>,
    /// Critical activities ordered by early start, then identifier.
    pub critical_path: Vec<ActivityId>,
}

impl Schedule {
    /// Look up a scheduled activity by identifier.
    pub fn activity(&self, id: ActivityId) -> Option<&ActivitySchedule> {
        self.activities.iter().find(|activity| activity.id == id)
    }

    /// Copy this schedule into the optional cached fields on [`Project`]
    /// activities.
    pub fn apply_to(&self, project: &mut Project) {
        let positions: HashMap<ActivityId, usize> = project
            .activities
            .iter()
            .enumerate()
            .map(|(position, activity)| (activity.id, position))
            .collect();

        for result in &self.activities {
            if let Some(&position) = positions.get(&result.id) {
                let activity = &mut project.activities[position];
                activity.early_start = Some(result.early_start);
                activity.early_finish = Some(result.early_finish);
                activity.late_start = Some(result.late_start);
                activity.late_finish = Some(result.late_finish);
                activity.scheduled_start = Some(result.scheduled_start);
                activity.scheduled_finish = Some(result.scheduled_finish);
                activity.total_float = Some(result.total_float);
                activity.free_float = Some(result.free_float);
                activity.critical = result.critical;
            }
        }
    }
}

/// A single resource unit assigned to an activity on a given day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAssignment {
    /// Resource identifier.
    pub resource_id: ResourceId,
    /// Activity identifier using the resource.
    pub activity_id: ActivityId,
}

/// Resource histogram: for each working day in the project window, the set of
/// (resource, activity) pairs actively consuming that resource.
///
/// Each activity with assigned resources contributes one entry per resource for
/// every working day between its scheduled start and scheduled finish
/// (inclusive). Activities with no scheduled dates or no resources contribute
/// nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceHistogram {
    /// Working days to resource assignments active on that day, in chronological
    /// order.
    pub entries: BTreeMap<Date, Vec<ResourceAssignment>>,
}

impl ResourceHistogram {
    /// Number of distinct working days with at least one resource active.
    pub fn days_with_usage(&self) -> usize {
        self.entries.len()
    }

    /// Resources active on a given date, in insertion order.
    pub fn assignments_on(&self, date: Date) -> Option<&Vec<ResourceAssignment>> {
        self.entries.get(&date)
    }
}

/// Build a resource histogram from a schedule and project.
///
/// For every activity that has scheduled dates and at least one assigned
/// resource, emit an entry for each working day between its scheduled start and
/// scheduled finish (inclusive) under the activity's own calendar.
pub fn resource_histogram(project: &Project, schedule: &Schedule) -> ResourceHistogram {
    let mut histogram = ResourceHistogram::default();

    for result in &schedule.activities {
        let Some(activity) = project.activity(result.id) else {
            continue;
        };
        if activity.resources.is_empty() {
            continue;
        }

        let calendar = match activity.calendar {
            None => &project.calendar,
            Some(id) => project.calendars.calendar(id).unwrap_or(&project.calendar),
        };

        let mut day = result.scheduled_start;
        let finish = result.scheduled_finish;
        while day <= finish {
            if calendar.is_working_day(day) {
                for &resource_id in &activity.resources {
                    histogram
                        .entries
                        .entry(day)
                        .or_default()
                        .push(ResourceAssignment {
                            resource_id,
                            activity_id: result.id,
                        });
                }
            }
            day = day
                .checked_add(Duration::days(1))
                .expect("resource histogram date overflow");
        }
    }

    histogram
}

/// Compute earned value metrics for the whole project against a baseline.
///
/// Aggregates per-activity [`EvmMetrics`] using [`Baseline`] snapshots as the
/// planned-value reference. When no baseline is supplied, planned value is
/// derived from the activity's own budget-at-completion.
pub fn evm_metrics(
    project: &Project,
    schedule: &Schedule,
    baseline: Option<&Baseline>,
) -> EvmMetrics {
    let mut total_bac = 0i64;
    let mut total_pv = 0i64;
    let mut total_ev = 0i64;
    let mut total_ac = 0i64;

    // Index baseline snapshots by activity id for quick lookup.
    let snapshots: Option<std::collections::HashMap<ActivityId, &ActivityBaselineSnapshot>> =
        baseline.map(|b| b.snapshots.iter().map(|s| (s.activity_id, s)).collect());

    for result in &schedule.activities {
        let Some(activity) = project.activity(result.id) else {
            continue;
        };
        let snapshot = snapshots
            .as_ref()
            .and_then(|map| map.get(&result.id).copied());
        let metrics = EvmMetrics::for_activity(activity, snapshot);
        total_bac += metrics.budget_at_completion;
        total_pv += metrics.planned_value;
        total_ev += metrics.earned_value;
        total_ac += metrics.actual_cost;
    }

    let cv = total_ev - total_ac;
    let sv = total_ev - total_pv;
    let cpi = if total_ac != 0 {
        Some(total_ev as f64 / total_ac as f64)
    } else {
        None
    };
    let spi = if total_pv != 0 {
        Some(total_ev as f64 / total_pv as f64)
    } else {
        None
    };

    EvmMetrics {
        budget_at_completion: total_bac,
        planned_value: total_pv,
        earned_value: total_ev,
        actual_cost: total_ac,
        cost_variance: cv,
        schedule_variance: sv,
        cost_performance_index: cpi,
        schedule_performance_index: spi,
    }
}

/// Schedule a project with the Critical Path Method.
///
/// # Errors
///
/// Returns [`SchedError`] for duplicate identifiers, relationships that refer
/// to missing activities, cycles in the relationship graph, arithmetic
/// overflow, or an unusable calendar.
pub fn schedule(project: &Project) -> Result<Schedule, SchedError> {
    let calendar = &project.calendar;
    let origin = calendar.next_working_day_on_or_after(project.start_date)?;
    let count = project.activities.len();

    let index = index_activities(project)?;

    // Resolve each activity's calendar once. Activities without a named
    // calendar inherit the project default.
    let activity_calendars: Vec<&Calendar> = project
        .activities
        .iter()
        .map(|activity| match activity.calendar {
            None => Ok(&project.calendar),
            Some(id) => project
                .calendars
                .calendar(id)
                .map_err(|_| SchedError::UnknownCalendar {
                    activity: activity.id,
                    calendar: id,
                }),
        })
        .collect::<Result<_, SchedError>>()?;

    for activity in &project.activities {
        if let Some(wbs) = activity.wbs {
            if project.wbs.node(wbs).is_none() {
                return Err(SchedError::UnknownWbs {
                    activity: activity.id,
                    wbs,
                });
            }
        }
    }

    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut incoming: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut indegree = vec![0usize; count];
    let mut relationship_predecessor = vec![0usize; project.relationships.len()];
    let mut relationship_successor = vec![0usize; project.relationships.len()];
    let mut seen_relationships = HashSet::with_capacity(project.relationships.len());

    for (relationship_index, relationship) in project.relationships.iter().enumerate() {
        if !seen_relationships.insert(relationship.id) {
            return Err(SchedError::DuplicateRelationship(relationship.id));
        }

        let predecessor =
            *index
                .get(&relationship.predecessor)
                .ok_or(SchedError::UnknownActivity {
                    relationship: relationship.id,
                    activity: relationship.predecessor,
                })?;
        let successor = *index
            .get(&relationship.successor)
            .ok_or(SchedError::UnknownActivity {
                relationship: relationship.id,
                activity: relationship.successor,
            })?;

        relationship_predecessor[relationship_index] = predecessor;
        relationship_successor[relationship_index] = successor;
        successors[predecessor].push(successor);
        incoming[successor].push(relationship_index);
        outgoing[predecessor].push(relationship_index);
        indegree[successor] = indegree[successor].saturating_add(1);
    }

    let order = topological_order(count, &successors, &indegree);
    if order.len() != count {
        let mut remaining = vec![true; count];
        for &processed in &order {
            remaining[processed] = false;
        }
        let cycle = find_cycle(count, &successors, &remaining);
        let path = cycle
            .into_iter()
            .map(|activity_index| project.activities[activity_index].id)
            .collect();
        return Err(SchedError::Cycle { path });
    }

    // Forward pass: earliest start and finish.
    let mut early_start = vec![origin; count];
    let mut early_finish = vec![origin; count];
    for &activity_index in &order {
        let activity = &project.activities[activity_index];
        let duration = activity.original_duration;
        let constraint = activity.constraint;
        // The successor's calendar governs when it may start and finish.
        let calendar = activity_calendars[activity_index];
        let mut start = calendar.next_working_day_on_or_after(project.start_date)?;

        for &relationship_index in &incoming[activity_index] {
            let relationship = &project.relationships[relationship_index];
            let predecessor = relationship_predecessor[relationship_index];
            let candidate = match relationship.link_type {
                LinkType::Fs => advance(
                    calendar,
                    early_finish[predecessor],
                    lag_plus_one(relationship.lag)?,
                )?,
                LinkType::Ss => advance(calendar, early_start[predecessor], relationship.lag)?,
                LinkType::Ff => {
                    let finish = advance(calendar, early_finish[predecessor], relationship.lag)?;
                    start_from_finish(calendar, finish, duration)?
                }
                LinkType::Sf => {
                    let finish = advance(calendar, early_start[predecessor], relationship.lag)?;
                    start_from_finish(calendar, finish, duration)?
                }
            };

            start = start.max(candidate);
        }

        match constraint {
            Constraint::StartNoEarlierThan(date) => {
                start = start.max(calendar.next_working_day_on_or_after(date)?);
            }
            Constraint::FinishNoEarlierThan(date) => {
                let earliest_finish = calendar.next_working_day_on_or_after(date)?;
                start = start.max(start_from_finish(calendar, earliest_finish, duration)?);
            }
            _ => {}
        }

        let start = calendar.next_working_day_on_or_after(start)?;
        let finish = finish_from_start(calendar, start, duration)?;

        // Upper bounds are hard: if the earliest feasible dates already miss
        // one, no schedule can satisfy the constraint.
        match constraint {
            Constraint::StartNoLaterThan(date) => {
                let latest_start = calendar.prev_working_day_on_or_before(date)?;
                if start > latest_start {
                    return Err(SchedError::ConstraintViolated {
                        activity: activity.id,
                        constraint,
                    });
                }
            }
            Constraint::FinishNoLaterThan(date) => {
                let latest_finish = calendar.prev_working_day_on_or_before(date)?;
                if finish > latest_finish {
                    return Err(SchedError::ConstraintViolated {
                        activity: activity.id,
                        constraint,
                    });
                }
            }
            _ => {}
        }

        early_start[activity_index] = start;
        early_finish[activity_index] = finish;
    }

    let project_finish = early_finish.iter().copied().max().unwrap_or(origin);

    // Backward pass: latest finish and start.
    let mut late_finish = vec![project_finish; count];
    let mut late_start = vec![project_finish; count];
    for &activity_index in order.iter().rev() {
        let activity = &project.activities[activity_index];
        let duration = activity.original_duration;
        let constraint = activity.constraint;
        // The predecessor's calendar governs how late it may finish.
        let calendar = activity_calendars[activity_index];
        let mut finish = project_finish;

        for &relationship_index in &outgoing[activity_index] {
            let relationship = &project.relationships[relationship_index];
            let successor = relationship_successor[relationship_index];
            let candidate = match relationship.link_type {
                LinkType::Fs => retreat(
                    calendar,
                    late_start[successor],
                    lag_plus_one(relationship.lag)?,
                )?,
                LinkType::Ss => {
                    let start = retreat(calendar, late_start[successor], relationship.lag)?;
                    finish_from_start(calendar, start, duration)?
                }
                LinkType::Ff => retreat(calendar, late_finish[successor], relationship.lag)?,
                LinkType::Sf => {
                    let start = retreat(calendar, late_finish[successor], relationship.lag)?;
                    finish_from_start(calendar, start, duration)?
                }
            };

            finish = finish.min(candidate);
        }

        // Every activity must finish on a working day of its own calendar.
        // `project_finish` is taken from whichever calendar produced it and may
        // not be a working day here, so normalise (working-day inputs are left
        // untouched by `prev_working_day_on_or_before`).
        finish = calendar.prev_working_day_on_or_before(finish)?;

        match constraint {
            Constraint::StartNoLaterThan(date) => {
                let latest_start = calendar.prev_working_day_on_or_before(date)?;
                finish = finish.min(finish_from_start(calendar, latest_start, duration)?);
            }
            Constraint::FinishNoLaterThan(date) => {
                finish = finish.min(calendar.prev_working_day_on_or_before(date)?);
            }
            _ => {}
        }

        match constraint {
            Constraint::StartNoEarlierThan(date) => {
                let earliest_start = calendar.next_working_day_on_or_after(date)?;
                finish = finish.max(finish_from_start(calendar, earliest_start, duration)?);
            }
            Constraint::FinishNoEarlierThan(date) => {
                finish = finish.max(calendar.next_working_day_on_or_after(date)?);
            }
            _ => {}
        }

        late_finish[activity_index] = finish;
        late_start[activity_index] = start_from_finish(calendar, finish, duration)?;
    }

    // Scheduled dates. ASAP activities report their early dates. ALAP
    // activities are moved to their late dates, pulled back only when a
    // successor's scheduled date (which is fixed by then, because this walks
    // in reverse topological order) would otherwise be violated.
    let mut scheduled_start = early_start.clone();
    let mut scheduled_finish = early_finish.clone();
    for &activity_index in order.iter().rev() {
        let activity = &project.activities[activity_index];
        if activity.constraint != Constraint::AsLateAsPossible {
            continue;
        }
        let duration = activity.original_duration;
        let calendar = activity_calendars[activity_index];
        let mut finish = late_finish[activity_index];

        for &relationship_index in &outgoing[activity_index] {
            let relationship = &project.relationships[relationship_index];
            let successor = relationship_successor[relationship_index];
            let candidate = match relationship.link_type {
                LinkType::Fs => retreat(
                    calendar,
                    scheduled_start[successor],
                    lag_plus_one(relationship.lag)?,
                )?,
                LinkType::Ss => {
                    let start = retreat(calendar, scheduled_start[successor], relationship.lag)?;
                    finish_from_start(calendar, start, duration)?
                }
                LinkType::Ff => retreat(calendar, scheduled_finish[successor], relationship.lag)?,
                LinkType::Sf => {
                    let start = retreat(calendar, scheduled_finish[successor], relationship.lag)?;
                    finish_from_start(calendar, start, duration)?
                }
            };

            finish = finish.min(candidate);
        }

        let finish = finish.max(early_finish[activity_index]);
        scheduled_finish[activity_index] = finish;
        scheduled_start[activity_index] = start_from_finish(calendar, finish, duration)?;
    }

    // Float values and the critical set.
    let mut activities = Vec::with_capacity(count);
    for (activity_index, activity) in project.activities.iter().enumerate() {
        let calendar = activity_calendars[activity_index];
        let total_float =
            calendar.working_days_between(early_start[activity_index], late_start[activity_index]);
        let mut free_float = total_float;

        if !outgoing[activity_index].is_empty() {
            let mut minimum = i64::MAX;
            for &relationship_index in &outgoing[activity_index] {
                let relationship = &project.relationships[relationship_index];
                let successor = relationship_successor[relationship_index];
                let slack = match relationship.link_type {
                    LinkType::Fs => calendar
                        .working_days_between(early_finish[activity_index], early_start[successor])
                        .checked_sub(1)
                        .and_then(|value| value.checked_sub(relationship.lag)),
                    LinkType::Ss => calendar
                        .working_days_between(early_start[activity_index], early_start[successor])
                        .checked_sub(relationship.lag),
                    LinkType::Ff => calendar
                        .working_days_between(early_finish[activity_index], early_finish[successor])
                        .checked_sub(relationship.lag),
                    LinkType::Sf => calendar
                        .working_days_between(early_start[activity_index], early_finish[successor])
                        .checked_sub(relationship.lag),
                }
                .ok_or(SchedError::ArithmeticOverflow)?;
                minimum = minimum.min(slack);
            }
            free_float = minimum.max(0);
        }

        activities.push(ActivitySchedule {
            id: activity.id,
            early_start: early_start[activity_index],
            early_finish: early_finish[activity_index],
            late_start: late_start[activity_index],
            late_finish: late_finish[activity_index],
            scheduled_start: scheduled_start[activity_index],
            scheduled_finish: scheduled_finish[activity_index],
            total_float,
            free_float,
            critical: total_float <= 0,
        });
    }

    let mut critical_indices: Vec<usize> = activities
        .iter()
        .enumerate()
        .filter_map(|(activity_index, activity)| activity.critical.then_some(activity_index))
        .collect();
    critical_indices.sort_unstable_by_key(|&activity_index| {
        (
            early_start[activity_index],
            project.activities[activity_index].id,
        )
    });
    let critical_path = critical_indices
        .into_iter()
        .map(|activity_index| project.activities[activity_index].id)
        .collect();

    Ok(Schedule {
        project_start: early_start.iter().copied().min().unwrap_or(origin),
        project_finish,
        activities,
        critical_path,
    })
}

/// Build an identifier-to-index map, rejecting duplicates.
fn index_activities(project: &Project) -> Result<HashMap<ActivityId, usize>, SchedError> {
    let mut index = HashMap::with_capacity(project.activities.len());
    for (activity_index, activity) in project.activities.iter().enumerate() {
        if index.insert(activity.id, activity_index).is_some() {
            return Err(SchedError::DuplicateActivity(activity.id));
        }
    }
    Ok(index)
}

/// Kahn's algorithm. Returns a partial order when the graph is cyclic.
fn topological_order(count: usize, successors: &[Vec<usize>], indegree: &[usize]) -> Vec<usize> {
    let mut remaining = indegree.to_vec();
    let mut queue: VecDeque<usize> = (0..count)
        .filter(|activity_index| remaining[*activity_index] == 0)
        .collect();
    let mut order = Vec::with_capacity(count);

    while let Some(activity_index) = queue.pop_front() {
        order.push(activity_index);
        for &successor in &successors[activity_index] {
            if remaining[successor] > 0 {
                remaining[successor] -= 1;
            }
            if remaining[successor] == 0 {
                queue.push_back(successor);
            }
        }
    }

    order
}

/// Find one cycle among the unprocessed nodes. The returned path repeats the
/// first node at the end so it reads naturally (`A -> B -> A`).
fn find_cycle(count: usize, successors: &[Vec<usize>], remaining: &[bool]) -> Vec<usize> {
    let mut color = vec![0u8; count];

    for start in 0..count {
        if !remaining[start] || color[start] != 0 {
            continue;
        }

        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        let mut path: Vec<usize> = vec![start];
        color[start] = 1;

        while let Some(&(activity_index, next_child)) = stack.last() {
            if next_child < successors[activity_index].len() {
                let child = successors[activity_index][next_child];
                if let Some(entry) = stack.last_mut() {
                    entry.1 += 1;
                }

                if !remaining[child] {
                    continue;
                }

                match color[child] {
                    0 => {
                        color[child] = 1;
                        stack.push((child, 0));
                        path.push(child);
                    }
                    1 => {
                        if let Some(position) = path.iter().position(|entry| *entry == child) {
                            let mut cycle = path[position..].to_vec();
                            cycle.push(child);
                            return cycle;
                        }
                        return vec![child, child];
                    }
                    _ => {}
                }
            } else {
                color[activity_index] = 2;
                stack.pop();
                path.pop();
            }
        }
    }

    Vec::new()
}

/// Finish date for an activity of `duration` working days starting on `start`.
fn finish_from_start(calendar: &Calendar, start: Date, duration: u32) -> Result<Date, SchedError> {
    if duration == 0 {
        Ok(start)
    } else {
        let offset = i64::from(duration) - 1;
        Ok(calendar.add_working_days(start, offset)?)
    }
}

/// Start date for an activity of `duration` working days finishing on `finish`.
fn start_from_finish(calendar: &Calendar, finish: Date, duration: u32) -> Result<Date, SchedError> {
    if duration == 0 {
        Ok(finish)
    } else {
        let offset = -(i64::from(duration) - 1);
        Ok(calendar.add_working_days(finish, offset)?)
    }
}

/// Move `days` working days forward from `date`.
///
/// Unlike [`Calendar::add_working_days`], a non-working `date` does not become
/// an anchor that is then shifted an extra step: zero moves to the next working
/// day on or after `date`, and positive distances are counted strictly after
/// `date`. CPM needs this exact meaning when linked activities use different
/// calendars (for example, a Friday finish followed by a six-day calendar
/// starting on Saturday).
fn advance(calendar: &Calendar, date: Date, days: i64) -> Result<Date, SchedError> {
    if days == 0 {
        Ok(calendar.next_working_day_on_or_after(date)?)
    } else if days > 0 {
        let first = calendar.next_working_day_after(date)?;
        Ok(calendar.add_working_days(first, days - 1)?)
    } else {
        let distance = days.checked_neg().ok_or(SchedError::ArithmeticOverflow)?;
        retreat(calendar, date, distance)
    }
}

/// Move `days` working days backward from `date`.
///
/// The mirror of [`advance`]: zero moves to the previous working day on or
/// before `date`, and positive distances are counted strictly before `date`.
fn retreat(calendar: &Calendar, date: Date, days: i64) -> Result<Date, SchedError> {
    if days == 0 {
        Ok(calendar.prev_working_day_on_or_before(date)?)
    } else if days > 0 {
        let first = calendar.prev_working_day_before(date)?;
        Ok(calendar.add_working_days(first, -(days - 1))?)
    } else {
        let distance = days.checked_neg().ok_or(SchedError::ArithmeticOverflow)?;
        advance(calendar, date, distance)
    }
}

/// `lag + 1`, used by the FS forward constraint.
fn lag_plus_one(lag: i64) -> Result<i64, SchedError> {
    lag.checked_add(1).ok_or(SchedError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Activity, Constraint, LinkType, Relationship, Resource};
    use std::time::Instant;
    use time::Month;

    fn jan(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::January, day).expect("valid test date")
    }

    fn project() -> Project {
        Project::new("Engine test", jan(5))
    }

    fn add_activity(project: &mut Project, id: u64, name: &str, duration: u32) -> ActivityId {
        let activity_id = ActivityId(id);
        project
            .activities
            .push(Activity::new(activity_id, name, duration));
        activity_id
    }

    fn add_relationship(
        project: &mut Project,
        id: u64,
        predecessor: ActivityId,
        successor: ActivityId,
        link_type: LinkType,
        lag: i64,
    ) {
        project.relationships.push(Relationship::new(
            RelationshipId(id),
            predecessor,
            successor,
            link_type,
            lag,
        ));
    }

    fn result(schedule: &Schedule, id: ActivityId) -> &ActivitySchedule {
        schedule.activity(id).expect("activity is scheduled")
    }

    #[test]
    fn fs_link_with_lag_is_the_canonical_slice() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 1);

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(schedule.project_start, jan(5));
        assert_eq!(schedule.project_finish, jan(12));

        let a_result = result(&schedule, a);
        assert_eq!(a_result.early_start, jan(5));
        assert_eq!(a_result.early_finish, jan(6));
        assert_eq!(a_result.late_start, jan(5));
        assert_eq!(a_result.late_finish, jan(6));
        assert_eq!(a_result.total_float, 0);
        assert_eq!(a_result.free_float, 0);
        assert!(a_result.critical);

        let b_result = result(&schedule, b);
        assert_eq!(b_result.early_start, jan(8));
        assert_eq!(b_result.early_finish, jan(12));
        assert_eq!(b_result.late_start, jan(8));
        assert_eq!(b_result.late_finish, jan(12));
        assert_eq!(b_result.total_float, 0);
        assert_eq!(b_result.free_float, 0);
        assert!(b_result.critical);

        assert_eq!(schedule.critical_path, vec![a, b]);
    }

    #[test]
    fn project_start_on_weekend_moves_to_next_working_day() {
        let mut project = Project::new("Weekend start", jan(10)); // Saturday
        let a = add_activity(&mut project, 1, "A", 2);

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(schedule.project_start, jan(12));
        assert_eq!(result(&schedule, a).early_start, jan(12));
        assert_eq!(result(&schedule, a).early_finish, jan(13));
    }

    #[test]
    fn ss_link_uses_predecessor_start() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2); // Jan 5 - Jan 6
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Ss, 1);

        let schedule = schedule(&project).expect("schedule succeeds");
        let b_result = result(&schedule, b);

        assert_eq!(b_result.early_start, jan(6));
        assert_eq!(b_result.early_finish, jan(7));
        assert_eq!(b_result.late_start, jan(6));
        assert_eq!(b_result.late_finish, jan(7));
        assert_eq!(b_result.total_float, 0);
    }

    #[test]
    fn ff_link_uses_predecessor_finish() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2); // Jan 5 - Jan 6
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Ff, 1);

        let schedule = schedule(&project).expect("schedule succeeds");
        let b_result = result(&schedule, b);

        assert_eq!(b_result.early_start, jan(6));
        assert_eq!(b_result.early_finish, jan(7));
        assert_eq!(b_result.late_start, jan(6));
        assert_eq!(b_result.late_finish, jan(7));
        assert_eq!(b_result.total_float, 0);
    }

    #[test]
    fn sf_link_uses_predecessor_start() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 3); // Jan 5 - Jan 7
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Sf, 2);

        let schedule = schedule(&project).expect("schedule succeeds");
        let b_result = result(&schedule, b);

        assert_eq!(b_result.early_start, jan(6));
        assert_eq!(b_result.early_finish, jan(7));
        assert_eq!(b_result.late_start, jan(6));
        assert_eq!(b_result.late_finish, jan(7));
        assert_eq!(b_result.total_float, 0);
    }

    #[test]
    fn negative_lag_becomes_a_lead() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2); // Jan 5 - Jan 6
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, -1);

        let schedule = schedule(&project).expect("schedule succeeds");
        let b_result = result(&schedule, b);

        assert_eq!(b_result.early_start, jan(6));
        assert_eq!(b_result.early_finish, jan(7));
    }

    #[test]
    fn total_and_free_float_are_computed() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1); // Jan 5
        let b = add_activity(&mut project, 2, "B", 1); // Jan 6
        let c = add_activity(&mut project, 3, "C", 1); // Jan 9
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, c, LinkType::Fs, 3);

        let schedule = schedule(&project).expect("schedule succeeds");

        let a_result = result(&schedule, a);
        assert_eq!(a_result.early_start, jan(5));
        assert_eq!(a_result.early_finish, jan(5));
        assert_eq!(a_result.late_start, jan(5));
        assert_eq!(a_result.late_finish, jan(5));
        assert_eq!(a_result.total_float, 0);
        assert_eq!(a_result.free_float, 0);
        assert!(a_result.critical);

        let b_result = result(&schedule, b);
        assert_eq!(b_result.early_start, jan(6));
        assert_eq!(b_result.early_finish, jan(6));
        assert_eq!(b_result.late_start, jan(9));
        assert_eq!(b_result.late_finish, jan(9));
        assert_eq!(b_result.total_float, 3);
        assert_eq!(b_result.free_float, 3);
        assert!(!b_result.critical);

        let c_result = result(&schedule, c);
        assert_eq!(c_result.early_start, jan(9));
        assert_eq!(c_result.late_start, jan(9));
        assert_eq!(c_result.total_float, 0);
        assert!(c_result.critical);

        assert_eq!(schedule.critical_path, vec![a, c]);
    }

    #[test]
    fn holidays_are_respected_by_the_engine() {
        let mut project = project();
        project.calendar = project.calendar.with_holiday(jan(7)); // Wednesday
        let a = add_activity(&mut project, 1, "A", 2); // Jan 5 - Jan 6
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);

        let schedule = schedule(&project).expect("schedule succeeds");
        let b_result = result(&schedule, b);

        assert_eq!(b_result.early_start, jan(8));
        assert_eq!(b_result.early_finish, jan(9));
    }

    #[test]
    fn cycle_is_reported_with_the_path() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        let b = add_activity(&mut project, 2, "B", 1);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 2, b, a, LinkType::Fs, 0);

        let error = schedule(&project).expect_err("cycle must be rejected");

        match error {
            SchedError::Cycle { path } => {
                assert_eq!(path.first(), path.last());
                assert!(path.contains(&a));
                assert!(path.contains(&b));
                assert_eq!(path.len(), 3);
            }
            other => panic!("expected cycle error, got {other:?}"),
        }
    }

    #[test]
    fn self_loop_is_reported_as_a_cycle() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        add_relationship(&mut project, 1, a, a, LinkType::Fs, 0);

        let error = schedule(&project).expect_err("self loop must be rejected");

        match error {
            SchedError::Cycle { path } => assert_eq!(path, vec![a, a]),
            other => panic!("expected cycle error, got {other:?}"),
        }
    }

    #[test]
    fn unknown_activity_is_rejected() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        add_relationship(&mut project, 1, a, ActivityId(99), LinkType::Fs, 0);

        assert!(matches!(
            schedule(&project),
            Err(SchedError::UnknownActivity { .. })
        ));
    }

    #[test]
    fn duplicate_activity_ids_are_rejected() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 1);
        add_activity(&mut project, 1, "Duplicate", 1);

        assert_eq!(
            schedule(&project),
            Err(SchedError::DuplicateActivity(ActivityId(1)))
        );
    }

    #[test]
    fn empty_project_schedules_to_its_start() {
        let project = project();
        let schedule = schedule(&project).expect("empty project schedules");

        assert_eq!(schedule.project_start, jan(5));
        assert_eq!(schedule.project_finish, jan(5));
        assert!(schedule.activities.is_empty());
        assert!(schedule.critical_path.is_empty());
    }

    #[test]
    fn milestones_have_zero_duration() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "Milestone", 0);

        let schedule = schedule(&project).expect("schedule succeeds");
        let a_result = result(&schedule, a);

        assert_eq!(a_result.early_start, jan(5));
        assert_eq!(a_result.early_finish, jan(5));
        assert_eq!(a_result.late_start, jan(5));
        assert_eq!(a_result.late_finish, jan(5));
    }

    #[test]
    fn schedule_writes_cached_values_back_to_the_project() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);

        let schedule = schedule(&project).expect("schedule succeeds");
        schedule.apply_to(&mut project);

        let a_activity = project.activity(a).expect("activity exists");
        assert_eq!(a_activity.early_start, Some(jan(5)));
        assert_eq!(a_activity.early_finish, Some(jan(6)));
        assert_eq!(a_activity.late_start, Some(jan(5)));
        assert_eq!(a_activity.late_finish, Some(jan(6)));
        assert_eq!(a_activity.total_float, Some(0));
        assert_eq!(a_activity.free_float, Some(0));
        assert!(a_activity.critical);
    }

    #[test]
    fn schedule_round_trips_through_json() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 1);

        let schedule = schedule(&project).expect("schedule succeeds");
        let json = serde_json::to_string(&schedule).expect("serialise schedule");
        let decoded: Schedule = serde_json::from_str(&json).expect("deserialise schedule");

        assert_eq!(decoded, schedule);
    }

    fn six_day_week() -> Calendar {
        Calendar::new([true, true, true, true, true, true, false]).expect("valid six-day week")
    }

    fn add_named_calendar(
        project: &mut Project,
        id: u64,
        name: &str,
        calendar: Calendar,
    ) -> CalendarId {
        let calendar_id = CalendarId(id);
        project
            .calendars
            .add(calendar_id, name, calendar)
            .expect("named calendar is valid");
        calendar_id
    }

    #[test]
    fn successor_uses_its_own_calendar_for_start_and_finish() {
        let mut project = project();
        let six_day = add_named_calendar(&mut project, 1, "Six day", six_day_week());
        let a = add_activity(&mut project, 1, "A", 5); // Mon 5 Jan – Fri 9 Jan
        let b = add_activity(&mut project, 2, "B", 2);
        project.activity_mut(b).expect("b").calendar = Some(six_day);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);

        let schedule = schedule(&project).expect("schedule succeeds");
        let a_result = result(&schedule, a);
        let b_result = result(&schedule, b);

        assert_eq!(a_result.early_finish, jan(9));
        assert_eq!(b_result.early_start, jan(10)); // Saturday
        assert_eq!(b_result.early_finish, jan(12)); // Sunday skipped
        assert_eq!(schedule.project_finish, jan(12));
        // B's six-day week starts the day after A finishes, so both are
        // critical despite the different calendars.
        assert_eq!(a_result.total_float, 0);
        assert!(a_result.critical);
        assert!(b_result.critical);
    }

    #[test]
    fn holiday_on_named_calendar_does_not_affect_default_calendar() {
        let mut project = Project::new("Holiday", jan(10)); // Saturday
        let calendar = six_day_week().with_holiday(jan(12));
        add_named_calendar(&mut project, 1, "Six day + holiday", calendar);
        let a = add_activity(&mut project, 1, "Default", 1);
        let b = add_activity(&mut project, 2, "Named", 2);
        project.activity_mut(b).expect("b").calendar = Some(CalendarId(1));

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(result(&schedule, a).early_start, jan(12)); // Monday
        assert_eq!(result(&schedule, b).early_start, jan(10)); // Saturday
        assert_eq!(result(&schedule, b).early_finish, jan(13)); // skips Sunday + holiday
        assert_eq!(result(&schedule, a).early_finish, jan(12));
    }

    #[test]
    fn activity_calendar_can_start_before_default_calendar_origin() {
        let mut project = Project::new("Weekend start", jan(10)); // Saturday
        add_named_calendar(&mut project, 1, "Six day", six_day_week());
        let a = add_activity(&mut project, 1, "Six day", 1);
        project.activity_mut(a).expect("a").calendar = Some(CalendarId(1));
        let b = add_activity(&mut project, 2, "Five day", 1);

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(result(&schedule, a).early_start, jan(10)); // Saturday
        assert_eq!(result(&schedule, b).early_start, jan(12)); // Monday
        assert_eq!(schedule.project_start, jan(10));
    }

    #[test]
    fn unknown_calendar_selection_is_rejected() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        project.activity_mut(a).expect("a").calendar = Some(CalendarId(99));

        assert_eq!(
            schedule(&project),
            Err(SchedError::UnknownCalendar {
                activity: a,
                calendar: CalendarId(99),
            })
        );
    }

    #[test]
    fn unknown_wbs_reference_is_rejected() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        project.activity_mut(a).expect("a").wbs = Some(WbsId(99));

        assert_eq!(
            schedule(&project),
            Err(SchedError::UnknownWbs {
                activity: a,
                wbs: WbsId(99),
            })
        );
    }

    fn constrain(project: &mut Project, id: ActivityId, constraint: Constraint) {
        project
            .activity_mut(id)
            .expect("activity exists")
            .constraint = constraint;
    }

    #[test]
    fn start_no_earlier_than_delays_the_activity() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        constrain(&mut project, a, Constraint::StartNoEarlierThan(jan(7)));

        let schedule = schedule(&project).expect("schedule succeeds");
        let a_result = result(&schedule, a);

        assert_eq!(a_result.early_start, jan(7));
        assert_eq!(a_result.early_finish, jan(8));
        assert_eq!(a_result.scheduled_start, jan(7));
        assert_eq!(a_result.scheduled_finish, jan(8));
        assert_eq!(a_result.total_float, 0);
        assert_eq!(schedule.project_start, jan(7));
        assert_eq!(schedule.project_finish, jan(8));
    }

    #[test]
    fn finish_no_earlier_than_raises_the_forward_dates() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        let c = add_activity(&mut project, 2, "C", 5);
        constrain(&mut project, a, Constraint::FinishNoEarlierThan(jan(7)));

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(result(&schedule, a).early_start, jan(7));
        assert_eq!(result(&schedule, a).early_finish, jan(7));
        assert_eq!(result(&schedule, a).scheduled_start, jan(7));
        assert_eq!(schedule.project_finish, jan(9));
        assert!(result(&schedule, c).critical);
    }

    #[test]
    fn finish_no_later_than_caps_late_dates() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        add_activity(&mut project, 2, "C", 5);
        constrain(&mut project, a, Constraint::FinishNoLaterThan(jan(9)));

        let schedule = schedule(&project).expect("schedule succeeds");
        let a_result = result(&schedule, a);

        assert_eq!(a_result.early_start, jan(5));
        assert_eq!(a_result.early_finish, jan(6));
        assert_eq!(a_result.late_finish, jan(9));
        assert_eq!(a_result.late_start, jan(8));
        assert_eq!(a_result.total_float, 3);
        assert_eq!(a_result.scheduled_start, jan(5));
    }

    #[test]
    fn start_no_later_than_limits_float() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        add_activity(&mut project, 2, "C", 5);
        constrain(&mut project, a, Constraint::StartNoLaterThan(jan(7)));

        let schedule = schedule(&project).expect("schedule succeeds");
        let a_result = result(&schedule, a);

        assert_eq!(a_result.late_start, jan(7));
        assert_eq!(a_result.late_finish, jan(7));
        assert_eq!(a_result.total_float, 2);
        assert_eq!(a_result.scheduled_start, jan(5));
    }

    #[test]
    fn an_unsatisfiable_upper_bound_is_reported() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 5);
        let constraint = Constraint::FinishNoLaterThan(jan(7));
        constrain(&mut project, a, constraint);

        assert_eq!(
            schedule(&project),
            Err(SchedError::ConstraintViolated {
                activity: a,
                constraint,
            })
        );
    }

    #[test]
    fn alap_uses_late_dates_without_delaying_the_project() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        let c = add_activity(&mut project, 2, "C", 5);
        constrain(&mut project, a, Constraint::AsLateAsPossible);

        let schedule = schedule(&project).expect("schedule succeeds");
        let a_result = result(&schedule, a);

        assert_eq!(a_result.early_start, jan(5));
        assert_eq!(a_result.early_finish, jan(5));
        assert_eq!(a_result.late_start, jan(9));
        assert_eq!(a_result.scheduled_start, jan(9));
        assert_eq!(a_result.scheduled_finish, jan(9));
        assert_eq!(a_result.total_float, 4);
        assert_eq!(result(&schedule, c).scheduled_start, jan(5));
        assert_eq!(schedule.project_finish, jan(9));
    }

    #[test]
    fn alap_predecessor_does_not_delay_an_asap_successor() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        let b = add_activity(&mut project, 2, "B", 1);
        add_activity(&mut project, 3, "C", 5);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        constrain(&mut project, a, Constraint::AsLateAsPossible);

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(result(&schedule, a).late_start, jan(8));
        assert_eq!(result(&schedule, a).scheduled_start, jan(5));
        assert_eq!(result(&schedule, b).scheduled_start, jan(6));
        assert_eq!(schedule.project_finish, jan(9));
    }

    #[test]
    fn alap_successor_moves_to_its_late_date() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 1);
        let b = add_activity(&mut project, 2, "B", 1);
        add_activity(&mut project, 3, "C", 5);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        constrain(&mut project, b, Constraint::AsLateAsPossible);

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(result(&schedule, a).scheduled_start, jan(5));
        assert_eq!(result(&schedule, b).scheduled_start, jan(9));
        assert!(result(&schedule, b).scheduled_start > result(&schedule, a).scheduled_finish);
        assert_eq!(schedule.project_finish, jan(9));
    }

    #[test]
    fn constraint_dates_use_the_activity_calendar() {
        let mut project = Project::new("Constraint calendar", jan(10)); // Saturday
        add_named_calendar(&mut project, 1, "Six day", six_day_week());
        let a = add_activity(&mut project, 1, "A", 1);
        project.activity_mut(a).expect("a").calendar = Some(CalendarId(1));
        constrain(&mut project, a, Constraint::StartNoEarlierThan(jan(10)));

        let schedule = schedule(&project).expect("schedule succeeds");

        assert_eq!(result(&schedule, a).early_start, jan(10)); // Saturday
        assert_eq!(result(&schedule, a).early_finish, jan(10));
        assert_eq!(schedule.project_start, jan(10));
    }
    #[test]
    fn ten_thousand_activities_schedule_quickly() {
        let mut project = Project::new("Performance", jan(5));
        for index in 0..10_000 {
            project
                .activities
                .push(Activity::new(ActivityId(index + 1), format!("A{index}"), 1));
        }
        for index in 0..9_999 {
            project.relationships.push(Relationship::new(
                RelationshipId(index + 1),
                ActivityId(index + 1),
                ActivityId(index + 2),
                LinkType::Fs,
                0,
            ));
        }

        let started = Instant::now();
        let schedule = schedule(&project).expect("large project schedules");
        let elapsed = started.elapsed();

        assert_eq!(schedule.activities.len(), 10_000);
        assert!(
            elapsed.as_millis() < 500,
            "scheduling 10,000 activities took {} ms",
            elapsed.as_millis()
        );
    }

    fn add_resource(project: &mut Project, id: u64, name: &str) -> ResourceId {
        let resource_id = ResourceId(id);
        project.resources.push(Resource::new(resource_id, name));
        resource_id
    }

    #[test]
    fn resource_histogram_counts_assigned_days() {
        let mut project = project();
        let a1 = add_activity(&mut project, 1, "A1", 2);
        let a2 = add_activity(&mut project, 2, "A2", 1);
        let r1 = add_resource(&mut project, 1, "Resource one");
        let r2 = add_resource(&mut project, 2, "Resource two");

        project.activities[0].resources.push(r1);
        project.activities[1].resources.push(r2);
        project.relationships.push(Relationship::new(
            RelationshipId(1),
            a1,
            a2,
            LinkType::Fs,
            0,
        ));

        let schedule = schedule(&project).expect("schedule");
        let histogram = resource_histogram(&project, &schedule);

        assert_eq!(histogram.days_with_usage(), 3);

        let jan5 = jan(5);
        let jan6 = jan(6);
        let jan7 = jan(7);

        assert_eq!(
            histogram.assignments_on(jan5),
            Some(&vec![ResourceAssignment {
                resource_id: r1,
                activity_id: a1,
            }])
        );
        assert_eq!(
            histogram.assignments_on(jan6),
            Some(&vec![ResourceAssignment {
                resource_id: r1,
                activity_id: a1,
            }])
        );
        assert_eq!(
            histogram.assignments_on(jan7),
            Some(&vec![ResourceAssignment {
                resource_id: r2,
                activity_id: a2,
            }])
        );
        assert!(histogram.assignments_on(jan(4)).is_none());
    }

    #[test]
    fn resource_histogram_skips_unscheduled_and_unassigned_activities() {
        let mut project = project();
        let scheduled = add_activity(&mut project, 1, "Scheduled", 2);
        let unscheduled = add_activity(&mut project, 2, "Unscheduled", 2);
        let no_resources = add_activity(&mut project, 3, "No resources", 2);
        let r = add_resource(&mut project, 1, "R");

        project.activities[0].resources.push(r);
        project
            .resources
            .push(Resource::new(ResourceId(2), "Unused"));

        let schedule = schedule(&project).expect("schedule");
        let histogram = resource_histogram(&project, &schedule);

        // Only activity 1 has resources and scheduled dates.
        assert_eq!(histogram.days_with_usage(), 2);
        assert!(histogram.assignments_on(jan(5)).is_some());
        assert!(histogram.assignments_on(jan(6)).is_some());
        assert!(histogram.assignments_on(jan(7)).is_none());
    }
}
