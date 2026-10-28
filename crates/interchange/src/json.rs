//! JSON import and export.
//!
//! JSON is Cadence's own open interchange schema: unlike MSPDI, MPX and CSV it
//! is not modelled on another application's dialect, so it can carry exactly
//! what Cadence models — the project settings, the WBS tree, named calendars,
//! the activity list with constraints, progress and calendar/WBS membership,
//! relationships with lag, the resource dictionary with capacities,
//! assignments and baselines — without lossy unit conversions. It is the only
//! interchange format that is complete in this sense; the others deliberately
//! carry subsets.
//!
//! Schema version **2** added `wbs`, `calendars` and the per-activity `wbs`
//! and `calendar` members. Version 1 documents still load, with an empty WBS
//! (just its root) and no named calendars, because version 1 had no way to
//! express them; the version is checked so an older build refuses a newer
//! document instead of silently dropping the tree. Unknown members are
//! ignored, so the schema can keep growing.
//!
//! The document looks like this (`cadence` is the schema version):
//!
//! ```json
//! {
//!   "cadence": 2,
//!   "project": {
//!     "name": "Bridge & Road",
//!     "start_date": "2026-01-05",
//!     "wbs": [
//!       {"id": 1, "name": "Bridge & Road", "parent": null},
//!       {"id": 2, "name": "Design", "parent": 1}
//!     ],
//!     "calendars": [
//!       {
//!         "id": 1, "name": "Six day",
//!         "workweek": [true, true, true, true, true, true, false],
//!         "holidays": ["2026-01-01"]
//!       }
//!     ],
//!     "activities": [
//!       {
//!         "id": 1, "name": "Survey", "duration": 2,
//!         "constraint": "ASAP", "constraint_date": null,
//!         "percent_complete": 50.0,
//!         "budget_at_completion": 1500.00, "actual_cost": 750.50,
//!         "actual_start": "2026-01-05", "actual_finish": "2026-01-06",
//!         "wbs": 2, "calendar": 1, "resources": [1]
//!       }
//!     ],
//!     "relationships": [
//!       {"id": 1, "predecessor": 1, "successor": 2, "type": "FS", "lag": 1}
//!     ],
//!     "resources": [
//!       {"id": 1, "name": "Crane", "kind": "equipment", "capacity": 2}
//!     ],
//!     "baselines": [
//!       {
//!         "id": 1, "name": "Baseline 1", "created_at": "2026-01-05",
//!         "snapshots": [
//!           {"activity_id": 1, "duration": 2,
//!            "start": "2026-01-05", "finish": "2026-01-06"}
//!         ]
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! `wbs` is written in preorder (the root first, so every parent precedes its
//! children), and `workweek` is seven Monday-first booleans.
//! Constraint codes are the same `ASAP`/`ALAP`/`SNET`/`SNLT`/`FNET`/`FNLT`
//! set used by MPX and CSV, link types are `FS`/`SS`/`FF`/`SF`, money is a
//! decimal amount converted at the boundary, and `percent_complete` is a
//! percentage. Scheduled dates are not written: they are derived data (D9),
//! and the engine recomputes them from logic, calendars and constraints.
//!
//! The reader and writer live in this crate (see D14): the reader is a small
//! recursive-descent parser with a nesting limit, covered by its own tests,
//! and no JSON library is pulled into the build.

use std::fmt::Write as _;
use std::iter::Peekable;
use std::str::Chars;

use cadence_core::calendar_set::{CalendarId, CalendarSet, NamedCalendar};
use cadence_core::model::{
    Activity, ActivityBaselineSnapshot, ActivityId, Baseline, BaselineId, Calendar, Constraint,
    LinkType, Project, Relationship, RelationshipId, Resource, ResourceId, ResourceKind,
};
use cadence_core::wbs::{WbsId, WbsTree};
use time::Date;

use crate::{default_start_date, format_date, parse_date, InterchangeError};

/// The schema version this build reads and writes.
const SCHEMA_VERSION: i64 = 2;

/// The oldest schema version this build still reads.
const OLDEST_SCHEMA_VERSION: i64 = 1;

/// Maximum nesting depth accepted by the reader.
const MAX_DEPTH: usize = 64;

/// Import a Cadence JSON document.
///
/// # Errors
///
/// Returns [`InterchangeError`] when the document is not valid JSON, is not a
/// Cadence document of the supported version, or a required member is missing
/// or carries a value the schema does not allow.
pub fn import(text: &str) -> Result<Project, InterchangeError> {
    let document = parse(text)?;
    if !matches!(document, Value::Object(_)) {
        return Err(InterchangeError::Unsupported(
            "expected a single JSON object at the top level".to_string(),
        ));
    }
    let root = &document;

    let version = root
        .get("cadence")
        .and_then(Value::as_i64)
        .ok_or_else(|| InterchangeError::MissingField("cadence".to_string()))?;
    if !(OLDEST_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&version) {
        return Err(InterchangeError::Unsupported(format!(
            "Cadence JSON schema version {version}, which this build does not read"
        )));
    }

    let project = root
        .get("project")
        .ok_or_else(|| InterchangeError::MissingField("project".to_string()))?;
    let name = project
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("Imported project")
        .to_string();
    let start_text = project
        .get("start_date")
        .and_then(Value::as_str)
        .ok_or_else(|| InterchangeError::MissingField("project/start_date".to_string()))?;
    let start_date = parse_date(start_text).ok_or_else(|| InterchangeError::InvalidField {
        field: "project/start_date".to_string(),
        value: start_text.to_string(),
    })?;

    let mut imported = Project::new(name, start_date);
    imported.wbs = import_wbs(project.get("wbs"))?;
    imported.calendars = import_calendars(project.get("calendars"))?;

    if let Some(activities) = project.get("activities").and_then(Value::as_array) {
        for activity in activities {
            imported.activities.push(import_activity(activity)?);
        }
    }

    let mut next_relationship = 1u64;
    if let Some(relationships) = project.get("relationships").and_then(Value::as_array) {
        for relationship in relationships {
            let id = relationship
                .get("id")
                .and_then(Value::as_u64)
                .map(RelationshipId)
                .unwrap_or(RelationshipId(next_relationship));
            imported.relationships.push(Relationship {
                id,
                predecessor: require_activity_id(relationship, "predecessor")?,
                successor: require_activity_id(relationship, "successor")?,
                link_type: import_link_type(relationship.get("type"))?,
                lag: relationship.get("lag").and_then(Value::as_i64).unwrap_or(0),
            });
            next_relationship = imported.next_relationship_id().0;
        }
    }

    if let Some(resources) = project.get("resources").and_then(Value::as_array) {
        for resource in resources {
            imported.resources.push(import_resource(resource)?);
        }
    }

    if let Some(baselines) = project.get("baselines").and_then(Value::as_array) {
        for baseline in baselines {
            imported.baselines.push(import_baseline(baseline)?);
        }
    }

    // Membership is validated here rather than left for the project file to
    // reject later, so an import either produces a usable project or explains
    // exactly what is wrong with the document.
    for activity in &imported.activities {
        if let Some(wbs) = activity.wbs {
            if imported.wbs.node(wbs).is_none() {
                return Err(missing_reference(
                    "activity/wbs",
                    format!(
                        "activity {} references missing WBS node {}",
                        activity.id, wbs.0
                    ),
                ));
            }
        }
        if let Some(calendar) = activity.calendar {
            if imported.calendars.named(calendar).is_none() {
                return Err(missing_reference(
                    "activity/calendar",
                    format!(
                        "activity {} references missing calendar {}",
                        activity.id, calendar.0
                    ),
                ));
            }
        }
    }

    Ok(imported)
}

fn missing_reference(field: &str, message: String) -> InterchangeError {
    InterchangeError::InvalidField {
        field: field.to_string(),
        value: message,
    }
}

/// Read the WBS tree. Nodes are written in preorder, so a parent always
/// precedes its children and the root comes first.
fn import_wbs(value: Option<&Value>) -> Result<WbsTree, InterchangeError> {
    let mut tree = WbsTree::new();
    let Some(nodes) = value.and_then(Value::as_array) else {
        return Ok(tree);
    };

    let mut root_seen = false;
    for node in nodes {
        let id = WbsId(require_u64(node, "id", "wbs node")?);
        let name = node
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("WBS node")
            .to_string();

        match node.get("parent").and_then(Value::as_u64) {
            None => {
                if root_seen {
                    return Err(missing_reference(
                        "wbs/parent",
                        format!("WBS node {id} is a second root"),
                    ));
                }
                if id != tree.root() {
                    return Err(missing_reference(
                        "wbs/id",
                        format!("the WBS root must have id {}, found {id}", tree.root().0),
                    ));
                }
                tree.rename(id, name)
                    .map_err(|error| missing_reference("wbs/name", error.to_string()))?;
                root_seen = true;
            }
            Some(parent) => {
                if !root_seen {
                    return Err(missing_reference(
                        "wbs",
                        "the root must be the first WBS node".to_string(),
                    ));
                }
                tree.add(id, name, WbsId(parent))
                    .map_err(|error| missing_reference("wbs", error.to_string()))?;
            }
        }
    }

    Ok(tree)
}

/// Read the named calendars, each with its Monday-first working week and its
/// holiday exceptions.
fn import_calendars(value: Option<&Value>) -> Result<CalendarSet, InterchangeError> {
    let mut set = CalendarSet::new();
    let Some(calendars) = value.and_then(Value::as_array) else {
        return Ok(set);
    };

    for entry in calendars {
        let id = CalendarId(require_u64(entry, "id", "calendar")?);
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Calendar")
            .to_string();

        let weekdays = entry
            .get("workweek")
            .and_then(Value::as_array)
            .filter(|days| days.len() == 7)
            .ok_or_else(|| {
                missing_reference(
                    "calendar/workweek",
                    format!("calendar {id} needs seven Monday-first booleans"),
                )
            })?;
        let mut workweek = [false; 7];
        for (index, working) in weekdays.iter().enumerate() {
            workweek[index] = match working {
                Value::Bool(working) => *working,
                _ => {
                    return Err(missing_reference(
                        "calendar/workweek",
                        format!("calendar {id} has a non-boolean weekday"),
                    ))
                }
            };
        }
        let mut calendar = Calendar::new(workweek)
            .map_err(|error| missing_reference("calendar/workweek", error.to_string()))?;

        if let Some(days) = entry.get("holidays").and_then(Value::as_array) {
            let mut holidays = Vec::with_capacity(days.len());
            for day in days {
                let text = day.as_str().ok_or_else(|| {
                    missing_reference(
                        "calendar/holidays",
                        format!("calendar {id} has a non-string holiday"),
                    )
                })?;
                let date = parse_date(text).ok_or_else(|| {
                    missing_reference(
                        "calendar/holidays",
                        format!("calendar {id} has an invalid holiday \"{text}\""),
                    )
                })?;
                holidays.push(date);
            }
            calendar = calendar.with_holidays(holidays);
        }

        set.add(id, name, calendar)
            .map_err(|error| missing_reference("calendar", error.to_string()))?;
    }

    Ok(set)
}

/// Export a project as Cadence JSON.
///
/// Derived schedule fields are deliberately not written (D9); the reader
/// recomputes the schedule from logic, calendars and constraints.
#[must_use]
pub fn export(project: &Project) -> String {
    let document = Value::Object(vec![
        (
            "cadence".to_string(),
            Value::Number(Number::Int(SCHEMA_VERSION)),
        ),
        (
            "project".to_string(),
            Value::Object(vec![
                ("name".to_string(), Value::String(project.name.clone())),
                (
                    "start_date".to_string(),
                    Value::String(project.start_date.to_string()),
                ),
                (
                    "wbs".to_string(),
                    Value::Array(
                        preorder_nodes(&project.wbs)
                            .into_iter()
                            .filter_map(|id| project.wbs.node(id))
                            .map(wbs_node_json)
                            .collect(),
                    ),
                ),
                (
                    "calendars".to_string(),
                    Value::Array(
                        project
                            .calendars
                            .calendars()
                            .iter()
                            .map(calendar_json)
                            .collect(),
                    ),
                ),
                (
                    "activities".to_string(),
                    Value::Array(project.activities.iter().map(activity_json).collect()),
                ),
                (
                    "relationships".to_string(),
                    Value::Array(
                        project
                            .relationships
                            .iter()
                            .map(relationship_json)
                            .collect(),
                    ),
                ),
                (
                    "resources".to_string(),
                    Value::Array(project.resources.iter().map(resource_json).collect()),
                ),
                (
                    "baselines".to_string(),
                    Value::Array(project.baselines.iter().map(baseline_json).collect()),
                ),
            ]),
        ),
    ]);

    let mut output = String::new();
    write_value(&document, 0, &mut output);
    output.push('\n');
    output
}

fn import_activity(value: &Value) -> Result<Activity, InterchangeError> {
    let id = require_u64(value, "id", "activity")?;
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let duration = value
        .get("duration")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .min(u64::from(u32::MAX)) as u32;
    let constraint = import_constraint(
        value.get("constraint").and_then(Value::as_str),
        value.get("constraint_date").and_then(Value::as_str),
    )?;

    let mut activity = Activity::new(ActivityId(id), name, duration);
    activity.constraint = constraint;
    activity.percent_complete = value
        .get("percent_complete")
        .and_then(Value::as_f64)
        .map(|percent| (percent * 100.0).round() as i64)
        .filter(|percent| (0..=10_000).contains(percent));
    activity.budget_at_completion = value.get("budget_at_completion").and_then(money_from_json);
    activity.actual_cost = value.get("actual_cost").and_then(money_from_json);
    activity.actual_start = value.get("actual_start").and_then(date_from_json);
    activity.actual_finish = value.get("actual_finish").and_then(date_from_json);
    activity.owner = match value.get("owner") {
        Some(Value::String(owner)) => Some(owner.clone()),
        _ => None,
    };
    activity.promised = matches!(value.get("promised"), Some(Value::Bool(true)));
    activity.delay_reason = match value.get("delay_reason") {
        Some(Value::String(reason)) => Some(reason.clone()),
        _ => None,
    };
    activity.wbs = value.get("wbs").and_then(Value::as_u64).map(WbsId);
    activity.calendar = value
        .get("calendar")
        .and_then(Value::as_u64)
        .map(CalendarId);
    if let Some(resources) = value.get("resources").and_then(Value::as_array) {
        for resource in resources {
            let resource_id =
                ResourceId(
                    resource
                        .as_u64()
                        .ok_or_else(|| InterchangeError::InvalidField {
                            field: "activity/resources".to_string(),
                            value: "expected a resource id".to_string(),
                        })?,
                );
            if !activity.resources.contains(&resource_id) {
                activity.resources.push(resource_id);
            }
        }
    }
    Ok(activity)
}

fn require_u64(value: &Value, key: &str, context: &str) -> Result<u64, InterchangeError> {
    value.get(key).and_then(Value::as_u64).ok_or_else(|| {
        InterchangeError::MissingField(format!("{context}/{key} must be a whole number"))
    })
}

fn require_activity_id(value: &Value, key: &str) -> Result<ActivityId, InterchangeError> {
    Ok(ActivityId(require_u64(value, key, "relationship")?))
}

fn import_constraint(
    code: Option<&str>,
    date_text: Option<&str>,
) -> Result<Constraint, InterchangeError> {
    let code = code.unwrap_or("ASAP").trim().to_ascii_uppercase();
    let date = date_text.and_then(parse_date);
    match code.as_str() {
        "" | "ASAP" => Ok(Constraint::AsSoonAsPossible),
        "ALAP" => Ok(Constraint::AsLateAsPossible),
        "SNET" => Ok(Constraint::StartNoEarlierThan(require_date_value(date)?)),
        "SNLT" => Ok(Constraint::StartNoLaterThan(require_date_value(date)?)),
        "FNET" => Ok(Constraint::FinishNoEarlierThan(require_date_value(date)?)),
        "FNLT" => Ok(Constraint::FinishNoLaterThan(require_date_value(date)?)),
        other => Err(InterchangeError::InvalidField {
            field: "constraint".to_string(),
            value: other.to_string(),
        }),
    }
}

fn require_date_value(date: Option<Date>) -> Result<Date, InterchangeError> {
    date.ok_or_else(|| {
        InterchangeError::MissingField("constraint_date for a dated constraint".to_string())
    })
}

fn import_link_type(value: Option<&Value>) -> Result<LinkType, InterchangeError> {
    let code = value.and_then(Value::as_str).unwrap_or("FS");
    match code.to_ascii_uppercase().as_str() {
        "FF" => Ok(LinkType::Ff),
        "FS" => Ok(LinkType::Fs),
        "SF" => Ok(LinkType::Sf),
        "SS" => Ok(LinkType::Ss),
        other => Err(InterchangeError::InvalidField {
            field: "type".to_string(),
            value: other.to_string(),
        }),
    }
}

fn import_resource(value: &Value) -> Result<Resource, InterchangeError> {
    let id = ResourceId(require_u64(value, "id", "resource")?);
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if name.is_empty() {
        return Err(InterchangeError::MissingField("resource/name".to_string()));
    }
    let kind = match value
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("labour")
    {
        "material" => ResourceKind::Material,
        "equipment" => ResourceKind::Equipment,
        _ => ResourceKind::Labour,
    };
    let capacity = value
        .get("capacity")
        .and_then(Value::as_u64)
        .map(|capacity| capacity.max(1).min(u64::from(u32::MAX)) as u32)
        .unwrap_or(1);
    Ok(Resource {
        id,
        name,
        kind,
        capacity,
    })
}

fn import_baseline(value: &Value) -> Result<Baseline, InterchangeError> {
    let id = BaselineId(require_u64(value, "id", "baseline")?);
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Baseline")
        .to_string();
    let created_at = value
        .get("created_at")
        .and_then(Value::as_str)
        .and_then(parse_date)
        .unwrap_or_else(default_start_date);

    let mut snapshots = Vec::new();
    if let Some(rows) = value.get("snapshots").and_then(Value::as_array) {
        for row in rows {
            snapshots.push(ActivityBaselineSnapshot {
                activity_id: ActivityId(require_u64(row, "activity_id", "baseline snapshot")?),
                baseline_duration: row
                    .get("duration")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
                    .min(u64::from(u32::MAX)) as u32,
                baseline_start: row
                    .get("start")
                    .and_then(Value::as_str)
                    .and_then(parse_date),
                baseline_finish: row
                    .get("finish")
                    .and_then(Value::as_str)
                    .and_then(parse_date),
            });
        }
    }
    Ok(Baseline::from_snapshots(id, name, created_at, snapshots))
}

fn money_from_json(value: &Value) -> Option<i64> {
    let amount = value.as_f64()?;
    if !amount.is_finite() {
        return None;
    }
    Some((amount * 100.0).round() as i64)
}

fn date_from_json(value: &Value) -> Option<Date> {
    value.as_str().and_then(parse_date)
}

fn activity_json(activity: &Activity) -> Value {
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(activity.id.0 as i64)),
        ),
        ("name".to_string(), Value::String(activity.name.clone())),
        (
            "duration".to_string(),
            Value::Number(Number::Int(i64::from(activity.original_duration))),
        ),
        (
            "constraint".to_string(),
            Value::String(constraint_code(activity.constraint).0.to_string()),
        ),
        (
            "constraint_date".to_string(),
            match constraint_code(activity.constraint).1 {
                Some(date) => Value::String(date.to_string()),
                None => Value::Null,
            },
        ),
        (
            "percent_complete".to_string(),
            match activity.percent_complete {
                Some(percent) => Value::Number(Number::Float(percent as f64 / 100.0)),
                None => Value::Null,
            },
        ),
        (
            "budget_at_completion".to_string(),
            money_json(activity.budget_at_completion),
        ),
        ("actual_cost".to_string(), money_json(activity.actual_cost)),
        ("actual_start".to_string(), date_json(activity.actual_start)),
        (
            "actual_finish".to_string(),
            date_json(activity.actual_finish),
        ),
        (
            "owner".to_string(),
            match &activity.owner {
                Some(owner) => Value::String(owner.clone()),
                None => Value::Null,
            },
        ),
        ("promised".to_string(), Value::Bool(activity.promised)),
        (
            "delay_reason".to_string(),
            match &activity.delay_reason {
                Some(reason) => Value::String(reason.clone()),
                None => Value::Null,
            },
        ),
        (
            "resources".to_string(),
            Value::Array(
                activity
                    .resources
                    .iter()
                    .map(|id| Value::Number(Number::Int(id.0 as i64)))
                    .collect(),
            ),
        ),
        (
            "wbs".to_string(),
            identifier_json(activity.wbs.map(|wbs| wbs.0)),
        ),
        (
            "calendar".to_string(),
            identifier_json(activity.calendar.map(|calendar| calendar.0)),
        ),
    ])
}

/// WBS node ids in preorder: the root first, then each subtree.
fn preorder_nodes(tree: &WbsTree) -> Vec<WbsId> {
    let mut ordered = Vec::with_capacity(tree.nodes().len());
    let mut stack = vec![tree.root()];
    while let Some(id) = stack.pop() {
        let Some(node) = tree.node(id) else {
            continue;
        };
        ordered.push(id);
        for child in node.children.iter().rev() {
            stack.push(*child);
        }
    }
    ordered
}

fn wbs_node_json(node: &cadence_core::wbs::WbsNode) -> Value {
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(node.id.0 as i64)),
        ),
        ("name".to_string(), Value::String(node.name.clone())),
        (
            "parent".to_string(),
            identifier_json(node.parent.map(|parent| parent.0)),
        ),
    ])
}

fn calendar_json(named: &NamedCalendar) -> Value {
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(named.id.0 as i64)),
        ),
        ("name".to_string(), Value::String(named.name.clone())),
        (
            "workweek".to_string(),
            Value::Array(
                named
                    .calendar
                    .working_weekdays()
                    .into_iter()
                    .map(Value::Bool)
                    .collect(),
            ),
        ),
        (
            "holidays".to_string(),
            Value::Array(
                named
                    .calendar
                    .holidays()
                    .iter()
                    .map(|date| Value::String(format_date(*date)))
                    .collect(),
            ),
        ),
    ])
}

fn identifier_json(value: Option<u64>) -> Value {
    match value {
        Some(value) => Value::Number(Number::Int(value as i64)),
        None => Value::Null,
    }
}

fn relationship_json(relationship: &Relationship) -> Value {
    let code = match relationship.link_type {
        LinkType::Ff => "FF",
        LinkType::Fs => "FS",
        LinkType::Sf => "SF",
        LinkType::Ss => "SS",
    };
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(relationship.id.0 as i64)),
        ),
        (
            "predecessor".to_string(),
            Value::Number(Number::Int(relationship.predecessor.0 as i64)),
        ),
        (
            "successor".to_string(),
            Value::Number(Number::Int(relationship.successor.0 as i64)),
        ),
        ("type".to_string(), Value::String(code.to_string())),
        (
            "lag".to_string(),
            Value::Number(Number::Int(relationship.lag)),
        ),
    ])
}

fn resource_json(resource: &Resource) -> Value {
    let kind = match resource.kind {
        ResourceKind::Material => "material",
        ResourceKind::Labour => "labour",
        ResourceKind::Equipment => "equipment",
    };
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(resource.id.0 as i64)),
        ),
        ("name".to_string(), Value::String(resource.name.clone())),
        ("kind".to_string(), Value::String(kind.to_string())),
        (
            "capacity".to_string(),
            Value::Number(Number::Int(i64::from(resource.capacity))),
        ),
    ])
}

fn baseline_json(baseline: &Baseline) -> Value {
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(baseline.id.0 as i64)),
        ),
        ("name".to_string(), Value::String(baseline.name.clone())),
        (
            "created_at".to_string(),
            Value::String(baseline.created_at.to_string()),
        ),
        (
            "snapshots".to_string(),
            Value::Array(
                baseline
                    .snapshots
                    .iter()
                    .map(|snapshot| {
                        Value::Object(vec![
                            (
                                "activity_id".to_string(),
                                Value::Number(Number::Int(snapshot.activity_id.0 as i64)),
                            ),
                            (
                                "duration".to_string(),
                                Value::Number(Number::Int(i64::from(snapshot.baseline_duration))),
                            ),
                            (
                                "start".to_string(),
                                match snapshot.baseline_start {
                                    Some(date) => Value::String(date.to_string()),
                                    None => Value::Null,
                                },
                            ),
                            (
                                "finish".to_string(),
                                match snapshot.baseline_finish {
                                    Some(date) => Value::String(date.to_string()),
                                    None => Value::Null,
                                },
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn constraint_code(constraint: Constraint) -> (&'static str, Option<Date>) {
    match constraint {
        Constraint::AsSoonAsPossible => ("ASAP", None),
        Constraint::AsLateAsPossible => ("ALAP", None),
        Constraint::StartNoEarlierThan(date) => ("SNET", Some(date)),
        Constraint::StartNoLaterThan(date) => ("SNLT", Some(date)),
        Constraint::FinishNoEarlierThan(date) => ("FNET", Some(date)),
        Constraint::FinishNoLaterThan(date) => ("FNLT", Some(date)),
    }
}

fn money_json(minor_units: Option<i64>) -> Value {
    match minor_units {
        Some(units) => Value::Number(Number::Float(units as f64 / 100.0)),
        None => Value::Null,
    }
}

fn date_json(date: Option<Date>) -> Value {
    match date {
        Some(date) => Value::String(format_date(date)),
        None => Value::Null,
    }
}

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

/// A JSON number: an integer when the text had no fraction or exponent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Number {
    Int(i64),
    Float(f64),
}

impl Value {
    /// The first member with the given key, if this is an object.
    #[must_use]
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(members) => members
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// The value as a string slice.
    #[must_use]
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(string) => Some(string),
            _ => None,
        }
    }

    /// The value as a whole number.
    #[must_use]
    pub(crate) fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(Number::Int(int)) if *int >= 0 => Some(*int as u64),
            _ => None,
        }
    }

    /// The value as a signed whole number.
    #[must_use]
    pub(crate) fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(Number::Int(int)) => Some(*int),
            _ => None,
        }
    }

    /// The value as a floating-point number, integers included.
    #[must_use]
    pub(crate) fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(Number::Int(int)) => Some(*int as f64),
            Value::Number(Number::Float(float)) => Some(*float),
            _ => None,
        }
    }

    /// The value as a slice of array items.
    #[must_use]
    pub(crate) fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }
}

fn malformed(message: impl Into<String>) -> InterchangeError {
    InterchangeError::Xml {
        message: message.into(),
    }
}

/// Parse a JSON document.
///
/// # Errors
///
/// Returns [`InterchangeError::Xml`] (the crate's malformed-document variant)
/// when the text is not a single valid JSON value.
pub(crate) fn parse(text: &str) -> Result<Value, InterchangeError> {
    let mut characters = text.chars().peekable();
    let value = parse_value(&mut characters, 0)?;
    skip_whitespace(&mut characters);
    if characters.next().is_some() {
        return Err(malformed("trailing characters after the JSON document"));
    }
    Ok(value)
}

type Characters<'a> = Peekable<Chars<'a>>;

fn parse_value(characters: &mut Characters<'_>, depth: usize) -> Result<Value, InterchangeError> {
    if depth > MAX_DEPTH {
        return Err(malformed("document is nested too deeply"));
    }
    skip_whitespace(characters);
    match characters.peek().copied() {
        Some('{') => parse_object(characters, depth),
        Some('[') => parse_array(characters, depth),
        Some('"') => Ok(Value::String(parse_string(characters)?)),
        Some('t') => literal(characters, "true", Value::Bool(true)),
        Some('f') => literal(characters, "false", Value::Bool(false)),
        Some('n') => literal(characters, "null", Value::Null),
        Some(_) => parse_number(characters),
        None => Err(malformed("unexpected end of document")),
    }
}

fn parse_object(characters: &mut Characters<'_>, depth: usize) -> Result<Value, InterchangeError> {
    characters.next();
    let mut members = Vec::new();
    skip_whitespace(characters);
    if characters.peek() == Some(&'}') {
        characters.next();
        return Ok(Value::Object(members));
    }
    loop {
        skip_whitespace(characters);
        if characters.peek() != Some(&'"') {
            return Err(malformed("expected a quoted member name"));
        }
        let name = parse_string(characters)?;
        skip_whitespace(characters);
        if characters.next() != Some(':') {
            return Err(malformed("expected ':' after a member name"));
        }
        let value = parse_value(characters, depth + 1)?;
        members.push((name, value));
        skip_whitespace(characters);
        match characters.next() {
            Some(',') => {}
            Some('}') => return Ok(Value::Object(members)),
            _ => return Err(malformed("expected ',' or '}' in an object")),
        }
    }
}

fn parse_array(characters: &mut Characters<'_>, depth: usize) -> Result<Value, InterchangeError> {
    characters.next();
    let mut items = Vec::new();
    skip_whitespace(characters);
    if characters.peek() == Some(&']') {
        characters.next();
        return Ok(Value::Array(items));
    }
    loop {
        let value = parse_value(characters, depth + 1)?;
        items.push(value);
        skip_whitespace(characters);
        match characters.next() {
            Some(',') => {}
            Some(']') => return Ok(Value::Array(items)),
            _ => return Err(malformed("expected ',' or ']' in an array")),
        }
    }
}

fn parse_string(characters: &mut Characters<'_>) -> Result<String, InterchangeError> {
    characters.next();
    let mut string = String::new();
    loop {
        let Some(character) = characters.next() else {
            return Err(malformed("unterminated string"));
        };
        match character {
            '"' => return Ok(string),
            '\\' => match characters.next() {
                Some('"') => string.push('"'),
                Some('\\') => string.push('\\'),
                Some('/') => string.push('/'),
                Some('b') => string.push('\u{8}'),
                Some('f') => string.push('\u{c}'),
                Some('n') => string.push('\n'),
                Some('r') => string.push('\r'),
                Some('t') => string.push('\t'),
                Some('u') => string.push(parse_unicode_escape(characters)?),
                _ => return Err(malformed("invalid escape sequence in a string")),
            },
            character if (character as u32) < 0x20 => {
                return Err(malformed("unescaped control character in a string"));
            }
            character => string.push(character),
        }
    }
}

fn parse_unicode_escape(characters: &mut Characters<'_>) -> Result<char, InterchangeError> {
    let high = hex_quartet(characters)?;
    let code = match high {
        0xDC00..=0xDFFF => return Err(malformed("unexpected low surrogate")),
        0xD800..=0xDBFF => {
            if characters.next() != Some('\\') || characters.next() != Some('u') {
                return Err(malformed("unpaired surrogate in a string"));
            }
            let low = hex_quartet(characters)?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err(malformed("invalid low surrogate"));
            }
            0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)
        }
        code => code,
    };
    char::from_u32(code).ok_or_else(|| malformed("invalid Unicode escape"))
}

fn hex_quartet(characters: &mut Characters<'_>) -> Result<u32, InterchangeError> {
    let mut value = 0u32;
    for _ in 0..4 {
        let digit = characters
            .next()
            .and_then(|character| character.to_digit(16))
            .ok_or_else(|| malformed("invalid \\u escape"))?;
        value = value * 16 + digit;
    }
    Ok(value)
}

fn parse_number(characters: &mut Characters<'_>) -> Result<Value, InterchangeError> {
    let mut token = String::new();
    while let Some(&character) = characters.peek() {
        if character.is_ascii_digit() || matches!(character, '-' | '+' | '.' | 'e' | 'E') {
            token.push(character);
            characters.next();
        } else {
            break;
        }
    }
    if token.is_empty() || token == "-" {
        return Err(malformed("expected a value"));
    }
    if token.contains(['.', 'e', 'E']) {
        let value: f64 = token
            .parse()
            .map_err(|_| malformed(format!("invalid number \"{token}\"")))?;
        if !value.is_finite() {
            return Err(malformed(format!("number \"{token}\" is out of range")));
        }
        return Ok(Value::Number(Number::Float(value)));
    }
    let value: i64 = token
        .parse()
        .map_err(|_| malformed(format!("number \"{token}\" is out of range")))?;
    Ok(Value::Number(Number::Int(value)))
}

fn literal(
    characters: &mut Characters<'_>,
    text: &str,
    value: Value,
) -> Result<Value, InterchangeError> {
    for expected in text.chars() {
        if characters.next() != Some(expected) {
            return Err(malformed("invalid literal"));
        }
    }
    Ok(value)
}

fn skip_whitespace(characters: &mut Characters<'_>) {
    while matches!(
        characters.peek(),
        Some(' ') | Some('\t') | Some('\n') | Some('\r')
    ) {
        characters.next();
    }
}

/// Serialize a value with two-space indentation.
pub(crate) fn write_value(value: &Value, indent: usize, output: &mut String) {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(true) => output.push_str("true"),
        Value::Bool(false) => output.push_str("false"),
        Value::Number(Number::Int(int)) => output.push_str(&int.to_string()),
        Value::Number(Number::Float(float)) => output.push_str(&format_number(*float)),
        Value::String(string) => write_string(string, output),
        Value::Array(items) => {
            if items.is_empty() {
                output.push_str("[]");
                return;
            }
            output.push_str("[\n");
            for (index, item) in items.iter().enumerate() {
                push_indent(output, indent + 1);
                write_value(item, indent + 1, output);
                if index + 1 < items.len() {
                    output.push(',');
                }
                output.push('\n');
            }
            push_indent(output, indent);
            output.push(']');
        }
        Value::Object(members) => {
            if members.is_empty() {
                output.push_str("{}");
                return;
            }
            output.push_str("{\n");
            for (index, (name, member)) in members.iter().enumerate() {
                push_indent(output, indent + 1);
                write_string(name, output);
                output.push_str(": ");
                write_value(member, indent + 1, output);
                if index + 1 < members.len() {
                    output.push(',');
                }
                output.push('\n');
            }
            push_indent(output, indent);
            output.push('}');
        }
    }
}

fn push_indent(output: &mut String, indent: usize) {
    for _ in 0..indent {
        output.push_str("  ");
    }
}

fn format_number(float: f64) -> String {
    if !float.is_finite() {
        return "null".to_string();
    }
    format!("{float}")
}

fn write_string(string: &str, output: &mut String) {
    output.push('"');
    for character in string.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{8}' => output.push_str("\\b"),
            '\u{c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid date")
    }

    /// A tree compared by structure rather than by insertion order: `WbsTree`
    /// keeps nodes in the order they were added, and import adds them in
    /// preorder.
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

    fn tree_with_branches() -> WbsTree {
        let mut tree = WbsTree::new();
        tree.rename(tree.root(), "Bridge & Road")
            .expect("rename root");
        tree.add(WbsId(2), "Design", tree.root())
            .expect("first child");
        tree.add(WbsId(3), "Construction", tree.root())
            .expect("second child");
        tree.add(WbsId(4), "Foundations", WbsId(3))
            .expect("grandchild");
        tree
    }

    fn sample() -> Project {
        let start = date(2026, Month::January, 5);
        let mut project = Project::new("Bridge & Road", start);

        let mut survey = Activity::new(ActivityId(1), "Survey", 2);
        survey.percent_complete = Some(5000);
        survey.budget_at_completion = Some(150_000);
        survey.actual_cost = Some(75_050);
        survey.actual_start = Some(start);
        survey.actual_finish = Some(date(2026, Month::January, 6));
        project.activities.push(survey);

        let mut excavate = Activity::new(ActivityId(2), "Excavate", 3);
        excavate.constraint = Constraint::StartNoEarlierThan(date(2026, Month::January, 8));
        project.activities.push(excavate);

        project.relationships.push(Relationship {
            id: RelationshipId(1),
            predecessor: ActivityId(1),
            successor: ActivityId(2),
            link_type: LinkType::Ss,
            lag: -1,
        });

        project
            .resources
            .push(Resource::new(ResourceId(1), "Crew A"));
        let mut crane = Resource::with_capacity(ResourceId(2), "Crane", 2);
        crane.kind = ResourceKind::Equipment;
        project.resources.push(crane);

        project.activities[0].resources = vec![ResourceId(1)];
        project.activities[1].resources = vec![ResourceId(1), ResourceId(2)];

        project.baselines.push(Baseline {
            id: BaselineId(1),
            name: "Baseline 1".to_string(),
            created_at: start,
            snapshots: vec![
                ActivityBaselineSnapshot {
                    activity_id: ActivityId(1),
                    baseline_duration: 2,
                    baseline_start: Some(start),
                    baseline_finish: Some(date(2026, Month::January, 6)),
                },
                ActivityBaselineSnapshot {
                    activity_id: ActivityId(2),
                    baseline_duration: 3,
                    baseline_start: Some(date(2026, Month::January, 7)),
                    baseline_finish: Some(date(2026, Month::January, 9)),
                },
            ],
        });

        project
    }

    #[test]
    fn exporting_and_importing_preserves_the_project() {
        let project = sample();
        let text = export(&project);
        let loaded = import(&text).expect("import");

        assert_eq!(loaded.name, project.name);
        assert_eq!(loaded.start_date, project.start_date);
        assert_eq!(loaded.activities, project.activities);
        assert_eq!(loaded.relationships, project.relationships);
        assert_eq!(loaded.resources, project.resources);
        assert_eq!(loaded.baselines.len(), 1);
        assert_eq!(
            loaded.baselines[0].snapshots,
            project.baselines[0].snapshots
        );
    }

    #[test]
    fn a_minimal_hand_written_document_is_read_with_defaults() {
        let text = r#"{
            "cadence": 1,
            "project": {
                "name": "Handover",
                "start_date": "2026-02-02",
                "activities": [
                    {"id": 10, "name": "Design", "duration": 3},
                    {"id": 11, "name": "Build", "duration": 0}
                ]
            }
        }"#;
        let project = import(text).expect("import");

        assert_eq!(project.name, "Handover");
        assert_eq!(project.activities.len(), 2);
        assert_eq!(
            project.activities[0].constraint,
            Constraint::AsSoonAsPossible
        );
        assert_eq!(project.activities[0].percent_complete, None);
        assert_eq!(project.activities[0].resources, Vec::<ResourceId>::new());
        assert_eq!(project.activities[1].original_duration, 0);
        assert!(project.relationships.is_empty());
        assert!(project.resources.is_empty());
    }

    #[test]
    fn the_work_breakdown_and_calendars_round_trip() {
        let mut project = sample();
        project.wbs = tree_with_branches();

        let six_day = Calendar::new([true, true, true, true, true, true, false])
            .expect("six-day week")
            .with_holiday(date(2026, Month::January, 1))
            .with_holiday(date(2026, Month::December, 25));
        project
            .calendars
            .add(CalendarId(1), "Six day", six_day)
            .expect("calendar");

        project.activities[0].wbs = Some(WbsId(2));
        project.activities[1].wbs = Some(WbsId(4));
        project.activities[1].calendar = Some(CalendarId(1));

        let text = export(&project);
        let loaded = import(&text).expect("import");

        assert_eq!(wbs_shape(&loaded.wbs), wbs_shape(&project.wbs));
        assert_eq!(
            loaded.wbs.node(loaded.wbs.root()).expect("root").name,
            "Bridge & Road"
        );
        assert_eq!(loaded.calendars.calendars(), project.calendars.calendars());
        assert_eq!(loaded.activities, project.activities);
    }

    #[test]
    fn the_exported_wbs_is_written_in_preorder() {
        let mut project = sample();
        project.wbs = tree_with_branches();

        let text = export(&project);
        let design = text.find("\"name\": \"Design\"").expect("design");
        let construction = text
            .find("\"name\": \"Construction\"")
            .expect("construction");
        let foundations = text.find("\"name\": \"Foundations\"").expect("foundations");
        // A parent always precedes its children, and the root comes first.
        let root = text.find("\"name\": \"Bridge & Road\"").expect("root");
        assert!(root < design && design < construction && construction < foundations);
    }

    #[test]
    fn a_version_one_document_still_loads_without_wbs_or_calendars() {
        let text = r#"{
            "cadence": 1,
            "project": {
                "name": "Old",
                "start_date": "2026-02-02",
                "activities": [{"id": 1, "name": "A", "duration": 1}]
            }
        }"#;
        let project = import(text).expect("import version 1");

        assert_eq!(project.activities.len(), 1);
        assert_eq!(project.activities[0].wbs, None);
        assert_eq!(project.activities[0].calendar, None);
        // Only the root exists, and there are no named calendars.
        assert_eq!(project.wbs.nodes().len(), 1);
        assert!(project.calendars.is_empty());
    }

    #[test]
    fn dangling_membership_is_rejected() {
        let text = r#"{
            "cadence": 2,
            "project": {
                "start_date": "2026-01-05",
                "activities": [{"id": 1, "name": "A", "duration": 1, "calendar": 7}]
            }
        }"#;
        let error = import(text).expect_err("missing calendar");
        assert!(matches!(error, InterchangeError::InvalidField { .. }));

        let text = r#"{
            "cadence": 2,
            "project": {
                "start_date": "2026-01-05",
                "activities": [{"id": 1, "name": "A", "duration": 1, "wbs": 9}]
            }
        }"#;
        let error = import(text).expect_err("missing WBS node");
        assert!(matches!(error, InterchangeError::InvalidField { .. }));
    }

    #[test]
    fn a_malformed_work_breakdown_is_rejected() {
        // A second root.
        let text = r#"{
            "cadence": 2,
            "project": {
                "start_date": "2026-01-05",
                "wbs": [
                    {"id": 1, "name": "Root", "parent": null},
                    {"id": 2, "name": "Another", "parent": null}
                ]
            }
        }"#;
        assert!(import(text).is_err());

        // A child before its parent.
        let text = r#"{
            "cadence": 2,
            "project": {
                "start_date": "2026-01-05",
                "wbs": [{"id": 2, "name": "Child", "parent": 1}]
            }
        }"#;
        assert!(import(text).is_err());

        // A weekday pattern with no working days.
        let text = r#"{
            "cadence": 2,
            "project": {
                "start_date": "2026-01-05",
                "calendars": [
                    {
                        "id": 1, "name": "Never",
                        "workweek": [false, false, false, false, false, false, false]
                    }
                ]
            }
        }"#;
        assert!(import(text).is_err());

        // A workweek that is not seven booleans.
        let text = r#"{
            "cadence": 2,
            "project": {
                "start_date": "2026-01-05",
                "calendars": [{"id": 1, "name": "Short", "workweek": [true, false]}]
            }
        }"#;
        assert!(import(text).is_err());
    }

    #[test]
    fn an_unknown_schema_version_is_rejected() {
        let text = r#"{"cadence": 99, "project": {}}"#;
        let error = import(text).expect_err("version");
        assert!(matches!(error, InterchangeError::Unsupported(_)));
    }

    #[test]
    fn a_missing_start_date_is_rejected() {
        let text = r#"{"cadence": 1, "project": {"name": "x"}}"#;
        let error = import(text).expect_err("no start");
        assert_eq!(
            error,
            InterchangeError::MissingField("project/start_date".to_string())
        );
    }

    #[test]
    fn a_dated_constraint_without_a_date_is_rejected() {
        let text = r#"{
            "cadence": 1,
            "project": {
                "start_date": "2026-01-05",
                "activities": [{"id": 1, "name": "A", "duration": 2, "constraint": "SNET"}]
            }
        }"#;
        let error = import(text).expect_err("no constraint date");
        assert_eq!(
            error,
            InterchangeError::MissingField("constraint_date for a dated constraint".to_string())
        );
    }

    #[test]
    fn a_resource_without_a_name_is_rejected() {
        let text = r#"{
            "cadence": 1,
            "project": {
                "start_date": "2026-01-05",
                "resources": [{"id": 1}]
            }
        }"#;
        let error = import(text).expect_err("no resource name");
        assert_eq!(
            error,
            InterchangeError::MissingField("resource/name".to_string())
        );
    }

    #[test]
    fn json_and_mspdi_agree_on_the_same_project() {
        let project = sample();

        let from_json = import(&export(&project));
        let from_xml = crate::mspdi::import(&crate::mspdi::export(&project).expect("xml"))
            .expect("xml import");
        let from_json = from_json.expect("json import");

        assert_eq!(from_json.activities, from_xml.activities);
        assert_eq!(from_json.start_date, from_xml.start_date);
        assert_eq!(from_json.resources, from_xml.resources);
    }

    #[test]
    fn the_reader_accepts_escapes_and_rejects_malformed_documents() {
        let value =
            parse(r#""caf\u00e9 \uD83D\uDE00 tab\tline\nquote\"back\\""#).expect("escape handling");
        assert_eq!(
            value.as_str(),
            Some("café \u{1F600} tab\tline\nquote\"back\\")
        );

        assert!(parse("").is_err());
        assert!(parse("{").is_err());
        assert!(parse("{} trailing").is_err());
        assert!(parse("{\"a\": }").is_err());
        assert!(parse("{\"a\" 1}").is_err());
        assert!(parse("[1, 2,]").is_err());
        assert!(parse("\"unterminated").is_err());
        assert!(parse("tru").is_err());
        assert!(parse("-").is_err());
        assert!(parse(r#""bad \q escape""#).is_err());
        assert!(parse(r#""lone surrogate \ud800""#).is_err());
        assert!(parse(r#""bad hex \u00zz""#).is_err());
    }

    #[test]
    fn numbers_are_read_as_integers_or_floats() {
        assert_eq!(parse("42").expect("int"), Value::Number(Number::Int(42)));
        assert_eq!(
            parse("-7").expect("negative"),
            Value::Number(Number::Int(-7))
        );
        assert_eq!(
            parse("1500.00").expect("float"),
            Value::Number(Number::Float(1500.0))
        );
        assert_eq!(
            parse("2e2").expect("exponent"),
            Value::Number(Number::Float(200.0))
        );
        assert!(parse("99999999999999999999999").is_err());
    }

    #[test]
    fn deeply_nested_documents_are_rejected() {
        let text = format!("{}{}", "[".repeat(80), "]".repeat(80));
        let error = parse(&text).expect_err("depth");
        assert!(matches!(error, InterchangeError::Xml { .. }));
    }

    #[test]
    fn strings_with_special_characters_survive_a_round_trip() {
        let project = Project::new(
            "A \"quoted\" \\ name, with\nnewline\tand\ttab",
            date(2026, Month::January, 5),
        );
        let text = export(&project);
        assert!(text.contains("A \\\"quoted\\\" \\\\ name, with\\nnewline\\tand\\ttab"));

        let loaded = import(&text).expect("import");
        assert_eq!(loaded.name, project.name);
    }

    #[test]
    fn the_writer_produces_indented_output() {
        let project = Project::new("Demo", date(2026, Month::January, 5));
        let text = export(&project);
        assert!(text.starts_with("{\n  \"cadence\": 2,\n  \"project\": {"));
        assert!(text.contains("\"activities\": []"));
        assert!(text.contains("\"resources\": []"));
        assert!(text.contains("\"calendars\": []"));
        // Even an untouched project has its WBS root.
        assert!(text.contains("\"wbs\": ["));
    }
}
