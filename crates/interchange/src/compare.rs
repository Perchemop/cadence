//! Machine-readable comparison output.
//!
//! [`crate::schedule`] writes the derived schedule and [`crate::audit`] writes
//! the derived audit; this module writes the derived **comparison** of two
//! programmes (or of a baseline against the current programme): the
//! added/removed activities, duration, date, constraint and relationship
//! changes that the forensic stage of the programme audit reports. Like the
//! other derived documents it is written by the in-crate JSON machinery and
//! never read back.
//!
//! The document looks like this:
//!
//! ```json
//! {
//!   "cadence": 1,
//!   "kind": "comparison",
//!   "added_activities": [{ "id": 3, "name": "Pour" }],
//!   "removed_activities": [],
//!   "duration_changes": [
//!     { "id": 2, "name": "Excavate", "before": 3, "after": 6 }
//!   ],
//!   "date_changes": [
//!     {
//!       "id": 2, "name": "Excavate",
//!       "start": { "before": "2026-01-06", "after": "2026-01-06", "days": 0 },
//!       "finish": { "before": "2026-01-08", "after": "2026-01-13", "days": 3 }
//!     }
//!   ],
//!   "constraint_changes": [],
//!   "relationship_changes": [],
//!   "project_dates": {
//!     "start": { "before": "2026-01-05", "after": "2026-01-05", "days": 0 },
//!     "finish": { "before": "2026-01-08", "after": "2026-01-13", "days": 3 }
//!   }
//! }
//! ```

use cadence_core::compare::{Comparison, DateShift, RelationshipChange};
use cadence_core::model::Project;

use crate::json::{Number, Value};
use crate::schedule::to_text;

/// The schema version of the comparison document this build writes.
const COMPARISON_SCHEMA_VERSION: i64 = 1;

/// Write the comparison of two programmes as a JSON document.
#[must_use]
pub fn comparison_export(project: &Project, comparison: &Comparison) -> String {
    let document = Value::Object(vec![
        (
            "cadence".to_string(),
            Value::Number(Number::Int(COMPARISON_SCHEMA_VERSION)),
        ),
        ("kind".to_string(), Value::String("comparison".to_string())),
        (
            "project".to_string(),
            Value::Object(vec![(
                "name".to_string(),
                Value::String(project.name.clone()),
            )]),
        ),
        (
            "added_activities".to_string(),
            Value::Array(
                comparison
                    .added_activities
                    .iter()
                    .map(activity_ref_json)
                    .collect(),
            ),
        ),
        (
            "removed_activities".to_string(),
            Value::Array(
                comparison
                    .removed_activities
                    .iter()
                    .map(activity_ref_json)
                    .collect(),
            ),
        ),
        (
            "duration_changes".to_string(),
            Value::Array(
                comparison
                    .duration_changes
                    .iter()
                    .map(|change| {
                        Value::Object(vec![
                            (
                                "id".to_string(),
                                Value::Number(Number::Int(change.id.0 as i64)),
                            ),
                            ("name".to_string(), Value::String(change.name.clone())),
                            (
                                "before".to_string(),
                                Value::Number(Number::Int(i64::from(change.before))),
                            ),
                            (
                                "after".to_string(),
                                Value::Number(Number::Int(i64::from(change.after))),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "date_changes".to_string(),
            Value::Array(
                comparison
                    .date_changes
                    .iter()
                    .map(|change| {
                        Value::Object(vec![
                            (
                                "id".to_string(),
                                Value::Number(Number::Int(change.id.0 as i64)),
                            ),
                            ("name".to_string(), Value::String(change.name.clone())),
                            ("start".to_string(), shift_json(&change.start)),
                            ("finish".to_string(), shift_json(&change.finish)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "constraint_changes".to_string(),
            Value::Array(
                comparison
                    .constraint_changes
                    .iter()
                    .map(|change| {
                        Value::Object(vec![
                            (
                                "id".to_string(),
                                Value::Number(Number::Int(change.id.0 as i64)),
                            ),
                            ("name".to_string(), Value::String(change.name.clone())),
                            ("before".to_string(), Value::String(change.before.clone())),
                            ("after".to_string(), Value::String(change.after.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "relationship_changes".to_string(),
            Value::Array(
                comparison
                    .relationship_changes
                    .iter()
                    .map(relationship_change_json)
                    .collect(),
            ),
        ),
        (
            "project_dates".to_string(),
            match &comparison.project_dates {
                Some(dates) => Value::Object(vec![
                    ("start".to_string(), shift_json(&dates.start)),
                    ("finish".to_string(), shift_json(&dates.finish)),
                ]),
                None => Value::Null,
            },
        ),
    ]);

    to_text(&document)
}

fn activity_ref_json(reference: &cadence_core::compare::ActivityRef) -> Value {
    Value::Object(vec![
        (
            "id".to_string(),
            Value::Number(Number::Int(reference.id.0 as i64)),
        ),
        ("name".to_string(), Value::String(reference.name.clone())),
    ])
}

fn shift_json(shift: &DateShift) -> Value {
    Value::Object(vec![
        (
            "before".to_string(),
            match shift.before {
                Some(date) => Value::String(date.to_string()),
                None => Value::Null,
            },
        ),
        (
            "after".to_string(),
            match shift.after {
                Some(date) => Value::String(date.to_string()),
                None => Value::Null,
            },
        ),
        ("days".to_string(), Value::Number(Number::Int(shift.days))),
    ])
}

fn relationship_change_json(change: &RelationshipChange) -> Value {
    let (kind, relationship) = match change {
        RelationshipChange::Added { relationship } => ("added", relationship),
        RelationshipChange::Removed { relationship } => ("removed", relationship),
        RelationshipChange::Changed { before, .. } => ("changed", before),
    };
    let mut members = vec![
        ("kind".to_string(), Value::String(kind.to_string())),
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
        (
            "link_type".to_string(),
            Value::String(relationship.link_type.abbreviation().to_string()),
        ),
        (
            "lag".to_string(),
            Value::Number(Number::Int(relationship.lag)),
        ),
    ];
    if let RelationshipChange::Changed { after, .. } = change {
        members.push((
            "after_link_type".to_string(),
            Value::String(after.link_type.abbreviation().to_string()),
        ));
        members.push((
            "after_lag".to_string(),
            Value::Number(Number::Int(after.lag)),
        ));
    }
    Value::Object(members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse;
    use cadence_core::compare::compare;
    use cadence_core::model::{Activity, ActivityId, LinkType, Relationship, RelationshipId};
    use time::Month;

    fn jan(day: u8) -> time::Date {
        time::Date::from_calendar_date(2026, Month::January, day).expect("valid date")
    }

    fn chain() -> Project {
        let mut project = Project::new("Bridge & Road", jan(5));
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

    fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
        value.get(key).and_then(Value::as_str)
    }

    #[test]
    fn the_comparison_document_carries_the_changes() {
        let before = chain();
        let mut after = chain();
        after.activities[1].original_duration = 6;

        let comparison = compare(&before, &after);
        let document_text = comparison_export(&after, &comparison);
        let document = parse(&document_text).expect("valid document");

        assert_eq!(text(&document, "kind"), Some("comparison"));
        let durations = document.get("duration_changes").expect("member");
        let Value::Array(durations) = durations else {
            panic!("duration_changes is an array");
        };
        assert_eq!(durations.len(), 1);
        let dates = document.get("date_changes").expect("member");
        let Value::Array(dates) = dates else {
            panic!("date_changes is an array");
        };
        assert_eq!(dates.len(), 1);
        assert!(document.get("project_dates").is_some());
    }

    #[test]
    fn an_empty_comparison_writes_empty_sections() {
        let before = chain();
        let after = chain();

        let comparison = compare(&before, &after);
        let document_text = comparison_export(&after, &comparison);
        let document = parse(&document_text).expect("valid document");

        for member in [
            "added_activities",
            "removed_activities",
            "duration_changes",
            "date_changes",
            "constraint_changes",
            "relationship_changes",
        ] {
            assert_eq!(document.get(member), Some(&Value::Array(Vec::new())));
        }
    }
}
