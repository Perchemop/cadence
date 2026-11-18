//! Mapping accepted proposals onto the ordinary command stack.
//!
//! The assistant proposes; the planner disposes — and whatever the planner
//! accepts must travel through the exact same undoable [`Command`] path as a
//! manual edit, so undo and redo behave identically. This module is that
//! seam: a pure, headless translation from a validated [`Proposal`] to the
//! [`cadence_commands::Command`] values the application's command stack
//! already knows how to apply, in proposal order.
//!
//! The mapping never invents schedule semantics. Identifiers are the next
//! free ones on the live project, and a suggested relationship is created as
//! the plainest finish-to-start link with no lag — the planner edits type
//! and lag afterwards, exactly as for a manually added link. Progress is
//! converted from the proposal's whole percentage to the command layer's
//! basis points at this one boundary.

use cadence_commands::{
    Command, CreateActivity, CreateRelationship, SetActivityDuration, SetActivityProgress,
};
use cadence_core::model::{LinkType, Project};

use crate::proposal::{Proposal, ProposalError, ProposedChange};

/// Map an already-validated [`Proposal`] into boxed [`Command`] values, one
/// per suggested change, in proposal order.
///
/// Validation runs here even though the caller may already have run it: the
/// mapping refuses to produce a *partial* command list, so a proposal that
/// fails [`Proposal::validate`] yields [`ProposalError`] and nothing else.
///
/// # Errors
///
/// Returns the first validation problem when the proposal does not hold
/// against the live project.
pub fn map_proposal_to_commands(
    project: &Project,
    proposal: &Proposal,
) -> Result<Vec<Box<dyn Command>>, ProposalError> {
    if let Some(error) = proposal.validate(project).into_iter().next() {
        return Err(error);
    }

    let mut commands: Vec<Box<dyn Command>> = Vec::with_capacity(proposal.changes.len());
    for change in &proposal.changes {
        match change {
            ProposedChange::CreateActivity {
                name,
                duration_days,
                ..
            } => {
                let id = project.next_activity_id();
                commands.push(Box::new(CreateActivity::new(
                    id,
                    name.clone(),
                    *duration_days,
                )));
            }
            ProposedChange::CreateRelationship {
                predecessor,
                successor,
                ..
            } => {
                let id = project.next_relationship_id();
                commands.push(Box::new(CreateRelationship::new(
                    id,
                    *predecessor,
                    *successor,
                    LinkType::Fs,
                    0,
                )));
            }
            ProposedChange::SetActivityDuration {
                activity,
                duration_days,
                ..
            } => {
                commands.push(Box::new(SetActivityDuration {
                    id: *activity,
                    duration: *duration_days,
                }));
            }
            ProposedChange::SetActivityProgress {
                activity, percent, ..
            } => {
                // The proposal carries a whole percentage (0..=100); the
                // command layer stores basis points (0..=10_000). This is the
                // one conversion point between the two scales.
                let basis_points = i64::from(*percent) * 100;
                commands.push(Box::new(SetActivityProgress {
                    id: *activity,
                    percent_complete: Some(basis_points),
                }));
            }
        }
    }
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadence_commands::CommandStack;
    use cadence_core::model::Activity;
    use time::{Date, Month};

    fn start_date() -> Date {
        Date::from_calendar_date(2026, Month::January, 5).expect("a valid date")
    }

    fn empty_project() -> Project {
        Project::new("Mapping test", start_date())
    }

    fn project_with_one_activity() -> (Project, cadence_core::model::ActivityId) {
        let mut project = empty_project();
        let id = project.next_activity_id();
        project.activities.push(Activity::new(id, "Existing", 3));
        (project, id)
    }

    #[test]
    fn create_activity_maps_to_the_create_command_and_undoes_cleanly() {
        let project = empty_project();
        let proposal = Proposal::new(
            "Add the design activity.",
            vec![ProposedChange::CreateActivity {
                name: "Design system".to_owned(),
                duration_days: 5,
                rationale: "From the scope note.".to_owned(),
            }],
        )
        .expect("well formed");
        let commands = map_proposal_to_commands(&project, &proposal).expect("valid proposal");
        assert_eq!(commands.len(), 1);

        let mut project = project;
        let mut stack = CommandStack::new();
        stack
            .execute(
                &mut project,
                commands.into_iter().next().expect("one command"),
            )
            .expect("command applies");
        assert_eq!(project.activities.len(), 1);
        assert_eq!(project.activities[0].name, "Design system");
        assert_eq!(project.activities[0].original_duration, 5);

        stack.undo(&mut project).expect("undo works");
        assert_eq!(project.activities.len(), 0);
    }

    #[test]
    fn create_relationship_maps_to_a_finish_to_start_link_and_undoes_cleanly() {
        let mut project = empty_project();
        let predecessor = project.next_activity_id();
        project
            .activities
            .push(Activity::new(predecessor, "Dig", 3));
        let successor = project.next_activity_id();
        project.activities.push(Activity::new(successor, "Pour", 5));

        let proposal = Proposal::new(
            "Link the activities.",
            vec![ProposedChange::CreateRelationship {
                predecessor,
                successor,
                rationale: "Pouring cannot start before digging.".to_owned(),
            }],
        )
        .expect("well formed");
        let commands = map_proposal_to_commands(&project, &proposal).expect("valid proposal");
        assert_eq!(commands.len(), 1);

        let mut stack = CommandStack::new();
        stack
            .execute(
                &mut project,
                commands.into_iter().next().expect("one command"),
            )
            .expect("command applies");
        assert_eq!(project.relationships.len(), 1);
        let relationship = &project.relationships[0];
        assert_eq!(relationship.predecessor, predecessor);
        assert_eq!(relationship.successor, successor);
        assert_eq!(relationship.link_type, LinkType::Fs);
        assert_eq!(relationship.lag, 0);

        stack.undo(&mut project).expect("undo works");
        assert_eq!(project.relationships.len(), 0);
    }

    #[test]
    fn set_duration_maps_and_undoes_to_the_previous_duration() {
        let (mut project, id) = project_with_one_activity();
        let proposal = Proposal::new(
            "Extend the activity.",
            vec![ProposedChange::SetActivityDuration {
                activity: id,
                duration_days: 8,
                rationale: "Scope grew.".to_owned(),
            }],
        )
        .expect("well formed");
        let commands = map_proposal_to_commands(&project, &proposal).expect("valid proposal");

        let mut stack = CommandStack::new();
        stack
            .execute(
                &mut project,
                commands.into_iter().next().expect("one command"),
            )
            .expect("applies");
        assert_eq!(project.activity(id).expect("exists").original_duration, 8);
        stack.undo(&mut project).expect("undo works");
        assert_eq!(project.activity(id).expect("exists").original_duration, 3);
    }

    #[test]
    fn set_progress_converts_percent_to_basis_points_and_undoes() {
        let (mut project, id) = project_with_one_activity();
        let proposal = Proposal::new(
            "Record progress.",
            vec![ProposedChange::SetActivityProgress {
                activity: id,
                percent: 75,
                rationale: "Site report.".to_owned(),
            }],
        )
        .expect("well formed");
        let commands = map_proposal_to_commands(&project, &proposal).expect("valid proposal");

        let mut stack = CommandStack::new();
        stack
            .execute(
                &mut project,
                commands.into_iter().next().expect("one command"),
            )
            .expect("applies");
        assert_eq!(
            project.activity(id).expect("exists").percent_complete,
            Some(7_500)
        );
        stack.undo(&mut project).expect("undo works");
        assert_eq!(project.activity(id).expect("exists").percent_complete, None);
    }

    #[test]
    fn a_multi_change_proposal_maps_in_order_and_undo_walks_backwards() {
        let mut project = empty_project();
        let first = project.next_activity_id();
        project.activities.push(Activity::new(first, "Dig", 3));
        let second = project.next_activity_id();
        project.activities.push(Activity::new(second, "Pour", 5));

        let proposal = Proposal::new(
            "Extend and link.",
            vec![
                ProposedChange::SetActivityDuration {
                    activity: first,
                    duration_days: 4,
                    rationale: "Rock expected.".to_owned(),
                },
                ProposedChange::CreateRelationship {
                    predecessor: first,
                    successor: second,
                    rationale: "Sequence.".to_owned(),
                },
            ],
        )
        .expect("well formed");
        let commands = map_proposal_to_commands(&project, &proposal).expect("valid proposal");
        assert_eq!(commands.len(), 2);

        let mut stack = CommandStack::new();
        for command in commands {
            stack.execute(&mut project, command).expect("applies");
        }
        assert_eq!(
            project.activity(first).expect("exists").original_duration,
            4
        );
        assert_eq!(project.relationships.len(), 1);

        // Undo walks backwards through the proposal, one change at a time.
        stack.undo(&mut project).expect("undo");
        assert_eq!(project.relationships.len(), 0);
        stack.undo(&mut project).expect("undo");
        assert_eq!(
            project.activity(first).expect("exists").original_duration,
            3
        );
    }

    #[test]
    fn an_invalid_proposal_yields_an_error_and_no_commands() {
        let project = empty_project();
        let proposal = Proposal::new(
            "Blank change name.",
            vec![ProposedChange::CreateActivity {
                name: "   ".to_owned(),
                duration_days: 3,
                rationale: "bad".to_owned(),
            }],
        )
        .expect("well formed");
        let mapped = map_proposal_to_commands(&project, &proposal);
        assert!(matches!(mapped, Err(ProposalError::EmptyName)));
    }

    #[test]
    fn an_out_of_range_progress_yields_an_error_and_no_commands() {
        let (project, id) = project_with_one_activity();
        let proposal = Proposal::new(
            "Too much progress.",
            vec![ProposedChange::SetActivityProgress {
                activity: id,
                percent: 110,
                rationale: "bad".to_owned(),
            }],
        )
        .expect("well formed");
        let mapped = map_proposal_to_commands(&project, &proposal);
        assert!(matches!(
            mapped,
            Err(ProposalError::PercentOutOfRange { percent: 110 })
        ));
    }
}
