//! Cadence application entry point.
//!
//! With no arguments the binary runs a small demo project through the CPM
//! engine and prints the resulting schedule as an aligned plain-text table;
//! with `--json` the same schedule is printed as a machine-readable document
//! for scripts and assistive technology (guide section 21.5).
//! `save` and `load` exercise SQLite project files. The GUI arrives in
//! Phase 2; this headless path keeps the engine usable and observable from
//! the command line in the meantime. Failures print a
//! `ERROR <timestamp> cadence: <message>` diagnostic to stderr, because a
//! script reading stdout and a person reading the terminal each need their
//! own copy of the story.

use std::process::ExitCode;

use cadence_commands::{Command, CommandStack, CreateActivity, CreateRelationship};
use cadence_core::engine::{schedule, Schedule};
use cadence_core::model::{LinkType, Project};
use time::{Date, Month};

mod crash_report;
mod update_check;

const USAGE: &str = "\
Cadence — construction project scheduling

Usage:
  cadence [--json]              Run the built-in demo project
  cadence save <file>            Save the demo project to a SQLite project file
  cadence load <file> [--json]   Load a project file and print its schedule
  cadence report <file> [out]    Write a Markdown report (stdout if no file)
  cadence print <file> <out.pdf>  Print a project report to PDF
  cadence version                Show the version and update-check status
  cadence gui [file]             Open the project in the GUI (demo if no file)
  cadence export-xml <in> <out>  Save a project file as Microsoft Project XML
  cadence import-xml <in> <out>  Import Microsoft Project XML into a project file
  cadence export-mpx <in> <out>  Save a project file as MPX
  cadence import-mpx <in> <out>  Import an MPX file into a project file
  cadence export-csv <in> <out>  Save a project file as a CSV activity table
  cadence import-csv <in> <out>  Import a CSV activity table into a project file
  cadence export-json <in> <out> Save a project file as Cadence JSON
  cadence import-json <in> <out> Import Cadence JSON into a project file
  cadence --help                 Show this help

  Exit codes: 0 success, 1 error, 2 usage error
";

fn main() -> ExitCode {
    crash_report::install();
    let (arguments, json) = split_json_flag(std::env::args().skip(1).collect());
    let mut usage_error = false;
    let result = match arguments.as_slice() {
        [] => run_demo(json),
        [flag] if flag == "--help" || flag == "-h" => {
            print!("{USAGE}");
            Ok(())
        }
        [command, path] if command == "save" => save_demo(path),
        [command, path] if command == "load" => load_and_print(path, json),
        [command, path] if command == "report" => print_report(path, None),
        [command, input, output] if command == "report" => print_report(input, Some(output)),
        [command, input, output] if command == "print" => print_report_pdf(input, output),
        [command] if command == "version" => print_version(),
        [command] if command == "gui" => gui_demo(),
        [command, path] if command == "gui" => gui_file(path),
        [command, input, output] if command == "export-xml" => {
            export_interchange(input, output, Format::Mspdi)
        }
        [command, input, output] if command == "import-xml" => {
            import_interchange(input, output, Format::Mspdi)
        }
        [command, input, output] if command == "export-mpx" => {
            export_interchange(input, output, Format::Mpx)
        }
        [command, input, output] if command == "import-mpx" => {
            import_interchange(input, output, Format::Mpx)
        }
        [command, input, output] if command == "export-csv" => {
            export_interchange(input, output, Format::Csv)
        }
        [command, input, output] if command == "import-csv" => {
            import_interchange(input, output, Format::Csv)
        }
        [command, input, output] if command == "export-json" => {
            export_interchange(input, output, Format::Json)
        }
        [command, input, output] if command == "import-json" => {
            import_interchange(input, output, Format::Json)
        }
        _ => {
            usage_error = true;
            eprint!("{USAGE}");
            eprintln!("Run 'cadence --help' for the command reference.");
            Err("invalid arguments".to_string())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            emit_error(&message);
            if usage_error {
                ExitCode::from(2)
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

/// Split the optional `--json` flag off the command-line arguments.
///
/// The flag is accepted only where a schedule is printed — `cadence --json`
/// for the demo and `cadence load <file> --json` — and it comes last, so any
/// other placement of `--json` stays in the arguments and reaches the
/// ordinary usage error explaining the command forms.
fn split_json_flag(arguments: Vec<String>) -> (Vec<String>, bool) {
    if matches!(
        arguments.as_slice(),
        [last] | ["load", _, last] if last == "--json"
    ) {
        let mut arguments = arguments;
        arguments.pop();
        return (arguments, true);
    }
    (arguments, false)
}

/// Write one diagnostic line to stderr, labelled with a severity and the
/// current UTC time (guide section 21.5). The machine-facing documents go to
/// stdout; this line is the same fact for the person watching the terminal.
fn emit_error(message: &str) {
    eprintln!("ERROR {} cadence: {message}", timestamp());
}

/// The current UTC second as `YYYY-MM-DDTHH:MM:SSZ`.
///
/// Built from the components because `Date`'s own display is already
/// ISO-8601 and a seconds-resolution timestamp is all a diagnostic line
/// needs; `now_utc()` carries subsecond digits that would only add noise.
fn timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        now.date(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// Open interchange format selected on the command line.
#[derive(Debug, Clone, Copy)]
enum Format {
    Mspdi,
    Mpx,
    Csv,
    Json,
}

impl Format {
    fn label(self) -> &'static str {
        match self {
            Self::Mspdi => "Microsoft Project XML",
            Self::Mpx => "MPX",
            Self::Csv => "CSV",
            Self::Json => "JSON",
        }
    }
}

fn export_interchange(input: &str, output: &str, format: Format) -> Result<(), String> {
    let project = cadence_persistence::load_project(input).map_err(|error| error.to_string())?;
    let document = match format {
        Format::Mspdi => cadence_interchange::mspdi::export(&project),
        Format::Mpx => cadence_interchange::mpx::export(&project),
        Format::Csv => cadence_interchange::csv::export(&project),
        Format::Json => Ok(cadence_interchange::json::export(&project)),
    }
    .map_err(|error| error.to_string())?;
    std::fs::write(output, document).map_err(|error| format!("cannot write {output}: {error}"))?;
    println!(
        "Exported \"{}\" as {} to {output}",
        project.name,
        format.label()
    );
    Ok(())
}

fn import_interchange(input: &str, output: &str, format: Format) -> Result<(), String> {
    let document =
        std::fs::read_to_string(input).map_err(|error| format!("cannot read {input}: {error}"))?;
    let project = match format {
        Format::Mspdi => cadence_interchange::mspdi::import(&document),
        Format::Mpx => cadence_interchange::mpx::import(&document),
        Format::Csv => cadence_interchange::csv::import(&document),
        Format::Json => cadence_interchange::json::import(&document),
    }
    .map_err(|error| error.to_string())?;
    cadence_persistence::save_project(&project, output).map_err(|error| error.to_string())?;
    println!(
        "Imported \"{}\" from {} ({} activities, {} resources) into {output}",
        project.name,
        format.label(),
        project.activities.len(),
        project.resources.len()
    );
    Ok(())
}

fn run_demo(json: bool) -> Result<(), String> {
    let project = demo_project()?;
    print_schedule_output(&project, json)
}

fn save_demo(path: &str) -> Result<(), String> {
    let project = demo_project()?;
    cadence_persistence::save_project(&project, path).map_err(|error| error.to_string())?;
    println!("Saved \"{}\" to {path}", project.name);
    Ok(())
}

fn load_and_print(path: &str, json: bool) -> Result<(), String> {
    let project = cadence_persistence::load_project(path).map_err(|error| error.to_string())?;
    print_schedule_output(&project, json)
}

/// Schedule a project and print the result: the aligned plain-text table by
/// default, or the JSON schedule document when `--json` asked for
/// machine-readable output.
///
/// In JSON mode stdout carries only the document. If the schedule cannot be
/// produced, the error document is still printed to stdout — a `kind` of
/// `"error"` is a valid answer — and reporting the same failure to the
/// person running the command is left to the caller, who decides the exit
/// code.
fn print_schedule_output(project: &Project, json: bool) -> Result<(), String> {
    match schedule(project) {
        Err(error) => {
            let message = error.to_string();
            if json {
                print!("{}", cadence_interchange::schedule::schedule_error(&message));
            }
            Err(message)
        }
        Ok(schedule) => {
            if json {
                print!(
                    "{}",
                    cadence_interchange::schedule::schedule_export(project, &schedule)
                );
            } else {
                print_schedule(project, &schedule);
            }
            Ok(())
        }
    }
}

/// Write a Markdown project report to stdout or to a file.
fn print_report(path: &str, output: Option<&str>) -> Result<(), String> {
    let project = cadence_persistence::load_project(path).map_err(|error| error.to_string())?;
    let report = cadence_core::report::project_report(&project, today());
    match output {
        Some(output) => {
            std::fs::write(output, &report)
                .map_err(|error| format!("cannot write {output}: {error}"))?;
            println!("Report written to {output}");
        }
        None => print!("{report}"),
    }
    Ok(())
}

/// Print a project report to a PDF file.
fn print_report_pdf(path: &str, output: &str) -> Result<(), String> {
    let project = cadence_persistence::load_project(path).map_err(|error| error.to_string())?;
    let pdf = cadence_core::report::project_report_pdf(&project, today());
    std::fs::write(output, &pdf).map_err(|error| format!("cannot write {output}: {error}"))?;
    println!("Printed {} page(s) to {output}", pdf_pages(&pdf));
    Ok(())
}

/// Count the pages of a generated document, for the status line.
fn pdf_pages(bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(bytes);
    text.matches("/Type /Page ").count() + text.matches("/Type /Page\n").count()
}

/// Print the version and the update-check status.
fn print_version() -> Result<(), String> {
    println!("Cadence {}", update_check::current_version());
    match update_check::check_for_update() {
        update_check::UpdateStatus::UpToDate => println!("Up to date."),
        update_check::UpdateStatus::UpdateAvailable(version) => {
            println!("Update available: {version}");
        }
        update_check::UpdateStatus::Stub => println!(
            "Update checks are not wired up yet (planned for a later release); \
             no network request was made."
        ),
    }
    Ok(())
}

/// Today, as the report status date.
fn today() -> Date {
    time::OffsetDateTime::now_utc().date()
}

fn gui_demo() -> Result<(), String> {
    let project = demo_project()?;
    cadence_ui_egui::run(project)
}

fn gui_file(path: &str) -> Result<(), String> {
    let project = cadence_persistence::load_project(path).map_err(|error| error.to_string())?;
    cadence_ui_egui::run(project)
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
