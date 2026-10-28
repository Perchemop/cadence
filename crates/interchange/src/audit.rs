//! Machine-readable audit output.
//!
//! [`crate::schedule`] writes the *derived schedule* of a project; this module
//! writes the derived **audit** of that schedule: the programme-quality checks
//! with their outcomes and findings (`docs/research/schedule-audit.md`). Like
//! the schedule document, it is written by the in-crate JSON machinery, is
//! pretty-printed the same way, and is never read back: rerun the audit rather
//! than trusting a stored report.
//!
//! The document looks like this (`cadence` is this document's own schema
//! version, and `kind` distinguishes the documents this crate writes):
//!
//! ```json
//! {
//!   "cadence": 1,
//!   "kind": "audit",
//!   "project": {
//!     "name": "Bridge & Road",
//!     "start": "2026-01-05",
//!     "finish": "2026-01-09"
//!   },
//!   "status_date": null,
//!   "checks": [
//!     {
//!       "id": "AUDIT-LOGIC",
//!       "category": "structure",
//!       "title": "Logic — open ends",
//!       "outcome": "pass",
//!       "measured": "0 of 2 open activities",
//!       "threshold": "no more than 5.0%",
//!       "summary": "0 of 2 open activities are missing a predecessor or successor (0.0%).",
//!       "findings": []
//!     }
//!   ]
//! }
//! ```
//!
//! Findings carry their kind: an `activity` finding has the activity `id` and
//! `name`, a `relationship` finding has the relationship `id`.

use cadence_core::audit::{AuditReport, CheckCategory, CheckOutcome, CheckResult, Finding};
use cadence_core::engine::Schedule;
use cadence_core::model::Project;
use time::Date;

use crate::json::{Number, Value};
use crate::schedule::to_text;

/// The schema version of the audit document this build writes.
const AUDIT_SCHEMA_VERSION: i64 = 1;

/// Write the audit of a scheduled project as a JSON document.
///
/// `status_date` repeats the configuration the audit ran with, so a consumer
/// can tell which date-sensitive checks were applicable.
#[must_use]
pub fn audit_export(
    project: &Project,
    schedule: &Schedule,
    status_date: Option<Date>,
    report: &AuditReport,
) -> String {
    let document = Value::Object(vec![
        (
            "cadence".to_string(),
            Value::Number(Number::Int(AUDIT_SCHEMA_VERSION)),
        ),
        ("kind".to_string(), Value::String("audit".to_string())),
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
        ("status_date".to_string(), date_json(status_date)),
        (
            "checks".to_string(),
            Value::Array(report.checks.iter().map(check_json).collect()),
        ),
    ]);

    to_text(&document)
}

/// Serialize a document exactly as `schedule::to_text` does.
fn date_json(date: Option<Date>) -> Value {
    match date {
        Some(date) => Value::String(date.to_string()),
        None => Value::Null,
    }
}

/// The stable wire label of a check outcome.
fn outcome_label(outcome: &CheckOutcome) -> &'static str {
    match outcome {
        CheckOutcome::Pass => "pass",
        CheckOutcome::Fail => "fail",
        CheckOutcome::NotApplicable => "not-applicable",
    }
}

/// The stable wire label of a check category.
fn category_label(category: &CheckCategory) -> &'static str {
    match category {
        CheckCategory::Structure => "structure",
        CheckCategory::Logic => "logic",
        CheckCategory::Float => "float",
        CheckCategory::Constraints => "constraints",
        CheckCategory::Dates => "dates",
        CheckCategory::Resources => "resources",
        CheckCategory::Baseline => "baseline",
    }
}

fn check_json(check: &CheckResult) -> Value {
    Value::Object(vec![
        ("id".to_string(), Value::String(check.id.code().to_string())),
        (
            "category".to_string(),
            Value::String(category_label(&check.category).to_string()),
        ),
        ("title".to_string(), Value::String(check.title.clone())),
        (
            "outcome".to_string(),
            Value::String(outcome_label(&check.outcome).to_string()),
        ),
        ("measured".to_string(), optional_text(&check.measured)),
        ("threshold".to_string(), optional_text(&check.threshold)),
        ("summary".to_string(), Value::String(check.summary.clone())),
        (
            "findings".to_string(),
            Value::Array(check.findings.iter().map(finding_json).collect()),
        ),
    ])
}

fn optional_text(value: &Option<String>) -> Value {
    match value {
        Some(text) => Value::String(text.clone()),
        None => Value::Null,
    }
}

fn finding_json(finding: &Finding) -> Value {
    match finding {
        Finding::Activity { id, name, detail } => Value::Object(vec![
            ("kind".to_string(), Value::String("activity".to_string())),
            ("id".to_string(), Value::Number(Number::Int(id.0 as i64))),
            ("name".to_string(), Value::String(name.clone())),
            ("detail".to_string(), Value::String(detail.clone())),
        ]),
        Finding::Relationship { id, detail } => Value::Object(vec![
            (
                "kind".to_string(),
                Value::String("relationship".to_string()),
            ),
            ("id".to_string(), Value::Number(Number::Int(id.0 as i64))),
            ("detail".to_string(), Value::String(detail.clone())),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse;
    use cadence_core::audit::{audit, AuditConfig};
    use cadence_core::engine::schedule;
    use cadence_core::model::{Activity, ActivityId, Relationship, RelationshipId};
    use time::Month;

    fn date(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::January, day).expect("valid date")
    }

    fn sample() -> Project {
        let mut project = Project::new("Bridge & Road", date(5));
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
            cadence_core::model::LinkType::Ss,
            0,
        ));
        project
    }

    fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
        value.get(key).and_then(Value::as_str)
    }

    #[test]
    fn the_audit_document_carries_checks_and_findings() {
        let project = sample();
        let schedule = schedule(&project).expect("schedule succeeds");
        let config = AuditConfig {
            status_date: Some(date(8)),
            ..AuditConfig::default()
        };
        let report = audit(&project, &schedule, None, &config);

        let document_text = audit_export(&project, &schedule, Some(date(8)), &report);
        let document = parse(&document_text).expect("valid document");

        assert_eq!(text(&document, "kind"), Some("audit"));
        assert_eq!(text(&document, "status_date"), Some("2026-01-08"));
        let project_value = document.get("project").expect("project member");
        assert_eq!(text(project_value, "name"), Some("Bridge & Road"));

        let checks = document.get("checks").expect("checks member");
        let Value::Array(checks) = checks else {
            panic!("checks is an array");
        };
        assert_eq!(checks.len(), report.checks.len());

        // The two-activity sample has no start or finish milestones, so the
        // logic check fails with one activity finding per open end.
        let logic = checks
            .iter()
            .find(|check| text(check, "id") == Some("AUDIT-LOGIC"))
            .expect("logic check present");
        assert_eq!(text(logic, "outcome"), Some("fail"));
        assert_eq!(text(logic, "category"), Some("structure"));
        let logic_findings = logic.get("findings").expect("findings member");
        let Value::Array(logic_findings) = logic_findings else {
            panic!("findings is an array");
        };
        assert_eq!(logic_findings.len(), 2);
        assert_eq!(text(&logic_findings[0], "kind"), Some("activity"));
        assert_eq!(logic_findings[0].get("id").and_then(Value::as_i64), Some(1));
        assert_eq!(text(&logic_findings[0], "name"), Some("Survey"));

        // The sample's relationship is start-to-start, so the type check fails
        // with a relationship finding.
        let reltypes = checks
            .iter()
            .find(|check| text(check, "id") == Some("AUDIT-RELTYPES"))
            .expect("relationship-type check present");
        assert_eq!(text(reltypes, "outcome"), Some("fail"));
        let reltypes_findings = reltypes.get("findings").expect("findings member");
        let Value::Array(reltypes_findings) = reltypes_findings else {
            panic!("findings is an array");
        };
        assert_eq!(reltypes_findings.len(), 1);
        assert_eq!(
            text(&reltypes_findings[0], "kind"),
            Some("relationship"),
            "a relationship finding carries its kind"
        );
        assert_eq!(
            reltypes_findings[0]
                .get("id")
                .and_then(Value::as_i64),
            Some(1),
            "the finding names the relationship id"
        );
    }

    #[test]
    fn an_absent_status_date_is_null_in_the_document() {
        let project = sample();
        let schedule = schedule(&project).expect("schedule succeeds");
        let config = AuditConfig::default();
        let report = audit(&project, &schedule, None, &config);

        let document_text = audit_export(&project, &schedule, None, &report);
        let document = parse(&document_text).expect("valid document");

        assert_eq!(document.get("status_date"), Some(&Value::Null));
    }
}
