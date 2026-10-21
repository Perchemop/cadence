//! Cadence application entry point.
//!
//! With no arguments the binary runs a small demo project through the CPM
//! engine and prints the resulting schedule as an aligned plain-text table.
//! `save` and `load` exercise SQLite project files. The GUI arrives in Phase 2;
//! this headless path keeps the engine usable and observable from the command
//! line in the meantime.

use std::process::ExitCode;

use cadence_commands::{Command, CommandStack, CreateActivity, CreateRelationship};
use cadence_core::engine::{schedule, Schedule};
use cadence_core::model::{LinkType, Project};
use time::{Date, Month};

const USAGE: &str = "\
Cadence — construction project scheduling

Usage:
  cadence               Run the built-in demo project
  cadence save <file>   Save the demo project to a SQLite project file
  cadence load <file>   Load a project file and print its schedule
  cadence --help        Show this help
";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.as_slice() {
        [] => run_demo(),
        [flag] if flag == "--help" || flag == "-h" => {
            print!("{USAGE}");
            Ok(())
        }
        [command, path] if command == "save" => save_demo(path),
        [command, path] if command == "load" => load_and_print(path),
        _ => {
            eprint!("{USAGE}");
            Err("invalid arguments".to_string())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("cadence: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_demo() -> Result<(), String> {
    let project = demo_project()?;
    let schedule = schedule(&project).map_err(|error| error.to_string())?;
    print_schedule(&project, &schedule);
    Ok(())
}

fn save_demo(path: &str) -> Result<(), String> {
    let project = demo_project()?;
    cadence_persistence::save_project(&project, path).map_err(|error| error.to_string())?;
    println!("Saved \"{}\" to {path}", project.name);
    Ok(())
}

fn load_and_print(path: &str) -> Result<(), String> {
    let project = cadence_persistence::load_project(path).map_err(|error| error.to_string())?;
    let schedule = schedule(&project).map_err(|error| error.to_string())?;
    print_schedule(&project, &schedule);
    Ok(())
}

fn demo_project() -> Result<Project, String> {
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

    Ok(project)
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
        "Start",
        "Finish",
        "Early Start",
        "Early Finish",
        "Late Start",
        "Late Finish",
        "Total Float",
        "Critical",
    ];

    let rows: Vec<[String; 10]> = project
        .activities
        .iter()
        .filter_map(|activity| {
            schedule.activity(activity.id).map(|result| {
                let critical = if result.critical { "Yes" } else { "No" };
                [
                    activity.name.clone(),
                    activity.original_duration.to_string(),
                    result.scheduled_start.to_string(),
                    result.scheduled_finish.to_string(),
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
