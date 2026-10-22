//! MPX import and export.
//!
//! MPX (Microsoft Project Exchange) is a line-oriented text format built from
//! `[section]` headers. `[mpx]` and `[project]` hold `key=value` properties;
//! the table sections hold a tab-separated field header row followed by the
//! records. The field header row is what makes MPX extensible: a reader knows
//! which columns it is looking at by name.
//!
//! Cadence writes and reads these sections:
//!
//! | Section | Fields |
//! |---|---|
//! | `[project]` | `Name`, `StartDate`, `FinishDate` |
//! | `[tasks]` | `ID`, `Name`, `Duration`, `Milestone`, `Constraint`, `ConstraintDate` |
//! | `[relations]` | `ID`, `PredecessorID`, `SuccessorID`, `Type`, `Lag` |
//! | `[resources]` | `ID`, `Name`, `Type`, `MaxUnits` |
//! | `[assignments]` | `TaskID`, `ResourceID`, `Units` |
//!
//! Durations are written as working days with a `d` suffix (`3d`), lag as
//! working days, constraint codes as `ASAP`, `ALAP`, `SNET`, `SNLT`, `FNET`
//! and `FNLT`, relation types as `FS`, `SS`, `FF` and `SF`, and resource types
//! as `0` (material), `1` (work) and `2` (cost) — the same codes Microsoft
//! Project uses.

use std::fmt::Write as _;

use cadence_core::engine::schedule;
use cadence_core::model::{
    Activity, ActivityId, Constraint, LinkType, Project, Relationship, RelationshipId, Resource,
    ResourceId, ResourceKind,
};
use time::Date;

use crate::{format_date, parse_date, InterchangeError};

/// One `[section]` of an MPX file.
struct Section {
    name: String,
    /// Field names from the header row of a table section.
    fields: Vec<String>,
    /// Records of a table section, already split into fields.
    records: Vec<Vec<String>>,
    /// `key=value` properties of a property section.
    properties: Vec<(String, String)>,
}

impl Section {
    fn new(name: &str) -> Self {
        Self {
            name: name.trim().to_ascii_lowercase(),
            fields: Vec::new(),
            records: Vec::new(),
            properties: Vec::new(),
        }
    }

    fn property(&self, key: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    }

    fn value<'a>(&self, record: &'a [String], field: &str) -> Option<&'a str> {
        let index = self
            .fields
            .iter()
            .position(|name| name.eq_ignore_ascii_case(field))?;
        record
            .get(index)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }
}

/// Import an MPX document.
///
/// # Errors
///
/// Returns [`InterchangeError`] when the document has no `[project]` section,
/// is missing its start date, or carries a value the format does not allow.
pub fn import(text: &str) -> Result<Project, InterchangeError> {
    let sections = parse_sections(text)?;

    let project_section = sections
        .iter()
        .find(|section| section.name == "project")
        .ok_or_else(|| InterchangeError::MissingField("project section".to_string()))?;
    let name = project_section
        .property("name")
        .filter(|value| !value.is_empty())
        .unwrap_or("Imported project")
        .to_string();
    let start_text = project_section
        .property("startdate")
        .ok_or_else(|| InterchangeError::MissingField("project/StartDate".to_string()))?;
    let start_date =
        parse_date(start_text).ok_or_else(|| InterchangeError::InvalidField {
            field: "project/StartDate".to_string(),
            value: start_text.to_string(),
        })?;

    let mut project = Project::new(name, start_date);

    if let Some(tasks) = sections.iter().find(|section| section.name == "tasks") {
        for record in &tasks.records {
            let id = tasks
                .value(record, "id")
                .and_then(|value| value.parse::<u64>().ok())
                .ok_or_else(|| InterchangeError::MissingField("tasks/ID".to_string()))?;
            let activity_name = tasks.value(record, "name").unwrap_or_default().to_string();
            let duration = tasks
                .value(record, "duration")
                .and_then(parse_days)
                .unwrap_or(0);
            let mut activity = Activity::new(ActivityId(id), activity_name, duration);
            activity.constraint = import_constraint(tasks, record)?;
            project.activities.push(activity);
        }
    }

    let mut next_relationship = 1u64;
    if let Some(relations) = sections.iter().find(|section| section.name == "relations") {
        for record in &relations.records {
            let predecessor = relations
                .value(record, "predecessorid")
                .and_then(|value| value.parse::<u64>().ok())
                .ok_or_else(|| {
                    InterchangeError::MissingField("relations/PredecessorID".to_string())
                })?;
            let successor = relations
                .value(record, "successorid")
                .and_then(|value| value.parse::<u64>().ok())
                .ok_or_else(|| {
                    InterchangeError::MissingField("relations/SuccessorID".to_string())
                })?;
            let link_type = match relations
                .value(record, "type")
                .unwrap_or("FS")
                .to_ascii_uppercase()
                .as_str()
            {
                "FF" => LinkType::Ff,
                "" | "FS" => LinkType::Fs,
                "SF" => LinkType::Sf,
                "SS" => LinkType::Ss,
                other => {
                    return Err(InterchangeError::InvalidField {
                        field: "relations/Type".to_string(),
                        value: other.to_string(),
                    })
                }
            };
            let lag = relations
                .value(record, "lag")
                .and_then(|value| value.parse::<i64>().ok())
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

    if let Some(resources) = sections.iter().find(|section| section.name == "resources") {
        for record in &resources.records {
            let Some(id) = resources
                .value(record, "id")
                .and_then(|value| value.parse::<u64>().ok())
            else {
                continue;
            };
            let resource_name = resources.value(record, "name").unwrap_or_default().to_string();
            if resource_name.is_empty() {
                continue;
            }
            let kind = match resources.value(record, "type").unwrap_or("1") {
                "0" => ResourceKind::Material,
                "2" => ResourceKind::Equipment,
                _ => ResourceKind::Labour,
            };
            let capacity = resources
                .value(record, "maxunits")
                .and_then(|value| value.parse::<f64>().ok())
                .map(|units| units.round().max(1.0) as u32)
                .unwrap_or(1);
            project.resources.push(Resource {
                id: ResourceId(id),
                name: resource_name,
                kind,
                capacity,
            });
        }
    }

    if let Some(assignments) = sections.iter().find(|section| section.name == "assignments") {
        for record in &assignments.records {
            let task = assignments
                .value(record, "taskid")
                .and_then(|value| value.parse::<u64>().ok());
            let resource = assignments
                .value(record, "resourceid")
                .and_then(|value| value.parse::<u64>().ok());
            let (Some(task), Some(resource)) = (task, resource) else {
                return Err(InterchangeError::MissingField(
                    "assignments/TaskID or ResourceID".to_string(),
                ));
            };
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
    }

    Ok(project)
}

/// Export a project as MPX.
///
/// The project is scheduled first so that the `[project]` header can carry a
/// finish date.
///
/// # Errors
///
/// Returns [`InterchangeError::Schedule`] when the project cannot be scheduled.
pub fn export(project: &Project) -> Result<String, InterchangeError> {
    let schedule =
        schedule(project).map_err(|error| InterchangeError::Schedule(error.to_string()))?;
    let mut output = String::new();

    output.push_str("[mpx]\n");
    output.push_str("Version=4.0\n");
    output.push_str("CodePage=ANSI\n");
    output.push_str("CurrencySymbol=$\n");
    output.push_str("CurrencyDigits=2\n");
    output.push_str("DateFormat=0\n\n");

    output.push_str("[project]\n");
    let _ = writeln!(output, "Name={}", sanitize(&project.name));
    let _ = writeln!(output, "StartDate={}", format_date(schedule.project_start));
    let _ = writeln!(output, "FinishDate={}", format_date(schedule.project_finish));
    output.push('\n');

    output.push_str("[tasks]\n");
    output.push_str("ID\tName\tDuration\tMilestone\tConstraint\tConstraintDate\n");
    for activity in &project.activities {
        let (code, date) = export_constraint(activity.constraint);
        let _ = writeln!(
            output,
            "{}\t{}\t{}d\t{}\t{}\t{}",
            activity.id.0,
            sanitize(&activity.name),
            activity.original_duration,
            u8::from(activity.original_duration == 0),
            code,
            date.map(format_date).unwrap_or_else(|| "NA".to_string()),
        );
    }
    output.push('\n');

    output.push_str("[relations]\n");
    output.push_str("ID\tPredecessorID\tSuccessorID\tType\tLag\n");
    for relationship in &project.relationships {
        let _ = writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}",
            relationship.id.0,
            relationship.predecessor.0,
            relationship.successor.0,
            export_link_type(relationship.link_type),
            relationship.lag,
        );
    }
    output.push('\n');

    output.push_str("[resources]\n");
    output.push_str("ID\tName\tType\tMaxUnits\n");
    for resource in &project.resources {
        let _ = writeln!(
            output,
            "{}\t{}\t{}\t{}",
            resource.id.0,
            sanitize(&resource.name),
            export_resource_kind(resource.kind),
            resource.capacity,
        );
    }
    output.push('\n');

    output.push_str("[assignments]\n");
    output.push_str("TaskID\tResourceID\tUnits\n");
    for activity in &project.activities {
        for resource in &activity.resources {
            let _ = writeln!(output, "{}\t{}\t1", activity.id.0, resource.0);
        }
    }

    Ok(output)
}

fn parse_sections(text: &str) -> Result<Vec<Section>, InterchangeError> {
    let mut sections: Vec<Section> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(name) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            sections.push(Section::new(name));
            continue;
        }
        if trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        let Some(section) = sections.last_mut() else {
            return Err(InterchangeError::Unsupported(
                "data appears before the first [section] header".to_string(),
            ));
        };

        if is_table(&section.name) {
            if section.fields.is_empty() {
                section.fields = split_fields(line);
            } else {
                section.records.push(split_fields(line));
            }
        } else if let Some((key, value)) = trimmed.split_once('=') {
            section
                .properties
                .push((key.trim().to_ascii_lowercase(), value.trim().to_string()));
        }
    }

    Ok(sections)
}

fn is_table(name: &str) -> bool {
    matches!(name, "tasks" | "relations" | "resources" | "assignments")
}

fn split_fields(line: &str) -> Vec<String> {
    line.split('\t').map(|field| field.trim().to_string()).collect()
}

fn import_constraint(section: &Section, record: &[String]) -> Result<Constraint, InterchangeError> {
    let code = section
        .value(record, "constraint")
        .unwrap_or("ASAP")
        .to_ascii_uppercase();
    let date = || section.value(record, "constraintdate").and_then(parse_date);
    match code.as_str() {
        "" | "ASAP" => Ok(Constraint::AsSoonAsPossible),
        "ALAP" => Ok(Constraint::AsLateAsPossible),
        "SNET" => Ok(Constraint::StartNoEarlierThan(require_date(date())?)),
        "SNLT" => Ok(Constraint::StartNoLaterThan(require_date(date())?)),
        "FNET" => Ok(Constraint::FinishNoEarlierThan(require_date(date())?)),
        "FNLT" => Ok(Constraint::FinishNoLaterThan(require_date(date())?)),
        other => Err(InterchangeError::InvalidField {
            field: "tasks/Constraint".to_string(),
            value: other.to_string(),
        }),
    }
}

fn require_date(date: Option<Date>) -> Result<Date, InterchangeError> {
    date.ok_or_else(|| InterchangeError::MissingField("tasks/ConstraintDate".to_string()))
}

fn export_constraint(constraint: Constraint) -> (&'static str, Option<Date>) {
    match constraint {
        Constraint::AsSoonAsPossible => ("ASAP", None),
        Constraint::AsLateAsPossible => ("ALAP", None),
        Constraint::StartNoEarlierThan(date) => ("SNET", Some(date)),
        Constraint::StartNoLaterThan(date) => ("SNLT", Some(date)),
        Constraint::FinishNoEarlierThan(date) => ("FNET", Some(date)),
        Constraint::FinishNoLaterThan(date) => ("FNLT", Some(date)),
    }
}

fn export_link_type(link_type: LinkType) -> &'static str {
    match link_type {
        LinkType::Ff => "FF",
        LinkType::Fs => "FS",
        LinkType::Sf => "SF",
        LinkType::Ss => "SS",
    }
}

fn export_resource_kind(kind: ResourceKind) -> u8 {
    match kind {
        ResourceKind::Material => 0,
        ResourceKind::Labour => 1,
        ResourceKind::Equipment => 2,
    }
}

/// Parse a `3d` style duration into whole working days.
fn parse_days(text: &str) -> Option<u32> {
    let trimmed = text.trim();
    let digits = trimmed
        .strip_suffix('d')
        .or_else(|| trimmed.strip_suffix('D'))
        .unwrap_or(trimmed);
    digits
        .trim()
        .parse::<f64>()
        .ok()
        .map(|value| value.round().max(0.0) as u32)
}

/// Replace characters that would break the record layout.
fn sanitize(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadence_core::model::ActivityBaselineSnapshot;
    use time::Month;

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid date")
    }

    fn sample() -> Project {
        let start = date(2026, Month::January, 5);
        let mut project = Project::new("Bridge & Road", start);

        project
            .activities
            .push(Activity::new(ActivityId(1), "Survey", 2));

        let mut excavate = Activity::new(ActivityId(2), "Excavate", 3);
        excavate.constraint = Constraint::StartNoEarlierThan(date(2026, Month::January, 8));
        project.activities.push(excavate);

        project
            .activities
            .push(Activity::new(ActivityId(3), "Handover", 0));

        project.relationships.push(Relationship {
            id: RelationshipId(1),
            predecessor: ActivityId(1),
            successor: ActivityId(2),
            link_type: LinkType::Fs,
            lag: 1,
        });
        project.relationships.push(Relationship {
            id: RelationshipId(2),
            predecessor: ActivityId(2),
            successor: ActivityId(3),
            link_type: LinkType::Ss,
            lag: -2,
        });

        project.resources.push(Resource::new(ResourceId(1), "Crew A"));
        let mut crane = Resource::with_capacity(ResourceId(2), "Crane", 2);
        crane.kind = ResourceKind::Equipment;
        project.resources.push(crane);

        project.activities[0].resources = vec![ResourceId(1)];
        project.activities[1].resources = vec![ResourceId(1), ResourceId(2)];

        // Baselines are not part of the MPX subset, but the field must not
        // break the export.
        project.baselines.push(cadence_core::model::Baseline {
            id: cadence_core::model::BaselineId(1),
            name: "Baseline 1".to_string(),
            created_at: start,
            snapshots: vec![ActivityBaselineSnapshot {
                activity_id: ActivityId(1),
                baseline_duration: 2,
                baseline_start: Some(start),
                baseline_finish: None,
            }],
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
        assert_eq!(loaded.activities.len(), 3);
        assert_eq!(loaded.activities[0].id, ActivityId(1));
        assert_eq!(loaded.activities[0].name, "Survey");
        assert_eq!(loaded.activities[0].original_duration, 2);
        assert_eq!(loaded.activities[2].original_duration, 0);
        assert_eq!(
            loaded.activities[1].constraint,
            Constraint::StartNoEarlierThan(date(2026, Month::January, 8))
        );

        assert_eq!(loaded.relationships.len(), 2);
        assert_eq!(loaded.relationships[0].link_type, LinkType::Fs);
        assert_eq!(loaded.relationships[0].lag, 1);
        assert_eq!(loaded.relationships[1].link_type, LinkType::Ss);
        assert_eq!(loaded.relationships[1].lag, -2);

        assert_eq!(loaded.resources.len(), 2);
        assert_eq!(loaded.resources[0].name, "Crew A");
        assert_eq!(loaded.resources[0].capacity, 1);
        assert_eq!(loaded.resources[1].name, "Crane");
        assert_eq!(loaded.resources[1].kind, ResourceKind::Equipment);
        assert_eq!(loaded.resources[1].capacity, 2);

        assert_eq!(loaded.activities[0].resources, vec![ResourceId(1)]);
        assert_eq!(
            loaded.activities[1].resources,
            vec![ResourceId(1), ResourceId(2)]
        );
    }

    #[test]
    fn a_hand_written_document_is_read() {
        let document = "\
[mpx]
Version=4.0

[project]
Name=Handover
StartDate=2026-02-02
FinishDate=2026-02-10

[tasks]
ID\tName\tDuration\tConstraint\tConstraintDate
7\tDesign\t2d\tSNET\t2026-02-03
8\tBuild\t5\tASAP\tNA

[relations]
ID\tPredecessorID\tSuccessorID\tType\tLag
1\t7\t8\tFS\t2

[resources]
ID\tName\tType\tMaxUnits
3\tSite crew\t1\t3

[assignments]
TaskID\tResourceID\tUnits
7\t3\t1
";
        let project = import(document).expect("import");

        assert_eq!(project.name, "Handover");
        assert_eq!(project.start_date, date(2026, Month::February, 2));
        assert_eq!(project.activities.len(), 2);
        assert_eq!(project.activities[0].id, ActivityId(7));
        assert_eq!(project.activities[0].original_duration, 2);
        assert_eq!(project.activities[1].original_duration, 5);
        assert_eq!(
            project.activities[0].constraint,
            Constraint::StartNoEarlierThan(date(2026, Month::February, 3))
        );
        assert_eq!(
            project.activities[1].constraint,
            Constraint::AsSoonAsPossible
        );
        assert_eq!(project.relationships.len(), 1);
        assert_eq!(project.relationships[0].lag, 2);
        assert_eq!(project.resources[0].capacity, 3);
        assert_eq!(project.activities[0].resources, vec![ResourceId(3)]);
    }

    #[test]
    fn a_document_without_a_project_section_is_rejected() {
        let error = import("[tasks]\nID\tName\n1\tA\n").expect_err("no project");
        assert_eq!(
            error,
            InterchangeError::MissingField("project section".to_string())
        );
    }

    #[test]
    fn a_project_without_a_start_date_is_rejected() {
        let error = import("[project]\nName=No start\n").expect_err("no start");
        assert_eq!(
            error,
            InterchangeError::MissingField("project/StartDate".to_string())
        );
    }

    #[test]
    fn durations_accept_days_or_plain_numbers() {
        assert_eq!(parse_days("3d"), Some(3));
        assert_eq!(parse_days("3"), Some(3));
        assert_eq!(parse_days("4D"), Some(4));
        assert_eq!(parse_days("2.4"), Some(2));
        assert_eq!(parse_days("oops"), None);
    }

    #[test]
    fn tabs_in_names_do_not_break_the_layout() {
        let start = date(2026, Month::January, 5);
        let mut project = Project::new("Tabbed", start);
        project
            .activities
            .push(Activity::new(ActivityId(1), "Line\nbreak\ttab", 1));

        let document = export(&project).expect("export");
        let loaded = import(&document).expect("import");
        assert_eq!(loaded.activities[0].name, "Line break tab");
    }
}
