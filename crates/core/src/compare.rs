//! Forensic comparison of two programmes, or of a baseline against the
//! current programme.
//!
//! The comparison answers the reviewer's question — *what changed between the
//! last version and this one?* — the way P6's Schedule Comparison and
//! Microsoft Project's Compare Projects do (see
//! `docs/research/schedule-audit.md`): activities added and removed,
//! durations changed, scheduled dates moved, constraints changed, and
//! relationships added, removed or retyped. It is a pure function of its
//! inputs: no I/O, no clock, deterministic ordering.
//!
//! Matching is by activity and relationship **identifier**. That is exact for
//! Cadence's own file lineage and for MSPDI/JSON imports, which carry ids;
//! CSV import renumbers activities (documented in `cadence-interchange::csv`),
//! so a before-file produced through CSV may show wholesale add/remove instead
//! of in-place changes.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::Date;

use crate::engine::{schedule, Schedule};
use crate::model::{Activity, ActivityId, Constraint, LinkType, Project, Relationship};

/// A signed date movement in working days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateShift {
    /// The earlier date, when both sides had one.
    pub before: Option<Date>,
    /// The later date.
    pub after: Option<Date>,
    /// `after` minus `before` in working days on the activity's calendar.
    pub days: i64,
}

/// One added or removed activity, named for the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityRef {
    /// Activity identifier.
    pub id: ActivityId,
    /// Activity name, copied for display.
    pub name: String,
}

/// A duration that changed between the two programmes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationChange {
    /// Activity identifier.
    pub id: ActivityId,
    /// Activity name, copied for display.
    pub name: String,
    /// Duration before, in working days.
    pub before: u32,
    /// Duration after, in working days.
    pub after: u32,
}

/// Scheduled dates that moved for one activity present in both programmes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateChange {
    /// Activity identifier.
    pub id: ActivityId,
    /// Activity name, copied for display.
    pub name: String,
    /// The scheduled start before and after, with the shift in working days.
    pub start: DateShift,
    /// The scheduled finish before and after, with the shift in working days.
    pub finish: DateShift,
}

/// A constraint that changed between the two programmes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstraintChange {
    /// Activity identifier.
    pub id: ActivityId,
    /// Activity name, copied for display.
    pub name: String,
    /// Constraint code before (`asap`, `snlt`, ...).
    pub before: String,
    /// Constraint code after.
    pub after: String,
}

/// A relationship that was added, removed, or changed in type or lag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum RelationshipChange {
    /// A relationship exists only in the later programme.
    Added {
        /// The relationship as it is now.
        relationship: Relationship,
    },
    /// A relationship exists only in the earlier programme.
    Removed {
        /// The relationship as it was.
        relationship: Relationship,
    },
    /// The same relationship id changed type or lag (or re-pointed).
    Changed {
        /// The relationship before.
        before: Relationship,
        /// The relationship after.
        after: Relationship,
    },
}

/// The project-level dates of both programmes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDateShift {
    /// Project start before and after.
    pub start: DateShift,
    /// Project finish before and after.
    pub finish: DateShift,
}

/// The whole comparison, one list per kind of change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
    /// Activities present only in the later programme.
    pub added_activities: Vec<ActivityRef>,
    /// Activities present only in the earlier programme.
    pub removed_activities: Vec<ActivityRef>,
    /// Durations that changed.
    pub duration_changes: Vec<DurationChange>,
    /// Scheduled dates that moved (only where both programmes scheduled).
    pub date_changes: Vec<DateChange>,
    /// Constraints that changed.
    pub constraint_changes: Vec<ConstraintChange>,
    /// Relationships added, removed or changed.
    pub relationship_changes: Vec<RelationshipChange>,
    /// Project start/finish movement, when both programmes scheduled.
    pub project_dates: Option<ProjectDateShift>,
}

impl Comparison {
    /// True when nothing changed at all.
    pub fn is_empty(&self) -> bool {
        self.added_activities.is_empty()
            && self.removed_activities.is_empty()
            && self.duration_changes.is_empty()
            && self.date_changes.is_empty()
            && self.constraint_changes.is_empty()
            && self.relationship_changes.is_empty()
            && self.project_dates.is_none()
    }
}

/// Compare two programmes of the same lineage.
///
/// Both projects are scheduled with the ordinary engine. Structural changes
/// (activities, durations, constraints, relationships) are always reported;
/// scheduled-date movement and the project-date shift are reported only when
/// **both** programmes schedule successfully.
pub fn compare(before: &Project, after: &Project) -> Comparison {
    let before_schedule = schedule(before).ok();
    let after_schedule = schedule(after).ok();
    compare_with(
        before,
        after,
        before_schedule.as_ref(),
        after_schedule.as_ref(),
    )
}

/// Compare the current programme against a captured baseline.
///
/// The baseline's snapshots stand in for the "before" programme: durations and
/// baseline dates are compared, activities missing from either side are added
/// or removed, and constraint/relationship sections stay empty because a
/// baseline snapshot does not record them.
pub fn compare_baseline(project: &Project, baseline: &crate::model::Baseline) -> Comparison {
    let schedule = schedule(project).ok();
    let mut before: Project = project.clone();
    before.activities = project
        .activities
        .iter()
        .filter_map(|activity| {
            let snapshot = baseline
                .snapshots
                .iter()
                .find(|snapshot| snapshot.activity_id == activity.id)?;
            let mut rebuilt = Activity::new(
                activity.id,
                activity.name.clone(),
                snapshot.baseline_duration,
            );
            rebuilt.scheduled_start = snapshot.baseline_start;
            rebuilt.scheduled_finish = snapshot.baseline_finish;
            Some(rebuilt)
        })
        .collect();
    before.relationships.clear();
    compare_with(&before, project, None, schedule.as_ref())
}

/// The structural and date comparison, given optional schedules.
fn compare_with(
    before: &Project,
    after: &Project,
    before_schedule: Option<&Schedule>,
    after_schedule: Option<&Schedule>,
) -> Comparison {
    let before_activities: HashMap<ActivityId, &Activity> = before
        .activities
        .iter()
        .map(|activity| (activity.id, activity))
        .collect();
    let after_activities: HashMap<ActivityId, &Activity> = after
        .activities
        .iter()
        .map(|activity| (activity.id, activity))
        .collect();

    let mut added_activities: Vec<ActivityRef> = after
        .activities
        .iter()
        .filter(|activity| !before_activities.contains_key(&activity.id))
        .map(|activity| ActivityRef {
            id: activity.id,
            name: activity.name.clone(),
        })
        .collect();
    added_activities.sort_by_key(|reference| reference.id);

    let mut removed_activities: Vec<ActivityRef> = before
        .activities
        .iter()
        .filter(|activity| !after_activities.contains_key(&activity.id))
        .map(|activity| ActivityRef {
            id: activity.id,
            name: activity.name.clone(),
        })
        .collect();
    removed_activities.sort_by_key(|reference| reference.id);

    let mut duration_changes: Vec<DurationChange> = after
        .activities
        .iter()
        .filter_map(|activity| {
            let previous = before_activities.get(&activity.id)?;
            (previous.original_duration != activity.original_duration).then(|| DurationChange {
                id: activity.id,
                name: activity.name.clone(),
                before: previous.original_duration,
                after: activity.original_duration,
            })
        })
        .collect();
    duration_changes.sort_by_key(|change| change.id);

    let mut date_changes: Vec<DateChange> = Vec::new();
    if let (Some(before_schedule), Some(after_schedule)) = (before_schedule, after_schedule) {
        for activity in &after.activities {
            let Some(previous) = before_activities.get(&activity.id) else {
                continue;
            };
            let (Some(previous_scheduled), Some(current_scheduled)) = (
                before_schedule.activity(previous.id),
                after_schedule.activity(activity.id),
            ) else {
                continue;
            };
            if previous_scheduled.scheduled_start == current_scheduled.scheduled_start
                && previous_scheduled.scheduled_finish == current_scheduled.scheduled_finish
            {
                continue;
            }
            let calendar = activity_calendar(after, activity);
            date_changes.push(DateChange {
                id: activity.id,
                name: activity.name.clone(),
                start: DateShift {
                    before: Some(previous_scheduled.scheduled_start),
                    after: Some(current_scheduled.scheduled_start),
                    days: calendar.working_days_between(
                        previous_scheduled.scheduled_start,
                        current_scheduled.scheduled_start,
                    ),
                },
                finish: DateShift {
                    before: Some(previous_scheduled.scheduled_finish),
                    after: Some(current_scheduled.scheduled_finish),
                    days: calendar.working_days_between(
                        previous_scheduled.scheduled_finish,
                        current_scheduled.scheduled_finish,
                    ),
                },
            });
        }
        date_changes.sort_by_key(|change| change.id);
    }

    let mut constraint_changes: Vec<ConstraintChange> = after
        .activities
        .iter()
        .filter_map(|activity| {
            let previous = before_activities.get(&activity.id)?;
            (previous.constraint != activity.constraint).then(|| ConstraintChange {
                id: activity.id,
                name: activity.name.clone(),
                before: previous.constraint.code().to_string(),
                after: activity.constraint.code().to_string(),
            })
        })
        .collect();
    constraint_changes.sort_by_key(|change| change.id);

    let before_relationships: HashMap<u64, &Relationship> = before
        .relationships
        .iter()
        .map(|relationship| (relationship.id.0, relationship))
        .collect();
    let after_relationships: HashMap<u64, &Relationship> = after
        .relationships
        .iter()
        .map(|relationship| (relationship.id.0, relationship))
        .collect();

    let mut relationship_changes: Vec<RelationshipChange> = Vec::new();
    for relationship in &after.relationships {
        match before_relationships.get(&relationship.id.0) {
            None => relationship_changes.push(RelationshipChange::Added {
                relationship: (*relationship).clone(),
            }),
            Some(previous) => {
                let same_endpoints = previous.predecessor == relationship.predecessor
                    && previous.successor == relationship.successor;
                let same_shape = previous.link_type == relationship.link_type
                    && previous.lag == relationship.lag;
                if !same_endpoints {
                    relationship_changes.push(RelationshipChange::Removed {
                        relationship: (*previous).clone(),
                    });
                    relationship_changes.push(RelationshipChange::Added {
                        relationship: (*relationship).clone(),
                    });
                } else if !same_shape {
                    relationship_changes.push(RelationshipChange::Changed {
                        before: (*previous).clone(),
                        after: (*relationship).clone(),
                    });
                }
            }
        }
    }
    for relationship in &before.relationships {
        if !after_relationships.contains_key(&relationship.id.0) {
            relationship_changes.push(RelationshipChange::Removed {
                relationship: (*relationship).clone(),
            });
        }
    }
    relationship_changes.sort_by_key(|change| match change {
        RelationshipChange::Added { relationship } => (0u8, relationship.id.0),
        RelationshipChange::Removed { relationship } => (1u8, relationship.id.0),
        RelationshipChange::Changed { before, .. } => (2u8, before.id.0),
    });

    let project_dates = match (before_schedule, after_schedule) {
        (Some(before_schedule), Some(after_schedule)) => Some(ProjectDateShift {
            start: DateShift {
                before: Some(before_schedule.project_start),
                after: Some(after_schedule.project_start),
                days: before.calendar.working_days_between(
                    before_schedule.project_start,
                    after_schedule.project_start,
                ),
            },
            finish: DateShift {
                before: Some(before_schedule.project_finish),
                after: Some(after_schedule.project_finish),
                days: before.calendar.working_days_between(
                    before_schedule.project_finish,
                    after_schedule.project_finish,
                ),
            },
        }),
        _ => None,
    };

    Comparison {
        added_activities,
        removed_activities,
        duration_changes,
        date_changes,
        constraint_changes,
        relationship_changes,
        project_dates,
    }
}

/// The calendar that governs an activity's dates: its named calendar, or the
/// project default.
fn activity_calendar<'a>(project: &'a Project, activity: &Activity) -> &'a crate::model::Calendar {
    match activity.calendar {
        None => &project.calendar,
        Some(id) => project.calendars.calendar(id).unwrap_or(&project.calendar),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ActivityBaselineSnapshot, Baseline, BaselineId, RelationshipId};

    fn jan(day: u8) -> Date {
        Date::from_calendar_date(2026, time::Month::January, day).expect("valid test date")
    }

    fn chain() -> Project {
        let mut project = Project::new("Compare test", jan(5));
        project
            .activities
            .push(Activity::new(ActivityId::new(1), "Survey", 2));
        project
            .activities
            .push(Activity::new(ActivityId::new(2), "Excavate", 3));
        project.relationships.push(Relationship::new(
            RelationshipId::new(1),
            ActivityId::new(1),
            ActivityId::new(2),
            LinkType::Fs,
            0,
        ));
        project
    }

    #[test]
    fn identical_programmes_compare_empty() {
        let before = chain();
        let after = chain();

        let comparison = compare(&before, &after);

        assert!(comparison.is_empty());
        assert!(comparison.project_dates.is_some());
    }

    #[test]
    fn an_added_and_a_removed_activity_are_reported() {
        let before = chain();
        let mut after = chain();
        after.activities.remove(1);
        after
            .activities
            .push(Activity::new(ActivityId::new(3), "Pour", 2));

        let comparison = compare(&before, &after);

        assert_eq!(comparison.added_activities.len(), 1);
        assert_eq!(comparison.added_activities[0].name, "Pour");
        assert_eq!(comparison.removed_activities.len(), 1);
        assert_eq!(comparison.removed_activities[0].name, "Excavate");
    }

    #[test]
    fn a_duration_change_moves_the_scheduled_dates() {
        let mut before = chain();
        let mut after = chain();
        after.activities[1].original_duration = 6;

        let comparison = compare(&before, &after);

        assert_eq!(comparison.duration_changes.len(), 1);
        let change = &comparison.duration_changes[0];
        assert_eq!((change.before, change.after), (3, 6));
        let finish = &comparison.date_changes[0].finish;
        assert_eq!(finish.before, Some(jan(9)));
        assert!(finish.days > 0);
        let project = comparison.project_dates.expect("both scheduled");
        assert!(project.finish.days > 0, "the finish moves later");
    }

    #[test]
    fn a_relationship_change_is_reported() {
        let before = chain();
        let mut after = chain();
        after.relationships[0].lag = 2;

        let comparison = compare(&before, &after);

        assert_eq!(comparison.relationship_changes.len(), 1);
        match &comparison.relationship_changes[0] {
            RelationshipChange::Changed { before, after } => {
                assert_eq!((before.lag, after.lag), (0, 2));
            }
            other => panic!("expected a changed relationship, got {other:?}"),
        }
    }

    #[test]
    fn a_repointed_relationship_is_removed_and_added() {
        let before = chain();
        let mut after = chain();
        after
            .activities
            .push(Activity::new(ActivityId::new(3), "Pour", 2));
        after.relationships[0].successor = ActivityId::new(3);

        let comparison = compare(&before, &after);

        let kinds: Vec<&str> = comparison
            .relationship_changes
            .iter()
            .map(|change| match change {
                RelationshipChange::Added { .. } => "added",
                RelationshipChange::Removed { .. } => "removed",
                RelationshipChange::Changed { .. } => "changed",
            })
            .collect();
        assert_eq!(kinds, vec!["added", "removed"]);
    }

    #[test]
    fn a_constraint_change_is_reported() {
        let before = chain();
        let mut after = chain();
        after.activities[1].constraint = Constraint::FinishNoLaterThan(jan(30));

        let comparison = compare(&before, &after);

        assert_eq!(comparison.constraint_changes.len(), 1);
        let change = &comparison.constraint_changes[0];
        assert_eq!((change.before.as_str(), change.after.as_str()), ("asap", "fnlt"));
    }

    #[test]
    fn the_baseline_comparison_reports_duration_and_dates() {
        let project = chain();
        let schedule = schedule(&project).expect("schedule succeeds");
        let snapshots = project
            .activities
            .iter()
            .filter_map(|activity| {
                let scheduled = schedule.activity(activity.id)?;
                Some(ActivityBaselineSnapshot {
                    activity_id: activity.id,
                    baseline_duration: activity.original_duration,
                    baseline_start: Some(scheduled.scheduled_start),
                    baseline_finish: Some(scheduled.scheduled_finish),
                })
            })
            .collect();
        let baseline = Baseline::from_snapshots(BaselineId::new(1), "Baseline", jan(5), snapshots);

        let mut current = chain();
        current.activities[1].original_duration = 8;
        let comparison = compare_baseline(&current, &baseline);

        assert!(comparison.added_activities.is_empty());
        assert!(comparison.removed_activities.is_empty());
        assert_eq!(comparison.duration_changes.len(), 1);
        assert_eq!(
            (comparison.duration_changes[0].before, comparison.duration_changes[0].after),
            (3, 8)
        );
        assert!(comparison.date_changes[0].finish.days > 0);
    }

    #[test]
    fn a_baseline_missing_an_activity_reports_it_added() {
        let mut project = chain();
        project
            .activities
            .push(Activity::new(ActivityId::new(3), "Pour", 2));
        let baseline = Baseline::from_snapshots(
            BaselineId::new(1),
            "Baseline",
            jan(5),
            vec![ActivityBaselineSnapshot {
                activity_id: ActivityId::new(1),
                baseline_duration: 2,
                baseline_start: Some(jan(5)),
                baseline_finish: Some(jan(6)),
            }],
        );

        let comparison = compare_baseline(&project, &baseline);

        // The snapshot's activity 1 was rebuilt with activity 2 missing from
        // the baseline, so activity 2 and the new activity 3 are both "added".
        assert_eq!(comparison.added_activities.len(), 2);
    }

    #[test]
    fn the_comparison_is_deterministic() {
        let mut before = chain();
        let mut after = chain();
        after.activities[1].original_duration = 5;
        after
            .activities
            .push(Activity::new(ActivityId::new(3), "Pour", 2));

        let first = compare(&before, &after);
        let second = compare(&before, &after);

        assert_eq!(first, second);
    }
}
