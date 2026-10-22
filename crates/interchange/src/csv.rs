//! Comma-separated values (CSV) import and export.
//!
//! CSV is the format spreadsheets speak, and task lists usually live in
//! spreadsheets. Cadence exchanges a project as one CSV table with one row
//! per activity:
//!
//! `ID, Name, Duration, Constraint, Constraint date, Predecessors, Resources,
//! Percent complete, Actual start, Actual finish, Budget at completion,
//! Actual cost, Scheduled start, Scheduled finish`
//!
//! Conventions:
//!
//! * `Duration` is whole working days; zero denotes a milestone.
//! * `Constraint` uses the codes `ASAP`, `ALAP`, `SNET`, `SNLT`, `FNET` and
//!   `FNLT`; a dated code requires `Constraint date` (`YYYY-MM-DD`).
//! * `Predecessors` holds semicolon-separated links in the form
//!   `ID[FS|SS|FF|SF][±lag]`, for example `1FS+1;2SS-1`. A bare `ID` is
//!   finish-to-start with no lag.
//! * `Resources` holds semicolon-separated names. Names that do not exist yet
//!   are created as Labour resources with capacity 1, because CSV carries no
//!   kind or capacity columns.
//! * `Percent complete` is a percentage (0–100); `Budget at completion` and
//!   `Actual cost` are decimal amounts. `Actual start` and `Actual finish`
//!   are `YYYY-MM-DD`.
//! * `Scheduled start` and `Scheduled finish` are written by Cadence so that
//!   a round trip can recover the project start date; the reader uses them
//!   only for that. If no date appears anywhere in the file, the imported
//!   project starts today.
//!
//! Fields are quoted and escaped per RFC 4180, and a semicolon inside a list
//! cell is escaped with a backslash, so activity and resource names may
//! contain commas, quotes, newlines and semicolons.

use cadence_core::engine::schedule;
use cadence_core::model::{
    Activity, ActivityId, Constraint, LinkType, Project, Relationship, RelationshipId, Resource,
    ResourceId,
};
use time::Date;

use crate::{
    default_start_date, format_date, format_money, parse_date, parse_money, InterchangeError,
};

/// The columns Cadence writes, in order.
const HEADER: [&str; 14] = [
    "ID",
    "Name",
    "Duration",
    "Constraint",
    "Constraint date",
    "Predecessors",
    "Resources",
    "Percent complete",
    "Actual start",
    "Actual finish",
    "Budget at completion",
    "Actual cost",
    "Scheduled start",
    "Scheduled finish",
];

/// One activity parsed from a CSV row.
struct Row {
    id: ActivityId,
    name: String,
    duration: u32,
    constraint: Constraint,
    percent_complete: Option<i64>,
    actual_start: Option<Date>,
    actual_finish: Option<Date>,
    budget: Option<i64>,
    cost: Option<i64>,
    scheduled_start: Option<Date>,
    resources: Vec<String>,
    predecessors: Vec<(u64, LinkType, i64)>,
}

/// Import a CSV activity table.
///
/// # Errors
///
/// Returns [`InterchangeError`] when the file has no header row, lacks the
/// `ID` or `Name` column, or a row carries a value the format does not allow.
pub fn import(text: &str) -> Result<Project, InterchangeError> {
    let rows = parse_csv(text);
    let Some(header) = rows.first() else {
        return Err(InterchangeError::MissingField("CSV header row".to_string()));
    };

    let column = |name: &str| {
        header
            .iter()
            .position(|value| value.trim().eq_ignore_ascii_case(name))
    };
    let id_column = column("ID").ok_or_else(|| missing_column("ID"))?;
    let name_column = column("Name").ok_or_else(|| missing_column("Name"))?;
    let duration_column = column("Duration").ok_or_else(|| missing_column("Duration"))?;
    let constraint_column = column("Constraint");
    let constraint_date_column = column("Constraint date");
    let predecessors_column = column("Predecessors");
    let resources_column = column("Resources");
    let percent_column = column("Percent complete");
    let actual_start_column = column("Actual start");
    let actual_finish_column = column("Actual finish");
    let budget_column = column("Budget at completion");
    let cost_column = column("Actual cost");
    let scheduled_start_column = column("Scheduled start");

    let mut activities = Vec::new();
    for (line, record) in rows.iter().enumerate().skip(1) {
        let text = |index: Option<usize>| -> Option<&str> {
            index.and_then(|index| record.get(index)).map(|value| value.trim())
        };

        let id = text(Some(id_column))
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| InterchangeError::InvalidField {
                field: "ID".to_string(),
                value: format!("line {}: {:?}", line + 1, text(Some(id_column))),
            })?;
        if activities.iter().any(|row: &Row| row.id == ActivityId(id)) {
            return Err(InterchangeError::InvalidField {
                field: "ID".to_string(),
                value: id.to_string(),
            });
        }

        let name = text(Some(name_column)).unwrap_or_default().to_string();
        let duration = text(Some(duration_column))
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| InterchangeError::InvalidField {
                field: "Duration".to_string(),
                value: format!("line {}: {:?}", line + 1, text(Some(duration_column))),
            })?;
        let constraint = import_constraint(
            text(constraint_column),
            text(constraint_date_column).and_then(parse_date),
        )?;

        let percent_complete = match text(percent_column) {
            Some(value) if !value.is_empty() => {
                let percent: f64 = value.parse().map_err(|_| InterchangeError::InvalidField {
                    field: "Percent complete".to_string(),
                    value: value.to_string(),
                })?;
                if !percent.is_finite() || !(0.0..=100.0).contains(&percent) {
                    return Err(InterchangeError::InvalidField {
                        field: "Percent complete".to_string(),
                        value: value.to_string(),
                    });
                }
                Some((percent * 100.0).round() as i64)
            }
            _ => None,
        };

        let budget = text(budget_column).and_then(parse_money);
        let cost = text(cost_column).and_then(parse_money);
        let actual_start = text(actual_start_column).and_then(parse_date);
        let actual_finish = text(actual_finish_column).and_then(parse_date);
        let scheduled_start = text(scheduled_start_column).and_then(parse_date);

        let resources = text(resources_column)
            .map(|value| split_escaped(value, ';'))
            .unwrap_or_default()
            .into_iter()
            .filter(|name| !name.is_empty())
            .collect();
        let predecessors = text(predecessors_column)
            .map(|value| split_escaped(value, ';'))
            .unwrap_or_default()
            .into_iter()
            .filter(|entry| !entry.trim().is_empty())
            .map(|entry| parse_predecessor(&entry))
            .collect::<Result<Vec<_>, _>>()?;

        activities.push(Row {
            id: ActivityId(id),
            name,
            duration,
            constraint,
            percent_complete,
            actual_start,
            actual_finish,
            budget,
            cost,
            scheduled_start,
            resources,
            predecessors,
        });
    }

    let start = activities
        .iter()
        .filter_map(|row| row.scheduled_start.or(row.actual_start))
        .min()
        .unwrap_or_else(default_start_date);
    let mut project = Project::new("Imported project", start);

    for row in &activities {
        if let (Some(start), Some(finish)) = (row.actual_start, row.actual_finish) {
            if finish < start {
                return Err(InterchangeError::InvalidField {
                    field: "Actual finish".to_string(),
                    value: format!(
                        "activity {}: {finish} is before the actual start {start}",
                        row.id
                    ),
                });
            }
        }
        let mut activity = Activity::new(row.id, row.name.clone(), row.duration);
        activity.constraint = row.constraint;
        activity.percent_complete = row.percent_complete;
        activity.budget_at_completion = row.budget;
        activity.actual_cost = row.cost;
        activity.actual_start = row.actual_start;
        activity.actual_finish = row.actual_finish;
        project.activities.push(activity);
    }

    // Resource names in first-seen order become the dictionary.
    let mut names: Vec<String> = Vec::new();
    for row in &activities {
        for name in &row.resources {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
    }
    for (index, name) in names.iter().enumerate() {
        project
            .resources
            .push(Resource::new(ResourceId(index as u64 + 1), name));
    }
    for row in &activities {
        let activity = project
            .activities
            .iter_mut()
            .find(|activity| activity.id == row.id)
            .expect("row was just pushed");
        for name in &row.resources {
            let resource = project
                .resources
                .iter()
                .find(|resource| &resource.name == name)
                .expect("every name was registered");
            activity.resources.push(resource.id);
        }
    }

    for (index, row) in activities.iter().enumerate() {
        for (predecessor, link_type, lag) in &row.predecessors {
            if !activities
                .iter()
                .any(|candidate| candidate.id.0 == *predecessor)
            {
                return Err(InterchangeError::InvalidField {
                    field: "Predecessors".to_string(),
                    value: format!("activity {} does not exist", predecessor),
                });
            }
            project.relationships.push(Relationship {
                id: RelationshipId(index as u64 + 1),
                predecessor: ActivityId(*predecessor),
                successor: row.id,
                link_type: *link_type,
                lag: *lag,
            });
        }
    }

    Ok(project)
}

/// Export a project as a CSV activity table.
///
/// The project is scheduled first so that the exported rows carry their
/// scheduled dates.
///
/// # Errors
///
/// Returns [`InterchangeError::Schedule`] when the project cannot be scheduled.
pub fn export(project: &Project) -> Result<String, InterchangeError> {
    let schedule =
        schedule(project).map_err(|error| InterchangeError::Schedule(error.to_string()))?;

    let mut rows: Vec<Vec<String>> = vec![HEADER.iter().map(|name| name.to_string()).collect()];
    for activity in &project.activities {
        let mut predecessors = Vec::new();
        for relationship in project
            .relationships
            .iter()
            .filter(|relationship| relationship.successor == activity.id)
        {
            predecessors.push(format_predecessor(relationship));
        }

        let resources: Vec<String> = activity
            .resources
            .iter()
            .filter_map(|id| project.resource(*id))
            .map(|resource| resource.name.clone())
            .collect();

        let result = schedule.activity(activity.id);
        let (code, constraint_date) = export_constraint(activity.constraint);
        rows.push(vec![
            activity.id.0.to_string(),
            activity.name.clone(),
            activity.original_duration.to_string(),
            code.to_string(),
            constraint_date.map(format_date).unwrap_or_default(),
            join_escaped(&predecessors, ';'),
            join_escaped(&resources, ';'),
            activity
                .percent_complete
                .map(|percent| (percent as f64 / 100.0).to_string())
                .unwrap_or_default(),
            activity.actual_start.map(format_date).unwrap_or_default(),
            activity.actual_finish.map(format_date).unwrap_or_default(),
            activity
                .budget_at_completion
                .map(format_money)
                .unwrap_or_default(),
            activity.actual_cost.map(format_money).unwrap_or_default(),
            result
                .map(|result| result.scheduled_start.to_string())
                .unwrap_or_default(),
            result
                .map(|result| result.scheduled_finish.to_string())
                .unwrap_or_default(),
        ]);
    }

    Ok(render_csv(&rows))
}

fn import_constraint(
    code: Option<&str>,
    date: Option<Date>,
) -> Result<Constraint, InterchangeError> {
    let code = code.unwrap_or("ASAP").trim().to_ascii_uppercase();
    match code.as_str() {
        "" | "ASAP" => Ok(Constraint::AsSoonAsPossible),
        "ALAP" => Ok(Constraint::AsLateAsPossible),
        "SNET" => Ok(Constraint::StartNoEarlierThan(require_date(date)?)),
        "SNLT" => Ok(Constraint::StartNoLaterThan(require_date(date)?)),
        "FNET" => Ok(Constraint::FinishNoEarlierThan(require_date(date)?)),
        "FNLT" => Ok(Constraint::FinishNoLaterThan(require_date(date)?)),
        other => Err(InterchangeError::InvalidField {
            field: "Constraint".to_string(),
            value: other.to_string(),
        }),
    }
}

fn require_date(date: Option<Date>) -> Result<Date, InterchangeError> {
    date.ok_or_else(|| {
        InterchangeError::MissingField("Constraint date for a dated constraint".to_string())
    })
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

/// Parse one `Predecessors` entry such as `2SS-1`.
fn parse_predecessor(entry: &str) -> Result<(u64, LinkType, i64), InterchangeError> {
    let invalid = || InterchangeError::InvalidField {
        field: "Predecessors".to_string(),
        value: entry.to_string(),
    };
    let digits: String = entry.chars().take_while(char::is_ascii_digit).collect();
    let id = digits.parse::<u64>().map_err(|_| invalid())?;
    let rest = &entry[digits.len()..];

    let (link_type, rest) = if rest.len() >= 2 {
        let (code, tail) = rest.split_at(2);
        match code.to_ascii_uppercase().as_str() {
            "FS" => (LinkType::Fs, tail),
            "SS" => (LinkType::Ss, tail),
            "FF" => (LinkType::Ff, tail),
            "SF" => (LinkType::Sf, tail),
            _ => (LinkType::Fs, rest),
        }
    } else {
        (LinkType::Fs, rest)
    };

    if rest.is_empty() {
        return Ok((id, link_type, 0));
    }

    let (sign, magnitude) = match rest.strip_prefix('-') {
        Some(tail) => (-1i64, tail),
        None => (1i64, rest.strip_prefix('+').unwrap_or(rest)),
    };
    let lag = magnitude.parse::<i64>().map_err(|_| invalid())?;
    Ok((id, link_type, sign * lag))
}

/// Render one relationship as a `Predecessors` entry.
fn format_predecessor(relationship: &Relationship) -> String {
    let code = match relationship.link_type {
        LinkType::Ff => "FF",
        LinkType::Fs => "FS",
        LinkType::Sf => "SF",
        LinkType::Ss => "SS",
    };
    let mut text = format!("{}{code}", relationship.predecessor.0);
    if relationship.lag != 0 {
        text.push_str(&format!("{:+}", relationship.lag));
    }
    text
}

fn missing_column(name: &str) -> InterchangeError {
    InterchangeError::MissingField(format!("CSV column {name}"))
}

/// Parse CSV text into rows of fields, honouring RFC 4180 quoting.
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut characters = text.chars().peekable();

    while let Some(character) = characters.next() {
        if in_quotes {
            match character {
                '"' => {
                    if characters.peek() == Some(&'"') {
                        characters.next();
                        field.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                other => field.push(other),
            }
            continue;
        }
        match character {
            '"' if field.is_empty() => in_quotes = true,
            ',' => row.push(std::mem::take(&mut field)),
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            other => field.push(other),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }

    rows.into_iter()
        .filter(|row| row.iter().any(|field| !field.trim().is_empty()))
        .collect()
}

/// Render rows back to CSV text.
fn render_csv(rows: &[Vec<String>]) -> String {
    let mut output = String::new();
    for row in rows {
        let cells: Vec<String> = row.iter().map(|field| escape_field(field)).collect();
        output.push_str(&cells.join(","));
        output.push('\n');
    }
    output
}

fn escape_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Split a list cell on `separator`, treating `\` as an escape character.
fn split_escaped(value: &str, separator: char) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == separator {
            items.push(std::mem::take(&mut current));
        } else {
            current.push(character);
        }
    }
    if escaped {
        current.push('\\');
    }
    if !current.is_empty() || !items.is_empty() {
        items.push(current);
    }
    items
}

/// Join list items with `separator`, escaping `\` and the separator.
fn join_escaped(items: &[String], separator: char) -> String {
    items
        .iter()
        .map(|item| {
            item.replace('\\', "\\\\")
                .replace(separator, &format!("\\{separator}"))
        })
        .collect::<Vec<_>>()
        .join(&separator.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mspdi;
    use cadence_core::model::ResourceKind;
    use time::Month;

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid date")
    }

    #[test]
    fn exporting_and_importing_preserves_the_project() {
        let start = date(2026, Month::January, 5);
        let mut project = Project::new("Bridge & Road", start);

        project
            .activities
            .push(Activity::new(ActivityId(1), "Survey", 2));
        let mut excavate = Activity::new(ActivityId(2), "Excavate", 3);
        excavate.constraint = Constraint::StartNoEarlierThan(date(2026, Month::January, 8));
        excavate.percent_complete = Some(5000);
        excavate.budget_at_completion = Some(150_000);
        excavate.actual_cost = Some(75_050);
        excavate.actual_start = Some(date(2026, Month::January, 9));
        project.activities.push(excavate);

        project.relationships.push(Relationship {
            id: RelationshipId(1),
            predecessor: ActivityId(1),
            successor: ActivityId(2),
            link_type: LinkType::Ss,
            lag: -1,
        });

        let mut crane = Resource::new(ResourceId(1), "Crane, #2");
        crane.kind = ResourceKind::Material;
        project.resources.push(crane);
        project.activities[0].resources = vec![ResourceId(1)];
        project.activities[1].resources = vec![ResourceId(1)];

        let text = export(&project).expect("export");
        let loaded = import(&text).expect("import");

        assert_eq!(loaded.activities.len(), 2);
        assert_eq!(loaded.activities[0].name, "Survey");
        assert_eq!(loaded.activities[0].original_duration, 2);
        assert_eq!(loaded.activities[1].original_duration, 3);
        assert_eq!(
            loaded.activities[1].constraint,
            Constraint::StartNoEarlierThan(date(2026, Month::January, 8))
        );
        assert_eq!(loaded.activities[1].percent_complete, Some(5000));
        assert_eq!(loaded.activities[1].budget_at_completion, Some(150_000));
        assert_eq!(loaded.activities[1].actual_cost, Some(75_050));
        assert_eq!(
            loaded.activities[1].actual_start,
            Some(date(2026, Month::January, 9))
        );

        assert_eq!(loaded.relationships.len(), 1);
        assert_eq!(loaded.relationships[0].link_type, LinkType::Ss);
        assert_eq!(loaded.relationships[0].lag, -1);

        // The name with a comma and the capacity default round-trip.
        assert_eq!(loaded.resources.len(), 1);
        assert_eq!(loaded.resources[0].name, "Crane, #2");
        assert_eq!(loaded.resources[0].capacity, 1);
        assert_eq!(loaded.activities[0].resources, vec![ResourceId(1)]);

        // The scheduled start of the first activity restores the project start.
        assert_eq!(loaded.start_date, date(2026, Month::January, 5));
    }

    #[test]
    fn a_hand_written_table_is_read() {
        let text = "\
ID,Name,Duration,Constraint,Constraint date,Predecessors,Resources
7,Design,2,SNET,2026-02-03,,Site crew
8,Build,5,ASAP,,7FS+2;7SS-1,Site crew;Crane
";
        let project = import(text).expect("import");

        assert_eq!(project.activities.len(), 2);
        assert_eq!(project.activities[0].id, ActivityId(7));
        assert_eq!(project.activities[0].original_duration, 2);
        assert_eq!(
            project.activities[0].constraint,
            Constraint::StartNoEarlierThan(date(2026, Month::February, 3))
        );
        assert_eq!(
            project.activities[1].constraint,
            Constraint::AsSoonAsPossible
        );

        assert_eq!(project.resources.len(), 2);
        assert_eq!(project.resources[0].name, "Site crew");
        assert_eq!(project.resources[0].kind, ResourceKind::Labour);
        assert_eq!(project.resources[1].name, "Crane");

        assert_eq!(project.activities[0].resources, vec![ResourceId(1)]);
        assert_eq!(
            project.activities[1].resources,
            vec![ResourceId(1), ResourceId(2)]
        );

        assert_eq!(project.relationships.len(), 2);
        assert_eq!(project.relationships[0].predecessor, ActivityId(7));
        assert_eq!(project.relationships[0].link_type, LinkType::Fs);
        assert_eq!(project.relationships[0].lag, 2);
        assert_eq!(project.relationships[1].link_type, LinkType::Ss);
        assert_eq!(project.relationships[1].lag, -1);
        assert_eq!(project.relationships[1].successor, ActivityId(8));
    }

    #[test]
    fn predecessor_shorthands_are_parsed() {
        assert_eq!(
            parse_predecessor("3").expect("bare id"),
            (3, LinkType::Fs, 0)
        );
        assert_eq!(
            parse_predecessor("4SF").expect("typed"),
            (4, LinkType::Sf, 0)
        );
        assert_eq!(
            parse_predecessor("5FF+2").expect("positive lag"),
            (5, LinkType::Ff, 2)
        );
        assert_eq!(
            parse_predecessor("6SS-1").expect("negative lag"),
            (6, LinkType::Ss, -1)
        );
        assert!(parse_predecessor("x").is_err());
        assert!(parse_predecessor("7QQ").is_err());
    }

    #[test]
    fn csv_quoting_round_trips_through_the_reader() {
        let rows = vec![
            vec!["a,b".to_string(), "quote \" inside".to_string()],
            vec!["line\nbreak".to_string(), "plain".to_string()],
        ];
        let text = render_csv(&rows);
        assert_eq!(parse_csv(&text), rows);
    }

    #[test]
    fn escaped_semicolons_survive_a_round_trip() {
        let items = vec!["semi;colon".to_string(), "back\\slash".to_string()];
        let joined = join_escaped(&items, ';');
        assert_eq!(split_escaped(&joined, ';'), items);
    }

    #[test]
    fn a_missing_header_is_rejected() {
        let error = import("").expect_err("empty");
        assert_eq!(
            error,
            InterchangeError::MissingField("CSV header row".to_string())
        );
    }

    #[test]
    fn a_missing_name_column_is_rejected() {
        let error = import("ID,Duration\n1,3\n").expect_err("no name");
        assert_eq!(error, missing_column("Name"));
    }

    #[test]
    fn a_dated_constraint_without_a_date_is_rejected() {
        let error = import("ID,Name,Duration,Constraint\n1,A,2,SNET\n").expect_err("no date");
        assert_eq!(
            error,
            InterchangeError::MissingField("Constraint date for a dated constraint".to_string())
        );
    }

    #[test]
    fn a_predecessor_that_does_not_exist_is_rejected() {
        let text = "ID,Name,Duration,Predecessors\n1,A,2,9FS\n";
        let error = import(text).expect_err("unknown predecessor");
        assert!(matches!(error, InterchangeError::InvalidField { .. }));
    }

    #[test]
    fn actual_finish_before_start_is_rejected() {
        let text = "\
ID,Name,Duration,Actual start,Actual finish
1,A,2,2026-02-05,2026-02-04
";
        let error = import(text).expect_err("finish before start");
        assert!(matches!(error, InterchangeError::InvalidField { .. }));
    }

    #[test]
    fn csv_and_mspdi_agree_on_the_same_project() {
        let start = date(2026, Month::January, 5);
        let mut project = Project::new("Round trip", start);
        project
            .activities
            .push(Activity::new(ActivityId(1), "A", 2));
        let mut b = Activity::new(ActivityId(2), "B", 3);
        b.constraint = Constraint::StartNoEarlierThan(date(2026, Month::January, 7));
        project.activities.push(b);
        project.relationships.push(Relationship {
            id: RelationshipId(1),
            predecessor: ActivityId(1),
            successor: ActivityId(2),
            link_type: LinkType::Fs,
            lag: 1,
        });

        let from_csv = import(&export(&project).expect("csv")).expect("csv import");
        let from_xml = mspdi::import(&mspdi::export(&project).expect("xml")).expect("xml import");

        assert_eq!(from_csv.activities, from_xml.activities);
        assert_eq!(from_csv.start_date, from_xml.start_date);
        assert_eq!(from_csv.relationships.len(), from_xml.relationships.len());
    }
}
