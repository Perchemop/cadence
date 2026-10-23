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

use cadence_core::calendar_set::{CalendarId, CalendarSet};
use cadence_core::model::{
    Activity, ActivityId, Calendar, Constraint, LinkType, Project, Relationship, RelationshipId,
    Resource, ResourceId, ResourceKind,
};
use cadence_core::wbs::{WbsId, WbsTree};
use cadence_interchange::{csv, json, mpx, mspdi};
use proptest::prelude::*;
use time::{Date, Month};

const MAX_ACTIVITIES: usize = 6;

/// Activities as every format must carry them: identity, name, duration and
/// constraint.
type ActivityPlan = Vec<(u64, String, u32, Constraint)>;

/// Links as every format must carry them, ordered so that renumbering on
/// import does not matter.
type LinkPlan = BTreeSet<(u64, u64, u8, i64)>;

fn origin() -> Date {
    Date::from_calendar_date(2026, Month::January, 5).expect("valid origin")
}

fn link_type(raw: u8) -> LinkType {
    LinkType::ALL[usize::from(raw % 4)]
}

/// One generated project shape.
///
/// The generator values are grouped so that the builder takes a single
/// argument, and the detail vectors are empty for the formats that do not
/// carry progress, costs or actual dates.
#[derive(Debug, Default)]
struct Shape {
    durations: Vec<u32>,
    links: Vec<(usize, usize, u8, i64)>,
    capacities: Vec<u32>,
    kinds: Vec<u8>,
    masks: Vec<u8>,
    money: Vec<Option<i64>>,
    percents: Vec<Option<i64>>,
    actual_offsets: Vec<Option<i64>>,
}

/// Build an arbitrary project: activities, acyclic links, resources with kinds
/// and capacities, assignments, and (when `money`, `percents` and
/// `actual_offsets` say so) progress, costs and actual dates.
fn build(shape: &Shape) -> Project {
    let mut project = Project::new("Round trip", origin());

    for (index, duration) in shape.durations.iter().enumerate() {
        let mut activity =
            Activity::new(ActivityId(index as u64 + 1), format!("A{index}"), *duration);
        if let Some(budget) = shape.money.get(index).copied().flatten() {
            activity.budget_at_completion = Some(budget);
            activity.actual_cost = Some(budget / 2);
        }
        if let Some(percent) = shape.percents.get(index).copied().flatten() {
            activity.percent_complete = Some(percent);
        }
        if let Some(offset) = shape.actual_offsets.get(index).copied().flatten() {
            if let Some(start) = origin().checked_add(time::Duration::days(offset)) {
                activity.actual_start = Some(start);
                activity.actual_finish = Some(
                    start
                        .checked_add(time::Duration::days(i64::from(*duration)))
                        .unwrap_or(start),
                );
            }
        }
        project.activities.push(activity);
    }

    let mut next_relationship = 1u64;
    for &(left, right, kind, lag) in &shape.links {
        if left >= shape.durations.len() || right >= shape.durations.len() || left == right {
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

    for (index, capacity) in shape.capacities.iter().enumerate() {
        let kind = match shape.kinds.get(index).copied().unwrap_or(0) % 3 {
            0 => ResourceKind::Labour,
            1 => ResourceKind::Equipment,
            _ => ResourceKind::Material,
        };
        let mut resource =
            Resource::with_capacity(ResourceId(index as u64 + 1), format!("R{index}"), *capacity);
        resource.kind = kind;
        project.resources.push(resource);
    }

    // Assignments only ever reference resources that exist, because the
    // formats are free to drop dangling identifiers (CSV does).
    for (index, activity) in project.activities.iter_mut().enumerate() {
        let mask = shape.masks.get(index).copied().unwrap_or(0);
        for bit in 0..shape.capacities.len() {
            if mask & (1 << bit) != 0 {
                activity.resources.push(ResourceId(bit as u64 + 1));
            }
        }
    }

    project
}

/// The plan half of a project: what every format promises to preserve.
fn plan(project: &Project) -> (ActivityPlan, LinkPlan) {
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

/// Everything CSV and MSPDI promise about progress and money, with resource
/// identifiers normalised: CSV renumbers the dictionary (assignments are
/// matched by name), so raw assignment vectors must not be compared.
fn assert_progress_and_costs_survived(original: &Project, loaded: &Project) {
    assert_eq!(loaded.activities.len(), original.activities.len());
    for (expected, actual) in original.activities.iter().zip(&loaded.activities) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.original_duration, expected.original_duration);
        assert_eq!(actual.percent_complete, expected.percent_complete);
        assert_eq!(actual.budget_at_completion, expected.budget_at_completion);
        assert_eq!(actual.actual_cost, expected.actual_cost);
        assert_eq!(actual.actual_start, expected.actual_start);
        assert_eq!(actual.actual_finish, expected.actual_finish);
    }
}

/// Percent complete rounded to the whole percentage MSPDI can store.
fn whole_percent(basis_points: Option<i64>) -> Option<i64> {
    basis_points.map(|value| (value + 50) / 100 * 100)
}

/// A WBS tree compared by structure: import rebuilds nodes in preorder while a
/// hand-built tree keeps insertion order, and both describe the same tree.
fn wbs_shape(tree: &WbsTree) -> Vec<(u64, String, Option<u64>, Vec<u64>)> {
    fn walk(tree: &WbsTree, id: WbsId, shape: &mut Vec<(u64, String, Option<u64>, Vec<u64>)>) {
        let Some(node) = tree.node(id) else {
            return;
        };
        shape.push((
            node.id.0,
            node.name.clone(),
            node.parent.map(|parent| parent.0),
            node.children.iter().map(|child| child.0).collect(),
        ));
        for child in &node.children {
            walk(tree, *child, shape);
        }
    }
    let mut shape = Vec::new();
    walk(tree, tree.root(), &mut shape);
    shape
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// JSON carries everything, so a round trip must be exact.
    #[test]
    fn json_round_trips_the_whole_project(
        durations in prop::collection::vec(0u32..6, 1..MAX_ACTIVITIES),
        links in prop::collection::vec(
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
        let shape = Shape {
            durations,
            links,
            capacities,
            kinds,
            masks,
            money,
            percents,
            actual_offsets,
        };
        let project = build(&shape);

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
        links in prop::collection::vec(
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
        let shape = Shape {
            durations,
            links,
            capacities,
            kinds,
            masks,
            money,
            percents,
            actual_offsets,
        };
        let project = build(&shape);

        let text = mspdi::export(&project).expect("export");
        let loaded = mspdi::import(&text).expect("the exported XML re-imports");

        prop_assert_eq!(&loaded.name, &project.name);
        prop_assert_eq!(loaded.start_date, project.start_date);
        // MSPDI stores percent complete as a whole percentage, so compare
        // against the quantised value rather than the exact basis points.
        prop_assert_eq!(loaded.activities.len(), project.activities.len());
        for (expected, actual) in project.activities.iter().zip(&loaded.activities) {
            let mut quantised = expected.clone();
            quantised.percent_complete = whole_percent(expected.percent_complete);
            prop_assert_eq!(actual, &quantised);
        }
        prop_assert_eq!(&loaded.resources, &project.resources);
        assert_plan_survived(&project, &loaded);
    }

    /// CSV carries the plan, costs and progress in one row per activity, but
    /// no resource kinds or capacities.
    #[test]
    fn csv_round_trips_the_plan_progress_and_costs(
        durations in prop::collection::vec(0u32..6, 1..MAX_ACTIVITIES),
        links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, -2i64..3),
            0..10,
        ),
        capacities in prop::collection::vec(1u32..5, 0..4),
        masks in prop::collection::vec(0u8..16, 1..MAX_ACTIVITIES),
        money in prop::collection::vec(prop::option::of(0i64..1_000_000), 1..MAX_ACTIVITIES),
        percents in prop::collection::vec(prop::option::of(0i64..10_001), 1..MAX_ACTIVITIES),
        actual_offsets in prop::collection::vec(prop::option::of(0i64..30), 1..MAX_ACTIVITIES),
    ) {
        let shape = Shape {
            durations,
            links,
            capacities,
            kinds: Vec::new(),
            masks,
            money,
            percents,
            actual_offsets,
        };
        let project = build(&shape);

        let text = csv::export(&project).expect("export");
        let loaded = csv::import(&text).expect("the exported CSV re-imports");

        assert_progress_and_costs_survived(&project, &loaded);
        assert_plan_survived(&project, &loaded);
    }

    /// MPX carries the plan and the resource dictionary, but no money.
    #[test]
    fn mpx_round_trips_the_plan_and_resources(
        durations in prop::collection::vec(0u32..6, 1..MAX_ACTIVITIES),
        links in prop::collection::vec(
            (0usize..MAX_ACTIVITIES, 0usize..MAX_ACTIVITIES, 0u8..4, -2i64..3),
            0..10,
        ),
        capacities in prop::collection::vec(1u32..5, 0..4),
        kinds in prop::collection::vec(0u8..6, 0..4),
        masks in prop::collection::vec(0u8..16, 1..MAX_ACTIVITIES),
    ) {
        let shape = Shape {
            durations,
            links,
            capacities,
            kinds,
            masks,
            ..Shape::default()
        };
        let project = build(&shape);

        let text = mpx::export(&project).expect("export");
        let loaded = mpx::import(&text).expect("the exported MPX re-imports");

        prop_assert_eq!(&loaded.resources, &project.resources);
        assert_plan_survived(&project, &loaded);
    }

    /// JSON is the only interchange format that carries the WBS tree and named
    /// calendars, so those must survive exactly, sibling order included.
    #[test]
    fn json_round_trips_the_work_breakdown_and_calendars(
        durations in prop::collection::vec(1u32..4, 1..MAX_ACTIVITIES),
        child_parents in prop::collection::vec(0u8..4, 0..4),
        node_choices in prop::collection::vec(prop::option::of(0u8..5), 1..MAX_ACTIVITIES),
        workweeks in prop::collection::vec(1u8..127, 0..3),
        calendar_choices in prop::collection::vec(prop::option::of(0u8..3), 1..MAX_ACTIVITIES),
        holiday_count in 0usize..3,
    ) {
        let mut project = Project::new("Work breakdown", origin());

        // A tree: the root plus one node per generated parent choice. Parents
        // are always already created, so the tree is acyclic by construction.
        let mut tree = WbsTree::new();
        let mut created = vec![tree.root()];
        for (index, parent_choice) in child_parents.iter().enumerate() {
            let parent = created[usize::from(*parent_choice) % created.len()];
            let id = WbsId(index as u64 + 2);
            tree.add(id, format!("W{index}"), parent).expect("valid node");
            created.push(id);
        }

        // Named calendars, each with its own working week and holidays.
        let mut calendars = CalendarSet::new();
        for (index, mask) in workweeks.iter().enumerate() {
            let workweek = [
                true,
                mask & 0b1 != 0,
                mask & 0b10 != 0,
                mask & 0b100 != 0,
                mask & 0b1000 != 0,
                mask & 0b10000 != 0,
                mask & 0b100000 != 0,
            ];
            let mut holidays = Vec::new();
            for offset in 0..holiday_count {
                let days = index as i64 + offset as i64 + 1;
                if let Some(date) = origin().checked_add(time::Duration::days(days)) {
                    holidays.push(date);
                }
            }
            let calendar = Calendar::new(workweek)
                .expect("Monday always works")
                .with_holidays(holidays);
            calendars
                .add(CalendarId(index as u64 + 1), format!("C{index}"), calendar)
                .expect("distinct calendar");
        }

        for (index, duration) in durations.iter().enumerate() {
            let mut activity =
                Activity::new(ActivityId(index as u64 + 1), format!("A{index}"), *duration);
            activity.wbs = node_choices
                .get(index)
                .copied()
                .flatten()
                .map(|choice| created[usize::from(choice) % created.len()]);
            activity.calendar = match calendar_choices.get(index).copied().flatten() {
                Some(choice) if !workweeks.is_empty() => {
                    Some(CalendarId(u64::from(choice) % workweeks.len() as u64 + 1))
                }
                _ => None,
            };
            project.activities.push(activity);
        }

        project.wbs = tree;
        project.calendars = calendars;

        let loaded = json::import(&json::export(&project)).expect("round trip");

        prop_assert_eq!(wbs_shape(&loaded.wbs), wbs_shape(&project.wbs));
        prop_assert_eq!(loaded.calendars.calendars(), project.calendars.calendars());
        prop_assert_eq!(&loaded.activities, &project.activities);
    }
}
