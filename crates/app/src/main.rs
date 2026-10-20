//! Cadence application entry point.
//!
//! With no arguments the binary runs a small demo project through the CPM
//! engine and prints the resulting schedule as an aligned plain-text table.
//! The GUI arrives in Phase 2; this headless path keeps the engine usable and
//! observable from the command line in the meantime.

use std::process::ExitCode;

use cadence_commands::{Command, CommandStack, CreateActivity, CreateRelationship};
use cadence_core::engine::{schedule, Schedule};
use cadence_core::model::{LinkType, Project};
use time::{Date, Month};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("cadence: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let start = Date::from_calendar_date(2026, Month::January, 5)
        .map_err(|error| format!("invalid demo start date: {error}"))?;
    let mut project = Project::new("Cadence Demo Project", start);
    let mut stack = CommandStack::new();

    let site = project.next_activity_id();
    execute(
        &mut stack,
        &mut project,
        Box::new(CreateActivity::new(site, "Site preparation", 3)),
    )?;

    let foundation = project.next_activity_id();
    execute(
        &mut stack,
        &mut project,
        Box::new(CreateActivity::new(foundation, "Foundation", 5)),
    )?;

    let frame = project.next_activity_id();
    execute(
        &mut stack,
        &mut project,
        Box::new(CreateActivity::new(frame, "Frame erection", 4)),
    )?;

    let site_to_foundation = project.next_relationship_id();
    execute(
        &mut stack,
        &mut project,
        Box::new(CreateRelationship::new(
            site_to_foundation,
            site,
            foundation,
            LinkType::Fs,
            1,
        )),
    )?;

    let foundation_to_frame = project.next_relationship_id();
    execute(
        &mut stack,
        &mut project,
        Box::new(CreateRelationship::new(
            foundation_to_frame,
            foundation,
            frame,
            LinkType::Fs,
            0,
        )),
    )?;

    let schedule = schedule(&project).map_err(|error| error.to_string())?;
    print_schedule(&project, &schedule);
    Ok(())
}

fn execute(
    stack: &mut CommandStack,
    project: &mut Project,
    command: Box<dyn Command>,
) -> Result<(), String> {
    stack
        .execute(project, command)
        .map_err(|error| format!("demo setup failed: {error}"))
}

fn print_schedule(project: &Project, schedule: &Schedule) {
    println!("{}", project.name);
    println!("Project start:  {}", schedule.project_start);
    println!("Project finish: {}", schedule.project_finish);
    println!();

    let headers = [
        "Activity",
        "Duration",
        "Early Start",
        "Early Finish",
        "Late Start",
        "Late Finish",
        "Total Float",
        "Critical",
    ];

    let rows: Vec<[String; 8]> = project
        .activities
        .iter()
        .filter_map(|activity| {
            schedule.activity(activity.id).map(|result| {
                let critical = if result.critical { "Yes" } else { "No" };
                [
                    activity.name.clone(),
                    activity.original_duration.to_string(),
                    result.early_start.to_string(),
                    result.early_finish.to_string(),
                    result.late_start.to_string(),
                    result.late_finish.to_string(),
                    result.total_float.to_string(),
                    critical.to_string(),
                ]
            })
        })
        .collect();

    let mut widths = headers.map(str::len);
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.len());
        }
    }

    for (header, width) in headers.iter().zip(widths.iter()) {
        print!("{header:<width$}  ", width = *width);
    }
    println!();

    for width in &widths {
        print!("{}  ", "-".repeat(*width));
    }
    println!();

    for row in &rows {
        for (cell, width) in row.iter().zip(widths.iter()) {
            print!("{cell:<width$}  ", width = *width);
        }
        println!();
    }
}
