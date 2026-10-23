//! Build the sample bridge project, print its Markdown report, and show the
//! interchange round trip in a few lines.
//!
//! Run with:
//!
//! ```text
//! cargo run -p cadence-app --example bridge_report
//! ```

use cadence_core::engine::{capture_baseline, schedule};
use cadence_core::model::{
    Activity, ActivityId, Baseline, BaselineId, Constraint, LinkType, Project, Relationship,
    RelationshipId, Resource, ResourceId, ResourceKind,
};
use cadence_core::report::project_report;
use time::{Date, Month};

fn main() {
    let project = bridge_project();
    let status_date = Date::from_calendar_date(2026, Month::January, 9).expect("status date");

    // The report is the printable artifact: write it anywhere, open it in a
    // browser, and print or export it to PDF from there.
    println!("{}", project_report(&project, status_date));

    // The same project survives a JSON round trip.
    let json = cadence_interchange::json::export(&project);
    let reloaded = cadence_interchange::json::import(&json).expect("the exported JSON re-imports");
    assert_eq!(reloaded.activities.len(), project.activities.len());
    println!(
        "JSON round trip: {} activities preserved",
        reloaded.activities.len()
    );
}

/// A small, fully scheduled bridge project with resources and a baseline.
fn bridge_project() -> Project {
    let start = Date::from_calendar_date(2026, Month::January, 5).expect("start date");
    let mut project = Project::new("Bridge & Road", start);

    let mut survey = Activity::new(ActivityId(1), "Survey", 2);
    survey.budget_at_completion = Some(150_000);
    survey.percent_complete = Some(5000);
    project.activities.push(survey);

    let mut excavate = Activity::new(ActivityId(2), "Excavate", 3);
    excavate.constraint = Constraint::StartNoEarlierThan(
        Date::from_calendar_date(2026, Month::January, 8).expect("constraint date"),
    );
    project.activities.push(excavate);

    let mut pour = Activity::new(ActivityId(3), "Pour foundations", 2);
    pour.budget_at_completion = Some(420_000);
    project.activities.push(pour);

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
        ActivityId(3),
        LinkType::Fs,
        1,
    ));

    let mut crew = Resource::new(ResourceId(1), "Crew A");
    crew.kind = ResourceKind::Labour;
    let mut crane = Resource::with_capacity(ResourceId(2), "Crane", 2);
    crane.kind = ResourceKind::Equipment;
    project.resources.push(crew);
    project.resources.push(crane);

    project.activities[0].resources = vec![ResourceId(1)];
    project.activities[1].resources = vec![ResourceId(1), ResourceId(2)];
    project.activities[2].resources = vec![ResourceId(2)];

    let computed = schedule(&project).expect("the sample project schedules");
    let baseline = capture_baseline(&project, &computed, BaselineId(1), "Board-approved", start);
    project.baselines.push(Baseline::from_snapshots(
        baseline.id,
        baseline.name,
        baseline.created_at,
        baseline.snapshots,
    ));

    project
}
