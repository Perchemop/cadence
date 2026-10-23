//! Property-based round-trip tests for the interchange formats.
//!
//! Every format is a documented subset, so the properties here check exactly
//! what each one promises:
//!
//! * **JSON** and **MSPDI** carry the whole project, so a round trip must
//!   preserve activities (with progress, costs and actual dates), links,
//!   resources with kinds and capacities, assignments and baselines.
//! * **CSV** and **MPX** carry the plan, not the money: the activity graph,
//!   links and assignments must survive, while costs and progress are allowed
//!   to be absent. CSV additionally loses resource kinds and capacities, which
//!   is documented in the module.
//!
//! Column-and-element level tests live in each module; these properties are
//! what guards the formats against arbitrary shapes.

use std::collections::BTreeSet;

use cadence_core::model::{
    Activity, ActivityId, Constraint, LinkType, Project, Relationship, RelationshipId, Resource,
    ResourceId, ResourceKind,
};
use cadence_interchange::{csv, json, mpx, mspdi};
use proptest::prelude::*;
use time::{Date, Month};

const MAX_ACTIVITIES: usize = 6;

fn origin() -> Date {
    Date::from_calendar_date(2026, Month::January, 5).expect("valid origin")
}

fn day(offset: i64) -> Date {
    Date::from_julian_day(origin().to_julian_day() + offset as i32).expect("valid date")
}

fn link_type(raw: u8) -> LinkType {
    LinkType::ALL[usize::from(raw % 4)]
}

/// An arbitrary project: activities, acyclic links, resources with kinds and
/// capacities, assignments, progress, costs and actual dates.
///
/// `keep_details` decides whether money, progress and actual dates are set,
/// which is what separates the full-fidelity round trips from the plan-only
/// ones.
fn build(
    durations: &[u32],
    raw_links: &[(usize, usize, u8, i64)],
    kinds: &[u8],
    capacities: &[u32],
    masks: &[u8],
    money: &[Option<i64>],
    percents: &[Option<i64>],
    actual_offsets: &[Option<i64>],
) -> Project {
    let mut project = Project::new("Round trip", origin());

    for (index, duration) in durations.iter().enumerate() {
        let mut activity =
            Activity::new(ActivityId(index as u64 + 1), format!("A{index}"), *duration);
        if let Some(budget) = money.get(index).copied().flatten() {
            activity.budget_at_completion = Some(budget);
            activity.actual_cost = Some(budget / 2);
        }
        if let Some(percent) = percents.get(index).copied().flatten() {
            activity.percent_complete = Some(percent);
        }
        if let Some(offset) = actual_offsets.get(index).copied().flatten() {
            if let Some(start) = origin().checked_add(time::Duration::days(offset)) {
                activity.actual_start = Some(start);
                activity.actual_finish = Some(day(offset + i64::from(*duration)));
            }
        }
        project.activities.push(activity);
    }

    let mut next_relationship = 1u64;
    for &(left, right, kind, lag) in raw_links {
        if left >= durations.len() || right >= durations.len() || left == right {
            continue;
        }
        let (predecessor, successor) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        project.relationships.push(Relationship {
            id: RelationshipId(next_relationship),
            predecessor: ActivityId(predecessor as u64 + 1),
            successor: ActivityId(successor as u64 + 1),
            link_type: link_type(kind),
            lag,
        });
        next_relationship += 1;
    }

    for (index, capacity) in capacities.iter().enumerate() {
        let kind = match kinds.get(index).copied().unwrap_or(0) % 3 {
            0 => ResourceKind::Labour,
            1 => ResourceKind::Equipment,
            _ => ResourceKind::Material,
        };
        let mut resource =
            Resource::with_capacity(ResourceId(index as u64 + 1), format!("R{index}"), *capacity);
        resource.kind = kind;
        project.resources.push(resource);
    }

    for (index, activity) in project.activities.iter_mut().enumerate() {
        let mask = masks.get(index).copied().unwrap_or(0);
        for bit in 0..capacities.len() {
            if mask & (1 << bit) != 0 {
                activity.resources.push(ResourceId(bit as u64 + 1));
            }
        }
    }

    project
}

/// The plan half of a project: what every format promises to preserve.
fn plan(
    project: &Project,
) -> (
    Vec<(u64, String, u32, Constraint)>,
    BTreeSet<(u64, u64, u8, i64)>,
) {
    let activities = project
        .activities
        .iter()
        .map(|activity| {
            (
                activity.id.0,
                activity.name.clone(),
                activity.original_duration,
                activity.constraint,
            )
        })
        .collect();
    let links = project
        .relationships
        .iter()
        .map(|relationship| {
            let kind = match relationship.link_type {
                LinkType::Fs => 0,
                LinkType::Ss => 1,
                LinkType::Ff => 2,
                LinkType::Sf => 3,
            };
            (
                relationship.predecessor.0,
                relationship.successor.0,
                kind,
                relationship.lag,
            )
        })
        .collect();
    (activities, links)
}

/// Assignment pairs expressed by resource *name*, because formats are free to
/// renumber resource identifiers.
fn assignment_names(project: &Project) -> BTreeSet<(u64, String)> {
    let mut pairs = BTreeSet::new();
    for activity in &project.activities {
        for resource in &activity.resources {
            let name = project
                .resource(*resource)
                .map(|resource| resource.name.clone())
                .unwrap_or_else(|| format!("unknown-{}", resource.0));
            pairs.insert((activity.id.0, name));
        }
    }
    pairs
}

fn assert_plan_survived(original: &Project, loaded: &Project) {
    let (expected_activities, expected_links) = plan(original);
    let (actual_activities, actual_links) = plan(loaded);
    assert_eq!(actual_activities, expected_activities);
    assert_eq!(actual_links, expected_links);
    assert_eq!(assignment_names(loaded), assignment_names(original));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// JSON carries everything, so a round trip must be exact.
    #[test]
    fn json_round_trips_the_whole_project(
        durations in prop::collection::vec(0u32..6, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, -2i64..3),
            0..10,
        ),
        capacities in prop::collection::vec(1u32..5, 0..4),
        kinds in prop::collection::vec(0u8..6, 0..4),
        masks in prop::collection::vec(0u8..16, 1..MAX_ACTIVITIES),
        money in prop::collection::vec(prop::option::of(0i64..1_000_000), 1..MAX_ACTIVITIES),
        percents in prop::collection::vec(prop::option::of(0i64..10_001), 1..MAX_ACTIVITIES),
        actual_offsets in prop::collection::vec(prop::option::of(0i64..30), 1..MAX_ACTIVITIES),
    ) {
        let project = build(&durations, &raw_links, &kinds, &capacities, &masks, &money, &percents, &actual_offsets);

        let text = json::export(&project);
        let loaded = json::import(&text).expect("the exported JSON re-imports");

        prop_assert_eq!(&loaded.name, &project.name);
        prop_assert_eq!(loaded.start_date, project.start_date);
        prop_assert_eq!(&loaded.activities, &project.activities);
        prop_assert_eq!(&loaded.relationships, &project.relationships);
        prop_assert_eq!(&loaded.resources, &project.resources);
    }

    /// MSPDI converts durations at eight hours per day and lag to whole
    /// working days, both of which are exact for whole-day input.
    #[test]
    fn mspdi_round_trips_activities_resources_and_assignments(
        durations in prop::collection::vec(0u32..6, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, -2i64..3),
            0..10,
        ),
        capacities in prop::collection::vec(1u32..5, 0..4),
        kinds in prop::collection::vec(0u8..6, 0..4),
        masks in prop::collection::vec(0u8..16, 1..MAX_ACTIVITIES),
        money in prop::collection::vec(prop::option::of(0i64..1_000_000), 1..MAX_ACTIVITIES),
        percents in prop::collection::vec(prop::option::of(0i64..10_001), 1..MAX_ACTIVITIES),
        actual_offsets in prop::collection::vec(prop::option::of(0i64..30), 1..MAX_ACTIVITIES),
    ) {
        let project = build(&durations, &raw_links, &kinds, &capacities, &masks, &money, &percents, &actual_offsets);

        let text = mspdi::export(&project).expect("export");
        let loaded = mspdi::import(&text).expect("the exported XML re-imports");

        prop_assert_eq!(&loaded.name, &project.name);
        prop_assert_eq!(loaded.start_date, project.start_date);
        prop_assert_eq!(&loaded.activities, &project.activities);
        prop_assert_eq!(&loaded.resources, &project.resources);
        prop_assert_eq!(assignment_names(&loaded), assignment_names(&project));
        let (_, expected_links) = plan(&project);
        let (_, actual_links) = plan(&loaded);
        prop_assert_eq!(actual_links, expected_links);
    }

    /// CSV carries the plan, costs and progress in one row per activity, but
    /// no resource kinds or capacities.
    #[test]
    fn csv_round_trips_the_plan_progress_and_costs(
        durations in prop::collection::vec(0u32..6, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, -2i64..3),
            0..10,
        ),
        capacities in prop::collection::vec(1u32..5, 0..4),
        masks in prop::collection::vec(0u8..16, 1..MAX_ACTIVITIES),
        money in prop::collection::vec(prop::option::of(0i64..1_000_000), 1..MAX_ACTIVITIES),
        percents in prop::collection::vec(prop::option::of(0i64..10_001), 1..MAX_ACTIVITIES),
        actual_offsets in prop::collection::vec(prop::option::of(0i64..30), 1..MAX_ACTIVITIES),
    ) {
        let project = build(&durations, &raw_links, &[0], &capacities, &masks, &money, &percents, &actual_offsets);

        let text = csv::export(&project).expect("export");
        let loaded = csv::import(&text).expect("the exported CSV re-imports");

        prop_assert_eq!(&loaded.activities, &project.activities);
        assert_plan_survived(&project, &loaded);
    }

    /// MPX carries the plan and the resource dictionary, but no money.
    #[test]
    fn mpx_round_trips_the_plan_and_resources(
        durations in prop::collection::vec(0u32..6, 1..MAX_ACTIVITIES),
        raw_links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, -2i64..3),
            0..10,
        ),
        capacities in prop::collection::vec(1u32..5, 0..4),
        kinds in prop::collection::vec(0u8..6, 0..4),
        masks in prop::collection::vec(0u8..16, 1..MAX_ACTIVITIES),
    ) {
        let project = build(&durations, &raw_links, &kinds, &capacities, &masks, &[], &[], &[]);

        let text = mpx::export(&project).expect("export");
        let loaded = mpx::import(&text).expect("the exported MPX re-imports");

        assert_plan_survived(&project, &loaded);
        prop_assert_eq!(&loaded.resources, &project.resources);
    }
}
