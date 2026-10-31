//! Machine-readable schedule output.
//!
//! The plain-text schedule that the CLI prints is good for a person at the
//! terminal and awkward for everyone else: a script has to parse a table
//! whose column positions can change, and a screen reader gets a wall of
//! padding. This module writes the same schedule as one small JSON document
//! instead (guide section 21.5, accessibility of tooling). The reader and
//! writer are the JSON machinery in [`crate::json`]: no second parser is
//! added to the build, and the document is written by the same functions the
//! interchange format uses (D14).
//!
//! This is deliberately the *one* document where derived dates are written.
//! The interchange document ([`crate::json::export`]) never carries
//! scheduled, early or late dates because they are derived data (D9) and the
//! engine recomputes them from logic, calendars and constraints; here the
//! engine has already run and the whole point of the document is to carry
//! its results. Nothing is ever read back in: recompute the schedule from
//! the project rather than trusting an exported one.
//!
//! The schedule document looks like this (`cadence` is this document's own
//! schema version, and `kind` distinguishes a schedule from an error):
//!
//! ```json
//! {
//!   "cadence": 1,
//!   "kind": "schedule",
//!   "project": {
//!     "name": "Bridge & Road",
//!     "start": "2026-01-05",
//!     "finish": "2026-01-12"
//!   },
//!   "activities": [
//!     {
//!       "id": 1, "name": "Survey", "duration_days": 2,
//!       "scheduled_start": "2026-01-05", "scheduled_finish": "2026-01-06",
//!       "early_start": "2026-01-05", "early_finish": "2026-01-06",
//!       "late_start": "2026-01-07", "late_finish": "2026-01-08",
//!       "total_float_days": 1, "free_float_days": 1,
//!       "critical": false
//!     }
//!   ],
//!   "critical_path": [2]
//! }
//! ```
//!
//! `activities` follows the project's activity order and skips any activity
//! the engine did not schedule — the same filter the plain-text report
//! applies. Durations and floats are working days, so a `duration_days` of
//! zero denotes a milestone.

use cadence_core::engine::{ActivitySchedule, Schedule};
use cadence_core::model::{Activity, Project};

use crate::json::{write_value, Number, Value};

/// The schema version of the schedule and error documents this build writes.
const SCHEDULE_SCHEMA_VERSION: i64 = 1;

/// Write the schedule of a project as a JSON document.
///
/// The document is pretty-printed exactly as [`crate::json::export`] prints
/// the interchange document, with one trailing newline, so both can be
/// consumed the same way.
#[must_use]
pub fn schedule_export(project: &Project, schedule: &Schedule) -> String {
    let document = Value::Object(vec![
        (
            "cadence".to_string(),
            Value::Number(Number::Int(SCHEDULE_SCHEMA_VERSION)),
        ),
        ("kind".to_string(), Value::String("schedule".to_string())),
        (
            "project".to_string(),
            Value::Object(vec![
                ("name".to_string(), Value::String(project.name.clone())),
                (
                    "start".to_string(),
                    Value::String(schedule.project_start.to_string()),
                ),
                (
                    "finish".to_string(),
                    Value::String(schedule.project_finish.to_string()),
                ),
            ]),
        ),
        (
            "activities".to_string(),
            Value::Array(
                project
                    .activities
                    .iter()
                    .filter_map(|activity| {
                        let result = schedule.activity(activity.id)?;
                        Some(activity_json(activity, result))
                    })
                    .collect(),
            ),
        ),
        (
            "critical_path".to_string(),
            Value::Array(
                schedule
                    .critical_path
                    .iter()
                    .map(|id| Value::Number(Number::Int(id.0 as i64)))
                    .collect(),
            ),
        ),
    ]);

    to_text(&document)
}

/// Write a document that reports why the schedule could not be produced.
///
/// This is the error half of the schema: a consumer that reads only
/// `cadence` and `kind` can tell an error document from a schedule without
/// reading further, and the machine-facing message is the same text the CLI
/// writes to stderr for the person watching the terminal.
#[must_use]
pub fn schedule_error(message: &str) -> String {
    let document = Value::Object(vec![
        (
            "cadence".to_string(),
            Value::Number(Number::Int(SCHEDULE_SCHEMA_VERSION)),
        ),
        ("kind".to_string(), Value::String("error".to_string())),
        (
            "error".to_string(),
            Value::Object(vec![(
                "message".to_string(),
                Value::String(message.to_string()),
            )]),
        ),
    ]);

    to_text(&document)
}

/// Serialize a document exactly as `json::export` serializes the interchange
/// document: pretty-printed from the top level with one trailing newline.
pub(crate) fn to_text(document: &Value) -> String {
    let mut output = String::new();
    write_value(document, 0, &mut output);
    output.push('\n');
    output
}

/// One entry of `activities`: the activity as planned plus its scheduled
/// result as derived.
fn activity_json(activity: &Activity, result: &ActivitySchedule) -> Value {
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(activity.id.0 as i64)),
        ),
        ("name".to_string(), Value::String(activity.name.clone())),
        (
            "duration_days".to_string(),
            Value::Number(Number::Int(i64::from(activity.original_duration))),
        ),
        (
            "scheduled_start".to_string(),
            Value::String(result.scheduled_start.to_string()),
        ),
        (
            "scheduled_finish".to_string(),
            Value::String(result.scheduled_finish.to_string()),
        ),
        (
            "early_start".to_string(),
            Value::String(result.early_start.to_string()),
        ),
        (
            "early_finish".to_string(),
            Value::String(result.early_finish.to_string()),
        ),
        (
            "late_start".to_string(),
            Value::String(result.late_start.to_string()),
        ),
        (
            "late_finish".to_string(),
            Value::String(result.late_finish.to_string()),
        ),
        (
            "total_float_days".to_string(),
            Value::Number(Number::Int(result.total_float)),
        ),
        (
            "free_float_days".to_string(),
            Value::Number(Number::Int(result.free_float)),
        ),
        ("critical".to_string(), Value::Bool(result.critical)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse;
    use cadence_core::model::{ActivityId, LinkType, Relationship, RelationshipId};
    use time::{Date, Month};

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid date")
    }

    /// A survey followed by excavation, the slice the engine tests use.
    fn sample() -> Project {
        let mut project = Project::new("Bridge & Road", date(2026, Month::January, 5));
        project
            .activities
            .push(Activity::new(ActivityId(1), "Survey", 2));
        project
            .activities
            .push(Activity::new(ActivityId(2), "Excavate", 3));
        project.relationships.push(Relationship::new(
            RelationshipId(1),
            ActivityId(1),
            ActivityId(2),
            LinkType::Fs,
            0,
        ));
        project
    }

    /// The integer of an object member, read back through the parser.
    fn int(value: &Value, key: &str) -> Option<i64> {
        value.get(key).and_then(Value::as_i64)
    }

    /// The string of an object member, read back through the parser.
    fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
        value.get(key).and_then(Value::as_str)
    }

    /// Every item of an array, read back as whole numbers through the parser.
    fn ints(value: &Value) -> Vec<i64> {
        value
            .as_array()
            .expect("array member")
            .iter()
            .map(|item| item.as_i64().expect("whole number"))
            .collect()
    }

    #[test]
    fn a_two_activity_project_exports_the_full_schedule() {
        let project = sample();
        let schedule = cadence_core::engine::schedule(&project).expect("schedule succeeds");
        let document = parse(&schedule_export(&project, &schedule)).expect("document parses");

        assert_eq!(int(&document, "cadence"), Some(1));
        assert_eq!(text(&document, "kind"), Some("schedule"));

        let project_json = document.get("project").expect("project member");
        assert_eq!(text(project_json, "name"), Some("Bridge & Road"));
        assert_eq!(text(project_json, "start"), Some("2026-01-05"));
        assert_eq!(text(project_json, "finish"), Some("2026-01-09"));

        let activities = document
            .get("activities")
            .and_then(Value::as_array)
            .expect("activities array");
        assert_eq!(activities.len(), 2);

        let survey = &activities[0];
        assert_eq!(int(survey, "id"), Some(1));
        assert_eq!(text(survey, "name"), Some("Survey"));
        assert_eq!(int(survey, "duration_days"), Some(2));
        assert_eq!(text(survey, "scheduled_start"), Some("2026-01-05"));
        assert_eq!(text(survey, "scheduled_finish"), Some("2026-01-06"));
        assert_eq!(text(survey, "early_start"), Some("2026-01-05"));
        assert_eq!(text(survey, "early_finish"), Some("2026-01-06"));
        assert_eq!(text(survey, "late_start"), Some("2026-01-05"));
        assert_eq!(text(survey, "late_finish"), Some("2026-01-06"));
        assert_eq!(int(survey, "total_float_days"), Some(0));
        assert_eq!(int(survey, "free_float_days"), Some(0));
        assert_eq!(survey.get("critical"), Some(&Value::Bool(true)));

        let excavate = &activities[1];
        assert_eq!(int(excavate, "id"), Some(2));
        assert_eq!(text(excavate, "scheduled_start"), Some("2026-01-07"));
        assert_eq!(text(excavate, "scheduled_finish"), Some("2026-01-09"));
        assert_eq!(excavate.get("critical"), Some(&Value::Bool(true)));

        let path = document.get("critical_path").expect("critical_path member");
        assert_eq!(ints(path), vec![1, 2]);
    }

    #[test]
    fn a_milestone_starts_and_finishes_on_the_same_day() {
        let mut project = Project::new("Notice served", date(2026, Month::January, 5));
        project
            .activities
            .push(Activity::new(ActivityId(1), "Go-ahead", 0));

        let schedule = cadence_core::engine::schedule(&project).expect("schedule succeeds");
        let document = parse(&schedule_export(&project, &schedule)).expect("document parses");
        let activity = &document
            .get("activities")
            .and_then(Value::as_array)
            .expect("activities array")[0];

        assert_eq!(int(activity, "duration_days"), Some(0));
        assert_eq!(
            text(activity, "scheduled_start"),
            text(activity, "scheduled_finish")
        );
    }

    #[test]
    fn names_with_quotes_and_newlines_survive_a_round_trip() {
        let mut project = Project::new("Awkward names", date(2026, Month::January, 5));
        project.activities.push(Activity::new(
            ActivityId(1),
            "Pile \"install\"\nsecond line",
            1,
        ));

        let schedule = cadence_core::engine::schedule(&project).expect("schedule succeeds");
        let text_document = schedule_export(&project, &schedule);
        assert!(text_document.contains("Pile \\\"install\\\"\\nsecond line"));

        let document = parse(&text_document).expect("document parses");
        let activity = &document
            .get("activities")
            .and_then(Value::as_array)
            .expect("activities array")[0];
        assert_eq!(
            text(activity, "name"),
            Some("Pile \"install\"\nsecond line")
        );
    }

    #[test]
    fn an_empty_project_writes_an_empty_activity_list() {
        let project = Project::new("Empty", date(2026, Month::January, 5));
        let schedule = cadence_core::engine::schedule(&project).expect("schedule succeeds");
        let document = parse(&schedule_export(&project, &schedule)).expect("document parses");

        let activities = document
            .get("activities")
            .and_then(Value::as_array)
            .expect("activities array");
        assert!(activities.is_empty());

        let project_json = document.get("project").expect("project member");
        assert_eq!(text(project_json, "start"), Some("2026-01-05"));
        assert_eq!(text(project_json, "finish"), Some("2026-01-05"));
        assert!(document
            .get("critical_path")
            .and_then(Value::as_array)
            .expect("critical_path array")
            .is_empty());
    }

    #[test]
    fn a_cyclic_project_writes_an_error_document() {
        let mut project = Project::new("Cycle", date(2026, Month::January, 5));
        project
            .activities
            .push(Activity::new(ActivityId(1), "A", 1));
        project
            .activities
            .push(Activity::new(ActivityId(2), "B", 1));
        project.relationships.push(Relationship::new(
            RelationshipId(1),
            ActivityId(1),
            ActivityId(2),
            LinkType::Fs,
            0,
        ));
        project.relationships.push(Relationship::new(
            RelationshipId(2),
            ActivityId(2),
            ActivityId(1),
            LinkType::Fs,
            0,
        ));

        let error = cadence_core::engine::schedule(&project).expect_err("cycle is rejected");
        let document = parse(&schedule_error(&error.to_string())).expect("document parses");

        assert_eq!(int(&document, "cadence"), Some(1));
        assert_eq!(text(&document, "kind"), Some("error"));
        let error_json = document.get("error").expect("error member");
        assert!(!text(error_json, "message").unwrap_or_default().is_empty());
    }

    #[test]
    fn exactly_one_activity_is_critical() {
        // No links: the longer activity alone fixes the project finish, and
        // the shorter one floats against it.
        let mut project = Project::new("Two lengths", date(2026, Month::January, 5));
        project
            .activities
            .push(Activity::new(ActivityId(1), "Build", 5));
        project
            .activities
            .push(Activity::new(ActivityId(2), "Inspect", 2));

        let schedule = cadence_core::engine::schedule(&project).expect("schedule succeeds");
        let document = parse(&schedule_export(&project, &schedule)).expect("document parses");

        let path = document.get("critical_path").expect("critical_path member");
        assert_eq!(ints(path), vec![1]);

        let activities = document
            .get("activities")
            .and_then(Value::as_array)
            .expect("activities array");
        assert_eq!(activities[0].get("critical"), Some(&Value::Bool(true)));
        assert_eq!(activities[1].get("critical"), Some(&Value::Bool(false)));
    }
}
