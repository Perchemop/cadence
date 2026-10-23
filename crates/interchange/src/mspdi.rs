//! Microsoft Project XML (MSPDI) import and export.
//!
//! The reader accepts the subset of the MSPDI schema that Cadence models:
//!
//! | MSPDI element | Cadence field |
//! |---|---|
//! | `Project/Name` | [`Project::name`] |
//! | `Project/StartDate` | [`Project::start_date`] |
//! | `Task/UID` | [`Activity::id`] |
//! | `Task/Name` | [`Activity::name`] |
//! | `Task/Duration` | [`Activity::original_duration`] (8-hour days) |
//! | `Task/ConstraintType`, `Task/ConstraintDate` | [`Activity::constraint`] |
//! | `Task/PercentComplete` | [`Activity::percent_complete`] (basis points) |
//! | `Task/FixedCost` | [`Activity::budget_at_completion`] (minor units) |
//! | `Task/ActualCost` | [`Activity::actual_cost`] (minor units) |
//! | `Task/ActualStart`, `Task/ActualFinish` | [`Activity::actual_start`], [`Activity::actual_finish`] |
//! | `Task/PredecessorLink` | [`Relationship`] (type and lag) |
//! | `Resource/UID`, `Name`, `Type`, `MaxUnits` | [`Resource`] |
//! | `Assignment/TaskUID`, `ResourceUID` | [`Activity::resources`] |
//! | `Task/Baseline` | [`Baseline`] snapshots |
//!
//! Durations are converted at eight working hours per day, matching the
//! default Microsoft Project calendar. Link lag is written in tenths of a
//! minute, which is the unit MSPDI uses, and converted to whole working days.
//! Percent complete is written as a whole percentage because that is what
//! MSPDI's `PercentComplete` holds, so sub-percent precision is rounded away
//! (149 basis points become 1%, 150 become 2%).
//!
//! Two MSPDI constraints have no exact Cadence equivalent: *must start on* is
//! read as *start no earlier than* and *must finish on* as *finish no earlier
//! than*. Everything the reader does not model (cost rate tables, custom
//! fields, calendars) is ignored rather than rejected.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use cadence_core::engine::schedule;
use cadence_core::model::{
    Activity, ActivityBaselineSnapshot, ActivityId, Baseline, BaselineId, Constraint, LinkType,
    Project, Relationship, RelationshipId, Resource, ResourceId, ResourceKind,
};
use time::Date;

use crate::xml::{escape, parse_document, Element};
use crate::{format_money, parse_date_prefix, parse_money, InterchangeError};

/// XML declaration written at the top of every exported document.
const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;
/// Namespace of the MSPDI schema.
const NAMESPACE: &str = "http://schemas.microsoft.com/project";
/// Working hours in one day, used for duration and lag conversion.
const HOURS_PER_DAY: f64 = 8.0;
/// Tenths of a minute in one working day (eight hours).
const TENTHS_PER_DAY: f64 = HOURS_PER_DAY * 60.0 * 10.0;

/// Import a Microsoft Project XML document.
///
/// # Errors
///
/// Returns [`InterchangeError`] when the document is not well formed, is not
/// an MSPDI `<Project>`, is missing a required field, or carries a value the
/// format does not allow.
pub fn import(text: &str) -> Result<Project, InterchangeError> {
    let root = parse_document(text)?;
    if root.name != "Project" {
        return Err(InterchangeError::Unsupported(format!(
            "expected a <Project> root element, found <{}>",
            root.name
        )));
    }

    let name = root
        .text_of("Name")
        .filter(|value| !value.is_empty())
        .unwrap_or("Imported project")
        .to_string();
    let start_text = root
        .text_of("StartDate")
        .ok_or_else(|| InterchangeError::MissingField("Project/StartDate".to_string()))?;
    let start_date =
        parse_date_prefix(start_text).ok_or_else(|| InterchangeError::InvalidField {
            field: "Project/StartDate".to_string(),
            value: start_text.to_string(),
        })?;

    let mut project = Project::new(name, start_date);
    let mut baselines: BTreeMap<i64, Vec<ActivityBaselineSnapshot>> = BTreeMap::new();

    if let Some(tasks) = root.child("Tasks") {
        for task in tasks.children_named("Task") {
            let uid = task_uid(task)?;
            let activity_name = task.text_of("Name").unwrap_or_default().to_string();
            let duration = match task.text_of("Duration") {
                Some(text) => parse_duration(text)?,
                None => 0,
            };

            let mut activity = Activity::new(ActivityId(uid), activity_name, duration);
            activity.constraint = import_constraint(task)?;
            activity.percent_complete = task
                .parse_of::<i64>("PercentComplete")
                .map(|percent| percent.clamp(0, 100) * 100);
            activity.budget_at_completion = task.text_of("FixedCost").and_then(parse_money);
            activity.actual_cost = task.text_of("ActualCost").and_then(parse_money);
            activity.actual_start = task.text_of("ActualStart").and_then(parse_date_prefix);
            activity.actual_finish = task.text_of("ActualFinish").and_then(parse_date_prefix);

            for baseline in task.children_named("Baseline") {
                let number = baseline.parse_of::<i64>("Number").unwrap_or(0);
                let baseline_duration = match baseline.text_of("Duration") {
                    Some(text) => parse_duration(text)?,
                    None => 0,
                };
                baselines
                    .entry(number)
                    .or_default()
                    .push(ActivityBaselineSnapshot {
                        activity_id: ActivityId(uid),
                        baseline_duration,
                        baseline_start: baseline.text_of("Start").and_then(parse_date_prefix),
                        baseline_finish: baseline.text_of("Finish").and_then(parse_date_prefix),
                    });
            }

            project.activities.push(activity);
        }
    }

    let mut next_relationship = 1u64;
    if let Some(tasks) = root.child("Tasks") {
        for task in tasks.children_named("Task") {
            let successor = task_uid(task)?;
            for link in task.children_named("PredecessorLink") {
                let predecessor = link.parse_of::<u64>("PredecessorUID").ok_or_else(|| {
                    InterchangeError::MissingField("PredecessorLink/PredecessorUID".to_string())
                })?;
                let link_type = match link.parse_of::<i64>("Type").unwrap_or(1) {
                    0 => LinkType::Ff,
                    1 => LinkType::Fs,
                    2 => LinkType::Sf,
                    3 => LinkType::Ss,
                    other => {
                        return Err(InterchangeError::InvalidField {
                            field: "PredecessorLink/Type".to_string(),
                            value: other.to_string(),
                        })
                    }
                };
                let lag = link
                    .parse_of::<i64>("LinkLag")
                    .map(|tenths| (tenths as f64 / TENTHS_PER_DAY).round() as i64)
                    .unwrap_or(0);
                project.relationships.push(Relationship {
                    id: RelationshipId(next_relationship),
                    predecessor: ActivityId(predecessor),
                    successor: ActivityId(successor),
                    link_type,
                    lag,
                });
                next_relationship = next_relationship.saturating_add(1);
            }
        }
    }

    if let Some(resources) = root.child("Resources") {
        for resource in resources.children_named("Resource") {
            let Some(uid) = resource.parse_of::<u64>("UID") else {
                continue;
            };
            let resource_name = resource.text_of("Name").unwrap_or_default().to_string();
            // Microsoft Project always writes an empty placeholder resource.
            if resource_name.is_empty() {
                continue;
            }
            let kind = match resource.parse_of::<i64>("Type").unwrap_or(1) {
                0 => ResourceKind::Material,
                1 => ResourceKind::Labour,
                _ => ResourceKind::Equipment,
            };
            let capacity = resource
                .parse_of::<f64>("MaxUnits")
                .map(|units| units.round().max(1.0) as u32)
                .unwrap_or(1);
            project.resources.push(Resource {
                id: ResourceId(uid),
                name: resource_name,
                kind,
                capacity,
            });
        }
    }

    let mut assignments = Vec::new();
    if let Some(section) = root.child("Assignments") {
        for assignment in section.children_named("Assignment") {
            collect_assignment(assignment, None, &mut assignments)?;
        }
    }
    if let Some(resources) = root.child("Resources") {
        for resource in resources.children_named("Resource") {
            let parent = resource.parse_of::<u64>("UID");
            for assignment in resource.children_named("Assignment") {
                collect_assignment(assignment, parent, &mut assignments)?;
            }
        }
    }
    for (task, resource) in assignments {
        if let Some(activity) = project
            .activities
            .iter_mut()
            .find(|activity| activity.id == ActivityId(task))
        {
            let resource = ResourceId(resource);
            if !activity.resources.contains(&resource) {
                activity.resources.push(resource);
            }
        }
    }

    for (number, snapshots) in baselines {
        project.baselines.push(Baseline {
            id: BaselineId(u64::try_from(number.saturating_add(1)).unwrap_or(1)),
            name: format!("Baseline {}", number.saturating_add(1)),
            created_at: start_date,
            snapshots,
        });
    }
    project.baselines.sort_by_key(|baseline| baseline.id);

    Ok(project)
}

/// Export a project as Microsoft Project XML.
///
/// The project is scheduled first so that every task carries its start and
/// finish dates. An empty project has no scheduled dates and still exports.
///
/// # Errors
///
/// Returns [`InterchangeError::Schedule`] when the project cannot be scheduled,
/// because an MSPDI file without dates would be unreadable elsewhere.
pub fn export(project: &Project) -> Result<String, InterchangeError> {
    let schedule =
        schedule(project).map_err(|error| InterchangeError::Schedule(error.to_string()))?;
    let mut output = String::new();

    output.push_str(XML_HEADER);
    output.push('\n');
    let _ = writeln!(output, "<Project xmlns=\"{NAMESPACE}\">");
    let _ = writeln!(output, "  <Name>{}</Name>", escape(&project.name));
    let _ = writeln!(
        output,
        "  <StartDate>{}</StartDate>",
        format_datetime(schedule.project_start, "08:00:00")
    );
    let _ = writeln!(
        output,
        "  <FinishDate>{}</FinishDate>",
        format_datetime(schedule.project_finish, "17:00:00")
    );
    output.push_str("  <ScheduleFromStart>1</ScheduleFromStart>\n");

    output.push_str("  <Tasks>\n");
    for (index, activity) in project.activities.iter().enumerate() {
        output.push_str("    <Task>\n");
        let _ = writeln!(output, "      <UID>{}</UID>", activity.id.0);
        let _ = writeln!(output, "      <ID>{}</ID>", index + 1);
        let _ = writeln!(output, "      <Name>{}</Name>", escape(&activity.name));
        let _ = writeln!(
            output,
            "      <Duration>{}</Duration>",
            format_duration(activity.original_duration)
        );
        let _ = writeln!(
            output,
            "      <Milestone>{}</Milestone>",
            u8::from(activity.original_duration == 0)
        );

        if let Some(result) = schedule.activity(activity.id) {
            let _ = writeln!(
                output,
                "      <Start>{}</Start>",
                format_datetime(result.scheduled_start, "08:00:00")
            );
            let _ = writeln!(
                output,
                "      <Finish>{}</Finish>",
                format_datetime(result.scheduled_finish, "17:00:00")
            );
        }

        let (constraint_type, constraint_date) = export_constraint(activity.constraint);
        let _ = writeln!(
            output,
            "      <ConstraintType>{constraint_type}</ConstraintType>"
        );
        match constraint_date {
            Some(date) => {
                let _ = writeln!(
                    output,
                    "      <ConstraintDate>{}</ConstraintDate>",
                    format_datetime(date, "08:00:00")
                );
            }
            None => output.push_str("      <ConstraintDate>NA</ConstraintDate>\n"),
        }

        if let Some(percent) = activity.percent_complete {
            let _ = writeln!(
                output,
                "      <PercentComplete>{}</PercentComplete>",
                round_to_whole_percent(percent)
            );
        }
        if let Some(cost) = activity.budget_at_completion {
            let _ = writeln!(
                output,
                "      <FixedCost>{}</FixedCost>",
                format_money(cost)
            );
        }
        if let Some(cost) = activity.actual_cost {
            let _ = writeln!(
                output,
                "      <ActualCost>{}</ActualCost>",
                format_money(cost)
            );
        }
        if let Some(start) = activity.actual_start {
            let _ = writeln!(
                output,
                "      <ActualStart>{}</ActualStart>",
                format_datetime(start, "08:00:00")
            );
        }
        if let Some(finish) = activity.actual_finish {
            let _ = writeln!(
                output,
                "      <ActualFinish>{}</ActualFinish>",
                format_datetime(finish, "17:00:00")
            );
        }

        for relationship in project
            .relationships
            .iter()
            .filter(|relationship| relationship.successor == activity.id)
        {
            output.push_str("      <PredecessorLink>\n");
            let _ = writeln!(
                output,
                "        <PredecessorUID>{}</PredecessorUID>",
                relationship.predecessor.0
            );
            let _ = writeln!(
                output,
                "        <Type>{}</Type>",
                export_link_type(relationship.link_type)
            );
            let _ = writeln!(
                output,
                "        <LinkLag>{}</LinkLag>",
                (relationship.lag as f64 * TENTHS_PER_DAY).round() as i64
            );
            output.push_str("        <LagFormat>7</LagFormat>\n");
            output.push_str("      </PredecessorLink>\n");
        }

        for baseline in &project.baselines {
            let Some(snapshot) = baseline
                .snapshots
                .iter()
                .find(|snapshot| snapshot.activity_id == activity.id)
            else {
                continue;
            };
            output.push_str("      <Baseline>\n");
            let _ = writeln!(
                output,
                "        <Number>{}</Number>",
                baseline.id.0.saturating_sub(1)
            );
            if let Some(start) = snapshot.baseline_start {
                let _ = writeln!(
                    output,
                    "        <Start>{}</Start>",
                    format_datetime(start, "08:00:00")
                );
            }
            if let Some(finish) = snapshot.baseline_finish {
                let _ = writeln!(
                    output,
                    "        <Finish>{}</Finish>",
                    format_datetime(finish, "17:00:00")
                );
            }
            let _ = writeln!(
                output,
                "        <Duration>{}</Duration>",
                format_duration(snapshot.baseline_duration)
            );
            output.push_str("      </Baseline>\n");
        }

        output.push_str("    </Task>\n");
    }
    output.push_str("  </Tasks>\n");

    output.push_str("  <Resources>\n");
    for (index, resource) in project.resources.iter().enumerate() {
        output.push_str("    <Resource>\n");
        let _ = writeln!(output, "      <UID>{}</UID>", resource.id.0);
        let _ = writeln!(output, "      <ID>{}</ID>", index + 1);
        let _ = writeln!(output, "      <Name>{}</Name>", escape(&resource.name));
        let _ = writeln!(
            output,
            "      <Type>{}</Type>",
            export_resource_kind(resource.kind)
        );
        let _ = writeln!(
            output,
            "      <MaxUnits>{}</MaxUnits>",
            f64::from(resource.capacity)
        );
        output.push_str("    </Resource>\n");
    }
    output.push_str("  </Resources>\n");

    output.push_str("  <Assignments>\n");
    for activity in &project.activities {
        for resource in &activity.resources {
            output.push_str("    <Assignment>\n");
            let _ = writeln!(output, "      <TaskUID>{}</TaskUID>", activity.id.0);
            let _ = writeln!(output, "      <ResourceUID>{}</ResourceUID>", resource.0);
            output.push_str("      <Units>1</Units>\n");
            output.push_str("    </Assignment>\n");
        }
    }
    output.push_str("  </Assignments>\n");

    output.push_str("</Project>\n");
    Ok(output)
}

fn task_uid(task: &Element) -> Result<u64, InterchangeError> {
    task.parse_of::<u64>("UID")
        .or_else(|| task.parse_of::<u64>("ID"))
        .ok_or_else(|| InterchangeError::MissingField("Task/UID".to_string()))
}

fn collect_assignment(
    element: &Element,
    parent_resource: Option<u64>,
    out: &mut Vec<(u64, u64)>,
) -> Result<(), InterchangeError> {
    let task = element
        .parse_of::<u64>("TaskUID")
        .ok_or_else(|| InterchangeError::MissingField("Assignment/TaskUID".to_string()))?;
    let resource = element
        .parse_of::<u64>("ResourceUID")
        .or(parent_resource)
        .ok_or_else(|| InterchangeError::MissingField("Assignment/ResourceUID".to_string()))?;
    out.push((task, resource));
    Ok(())
}

fn import_constraint(task: &Element) -> Result<Constraint, InterchangeError> {
    let kind = task.parse_of::<i64>("ConstraintType").unwrap_or(0);
    if kind == 0 {
        return Ok(Constraint::AsSoonAsPossible);
    }
    if kind == 1 {
        return Ok(Constraint::AsLateAsPossible);
    }
    let date = task
        .text_of("ConstraintDate")
        .and_then(parse_date_prefix)
        .ok_or_else(|| InterchangeError::MissingField("Task/ConstraintDate".to_string()))?;
    Ok(match kind {
        2 | 4 => Constraint::StartNoEarlierThan(date),
        3 | 6 => Constraint::FinishNoEarlierThan(date),
        5 => Constraint::StartNoLaterThan(date),
        7 => Constraint::FinishNoLaterThan(date),
        other => {
            return Err(InterchangeError::InvalidField {
                field: "Task/ConstraintType".to_string(),
                value: other.to_string(),
            })
        }
    })
}

fn export_constraint(constraint: Constraint) -> (i64, Option<Date>) {
    match constraint {
        Constraint::AsSoonAsPossible => (0, None),
        Constraint::AsLateAsPossible => (1, None),
        Constraint::StartNoEarlierThan(date) => (4, Some(date)),
        Constraint::StartNoLaterThan(date) => (5, Some(date)),
        Constraint::FinishNoEarlierThan(date) => (6, Some(date)),
        Constraint::FinishNoLaterThan(date) => (7, Some(date)),
    }
}

fn export_link_type(link_type: LinkType) -> i64 {
    match link_type {
        LinkType::Ff => 0,
        LinkType::Fs => 1,
        LinkType::Sf => 2,
        LinkType::Ss => 3,
    }
}

fn export_resource_kind(kind: ResourceKind) -> i64 {
    match kind {
        ResourceKind::Material => 0,
        ResourceKind::Labour => 1,
        ResourceKind::Equipment => 2,
    }
}

/// MSPDI stores percent complete as a whole percentage, so basis points are
/// rounded to the nearest one (149 → 1%, 150 → 2%).
fn round_to_whole_percent(basis_points: i64) -> i64 {
    (basis_points.saturating_add(50) / 100).clamp(0, 100)
}

fn format_datetime(date: Date, time: &str) -> String {
    format!("{date}T{time}")
}

fn format_duration(days: u32) -> String {
    let hours = u64::from(days) * 8;
    format!("PT{hours}H0M0S")
}

fn parse_duration(text: &str) -> Result<u32, InterchangeError> {
    let trimmed = text.trim();
    if trimmed.eq_ignore_ascii_case("NA") {
        return Ok(0);
    }
    let body = trimmed
        .strip_prefix('P')
        .ok_or_else(|| invalid_duration(trimmed))?;
    let (date_part, time_part) = match body.find('T') {
        Some(index) => (&body[..index], &body[index + 1..]),
        None => (body, ""),
    };
    let days = scalar_sum(date_part, &[('D', 1.0)]);
    let hours = scalar_sum(
        time_part,
        &[('H', 1.0), ('M', 1.0 / 60.0), ('S', 1.0 / 3600.0)],
    );
    let total_days = days + hours / HOURS_PER_DAY;
    if !total_days.is_finite() || total_days < 0.0 {
        return Err(invalid_duration(trimmed));
    }
    Ok(total_days.round() as u32)
}

fn scalar_sum(part: &str, units: &[(char, f64)]) -> f64 {
    let mut total = 0.0;
    let mut number = String::new();
    for character in part.chars() {
        if character.is_ascii_digit() || character == '.' {
            number.push(character);
            continue;
        }
        let value = number.parse::<f64>().unwrap_or(0.0);
        number.clear();
        if let Some((_, factor)) = units.iter().find(|(unit, _)| *unit == character) {
            total += value * factor;
        }
    }
    total
}

fn invalid_duration(value: &str) -> InterchangeError {
    InterchangeError::InvalidField {
        field: "Duration".to_string(),
        value: value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid date")
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
            link_type: LinkType::Fs,
            lag: 1,
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
        let document = export(&project).expect("export");
        let loaded = import(&document).expect("import");

        assert_eq!(loaded.name, project.name);
        assert_eq!(loaded.start_date, project.start_date);
        assert_eq!(loaded.activities.len(), 2);
        assert_eq!(loaded.activities[0].id, ActivityId(1));
        assert_eq!(loaded.activities[0].name, "Survey");
        assert_eq!(loaded.activities[0].original_duration, 2);
        assert_eq!(loaded.activities[1].original_duration, 3);
        assert_eq!(
            loaded.activities[1].constraint,
            Constraint::StartNoEarlierThan(date(2026, Month::January, 8))
        );
        assert_eq!(loaded.activities[0].percent_complete, Some(5000));
        assert_eq!(loaded.activities[0].budget_at_completion, Some(150_000));
        assert_eq!(loaded.activities[0].actual_cost, Some(75_050));
        assert_eq!(
            loaded.activities[0].actual_start,
            Some(date(2026, Month::January, 5))
        );
        assert_eq!(
            loaded.activities[0].actual_finish,
            Some(date(2026, Month::January, 6))
        );

        assert_eq!(loaded.relationships.len(), 1);
        assert_eq!(loaded.relationships[0].predecessor, ActivityId(1));
        assert_eq!(loaded.relationships[0].successor, ActivityId(2));
        assert_eq!(loaded.relationships[0].link_type, LinkType::Fs);
        assert_eq!(loaded.relationships[0].lag, 1);

        assert_eq!(loaded.resources.len(), 2);
        assert_eq!(loaded.resources[0].name, "Crew A");
        assert_eq!(loaded.resources[0].kind, ResourceKind::Labour);
        assert_eq!(loaded.resources[0].capacity, 1);
        assert_eq!(loaded.resources[1].name, "Crane");
        assert_eq!(loaded.resources[1].kind, ResourceKind::Equipment);
        assert_eq!(loaded.resources[1].capacity, 2);

        assert_eq!(loaded.activities[0].resources, vec![ResourceId(1)]);
        assert_eq!(
            loaded.activities[1].resources,
            vec![ResourceId(1), ResourceId(2)]
        );

        assert_eq!(loaded.baselines.len(), 1);
        assert_eq!(loaded.baselines[0].snapshots.len(), 2);
        assert_eq!(loaded.baselines[0].snapshots[0].baseline_duration, 2);
        assert_eq!(
            loaded.baselines[0].snapshots[0].baseline_start,
            Some(date(2026, Month::January, 5))
        );
    }

    #[test]
    fn milestones_are_exported_with_zero_duration() {
        let start = date(2026, Month::January, 5);
        let mut project = Project::new("Milestones", start);
        project
            .activities
            .push(Activity::new(ActivityId(1), "Handover", 0));

        let document = export(&project).expect("export");
        assert!(document.contains("<Duration>PT0H0M0S</Duration>"));
        assert!(document.contains("<Milestone>1</Milestone>"));

        let loaded = import(&document).expect("import");
        assert_eq!(loaded.activities[0].original_duration, 0);
    }

    #[test]
    fn a_hand_written_document_is_read() {
        let document = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Project xmlns="http://schemas.microsoft.com/project">
  <Name>Handover</Name>
  <StartDate>2026-02-02T08:00:00</StartDate>
  <Tasks>
    <Task>
      <UID>10</UID>
      <Name>Design</Name>
      <Duration>PT24H0M0S</Duration>
      <ConstraintType>4</ConstraintType>
      <ConstraintDate>2026-02-03T08:00:00</ConstraintDate>
      <PredecessorLink>
        <PredecessorUID>11</PredecessorUID>
        <Type>3</Type>
        <LinkLag>4800</LinkLag>
      </PredecessorLink>
    </Task>
    <Task>
      <UID>11</UID>
      <Name>Build</Name>
      <Duration>P1D</Duration>
      <ConstraintType>0</ConstraintType>
      <ConstraintDate>NA</ConstraintDate>
    </Task>
  </Tasks>
  <Resources>
    <Resource>
      <UID>0</UID>
      <Name></Name>
    </Resource>
    <Resource>
      <UID>5</UID>
      <Name>Site crew</Name>
      <Type>1</Type>
      <MaxUnits>3.00</MaxUnits>
      <Assignment>
        <TaskUID>10</TaskUID>
        <Units>1</Units>
      </Assignment>
    </Resource>
  </Resources>
</Project>"#;

        let project = import(document).expect("import");
        assert_eq!(project.name, "Handover");
        assert_eq!(project.activities.len(), 2);
        assert_eq!(project.activities[0].id, ActivityId(10));
        assert_eq!(project.activities[0].original_duration, 3);
        assert_eq!(
            project.activities[0].constraint,
            Constraint::StartNoEarlierThan(date(2026, Month::February, 3))
        );
        assert_eq!(project.activities[1].original_duration, 1);
        assert_eq!(project.relationships.len(), 1);
        assert_eq!(project.relationships[0].link_type, LinkType::Ss);
        assert_eq!(project.relationships[0].lag, 1);
        // The empty placeholder resource is skipped.
        assert_eq!(project.resources.len(), 1);
        assert_eq!(project.resources[0].id, ResourceId(5));
        assert_eq!(project.resources[0].capacity, 3);
        // The assignment nested inside the resource is read.
        assert_eq!(project.activities[0].resources, vec![ResourceId(5)]);
    }

    #[test]
    fn a_missing_start_date_is_rejected() {
        let error = import("<Project><Name>No start</Name></Project>").expect_err("no start");
        assert_eq!(
            error,
            InterchangeError::MissingField("Project/StartDate".to_string())
        );
    }

    #[test]
    fn a_non_project_root_is_rejected() {
        let error = import("<Spreadsheet><StartDate>2026-01-05</StartDate></Spreadsheet>")
            .expect_err("wrong root");
        assert!(matches!(error, InterchangeError::Unsupported(_)));
    }

    #[test]
    fn a_dated_constraint_without_a_date_is_rejected() {
        let document = r#"<Project>
            <StartDate>2026-01-05</StartDate>
            <Tasks><Task><UID>1</UID><ConstraintType>4</ConstraintType></Task></Tasks>
        </Project>"#;
        let error = import(document).expect_err("missing constraint date");
        assert_eq!(
            error,
            InterchangeError::MissingField("Task/ConstraintDate".to_string())
        );
    }

    #[test]
    fn durations_convert_at_eight_hours_per_day() {
        assert_eq!(parse_duration("PT0H0M0S").expect("zero"), 0);
        assert_eq!(parse_duration("PT8H0M0S").expect("one"), 1);
        assert_eq!(parse_duration("PT24H0M0S").expect("three"), 3);
        assert_eq!(parse_duration("P2D").expect("two"), 2);
        assert_eq!(parse_duration("P1DT4H0M0S").expect("one and a half"), 2);
        assert!(parse_duration("not a duration").is_err());
    }

    #[test]
    fn percent_complete_is_quantised_to_whole_percents() {
        // MSPDI's PercentComplete is an integer 0-100, so anything finer is
        // rounded to the nearest percent rather than truncated.
        assert_eq!(round_to_whole_percent(0), 0);
        assert_eq!(round_to_whole_percent(49), 0);
        assert_eq!(round_to_whole_percent(50), 1);
        assert_eq!(round_to_whole_percent(149), 1);
        assert_eq!(round_to_whole_percent(150), 2);
        assert_eq!(round_to_whole_percent(5_000), 50);
        assert_eq!(round_to_whole_percent(10_000), 100);

        let mut project = Project::new("Percent", date(2026, Month::January, 5));
        let mut activity = Activity::new(ActivityId(1), "Half a percent", 1);
        activity.percent_complete = Some(150);
        project.activities.push(activity);

        let document = export(&project).expect("export");
        assert!(document.contains("<PercentComplete>2</PercentComplete>"));
        let loaded = import(&document).expect("import");
        assert_eq!(loaded.activities[0].percent_complete, Some(200));
    }

    #[test]
    fn accounts_round_trip_through_minor_units() {
        assert_eq!(parse_money("1500.00"), Some(150_000));
        assert_eq!(parse_money("0"), Some(0));
        assert_eq!(parse_money("-12.34"), Some(-1_234));
        assert_eq!(parse_money("oops"), None);
        assert_eq!(format_money(150_000), "1500.00");
        assert_eq!(format_money(-1_234), "-12.34");
    }
}
