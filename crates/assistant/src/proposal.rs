//! Structured proposals and their validation against the real project.
//!
//! The assistant never edits the programme. A model response becomes a
//! [`Proposal`]: prose plus typed [`ProposedChange`] values, each carrying a
//! rationale the review panel can cite. Every change is validated against the
//! live project before it can be offered, and the user applies it through the
//! ordinary command layer, so undo and redo behave exactly as for a manual
//! edit.

use cadence_core::model::{ActivityId, Project};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// One typed change the assistant suggests, for review before application.
///
/// Every variant carries a `rationale` so the review panel can show where a
/// suggestion came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposedChange {
    /// Create a new activity.
    ///
    /// A zero duration is rejected: zero denotes a milestone, which the
    /// assistant must not create on the user's behalf.
    CreateActivity {
        /// Activity name.
        name: String,
        /// Duration in working days.
        duration_days: u32,
        /// Why the assistant suggests this activity.
        rationale: String,
    },
    /// Link two existing activities.
    CreateRelationship {
        /// The activity that must come first.
        predecessor: ActivityId,
        /// The activity that follows.
        successor: ActivityId,
        /// Why the assistant suggests this relationship.
        rationale: String,
    },
    /// Change an existing activity's duration.
    SetActivityDuration {
        /// The activity to change.
        activity: ActivityId,
        /// The new duration in working days.
        duration_days: u32,
        /// Why the assistant suggests this change.
        rationale: String,
    },
    /// Change an existing activity's progress.
    SetActivityProgress {
        /// The activity to change.
        activity: ActivityId,
        /// The new progress, as a whole percentage.
        percent: u8,
        /// Why the assistant suggests this change.
        rationale: String,
    },
}

/// A reason a change or proposal cannot be applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum ProposalError {
    /// A change referenced an activity that does not exist.
    #[error("no activity with id {id}")]
    UnknownActivity {
        /// The missing activity.
        id: ActivityId,
    },
    /// A progress value above one hundred.
    #[error("progress {percent} is above 100")]
    PercentOutOfRange {
        /// The offending value.
        percent: u8,
    },
    /// A relationship linking an activity to itself.
    #[error("activity {id} cannot be its own predecessor")]
    SelfRelationship {
        /// The offending activity.
        id: ActivityId,
    },
    /// A relationship that already exists.
    #[error("the relationship {predecessor} -> {successor} already exists")]
    DuplicateRelationship {
        /// The existing predecessor.
        predecessor: ActivityId,
        /// The existing successor.
        successor: ActivityId,
    },
    /// An activity proposed with a blank name.
    #[error("an activity name must not be blank")]
    EmptyName,
    /// A duration of zero, which is reserved for milestones.
    #[error("a duration of zero is reserved for milestones")]
    ZeroDuration,
    /// A proposal with no changes at all.
    #[error("a proposal must contain at least one change")]
    NoChanges,
}

/// Whether an activity with this identifier exists in the project.
fn activity_exists(project: &Project, id: ActivityId) -> bool {
    project.activity(id).is_some()
}

/// Whether a precedence relationship already links these two activities.
fn relationship_exists(project: &Project, predecessor: ActivityId, successor: ActivityId) -> bool {
    project
        .relationships
        .iter()
        .any(|existing| existing.predecessor == predecessor && existing.successor == successor)
}

impl ProposedChange {
    /// Every reason this change cannot be applied to `project`.
    ///
    /// Returns all problems at once so the review panel can show them together
    /// rather than one per round-trip.
    pub fn validate_against_project(&self, project: &Project) -> Vec<ProposalError> {
        let mut errors = Vec::new();
        match self {
            Self::CreateActivity { name, duration_days, .. } => {
                if name.trim().is_empty() {
                    errors.push(ProposalError::EmptyName);
                }
                if *duration_days == 0 {
                    errors.push(ProposalError::ZeroDuration);
                }
            }
            Self::CreateRelationship { predecessor, successor, .. } => {
                let predecessor = *predecessor;
                let successor = *successor;
                if !activity_exists(project, predecessor) {
                    errors.push(ProposalError::UnknownActivity { id: predecessor });
                }
                if !activity_exists(project, successor) {
                    errors.push(ProposalError::UnknownActivity { id: successor });
                }
                if predecessor == successor {
                    errors.push(ProposalError::SelfRelationship { id: predecessor });
                }
                if relationship_exists(project, predecessor, successor) {
                    errors.push(ProposalError::DuplicateRelationship {
                        predecessor,
                        successor,
                    });
                }
            }
            Self::SetActivityDuration { activity, duration_days, .. } => {
                if !activity_exists(project, *activity) {
                    errors.push(ProposalError::UnknownActivity { id: *activity });
                }
                if *duration_days == 0 {
                    errors.push(ProposalError::ZeroDuration);
                }
            }
            Self::SetActivityProgress { activity, percent, .. } => {
                if !activity_exists(project, *activity) {
                    errors.push(ProposalError::UnknownActivity { id: *activity });
                }
                if *percent > 100 {
                    errors.push(ProposalError::PercentOutOfRange { percent: *percent });
                }
            }
        }
        errors
    }
}

/// A model response prepared for review: prose plus typed changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    /// The model's explanation of what it wants to change.
    pub prose: String,
    /// The typed changes it suggests.
    pub changes: Vec<ProposedChange>,
}

impl Proposal {
    /// Build a proposal, rejecting blank prose and empty change lists.
    pub fn new(
        prose: impl Into<String>,
        changes: Vec<ProposedChange>,
    ) -> Result<Self, ProposalError> {
        let prose = prose.into();
        if prose.trim().is_empty() {
            return Err(ProposalError::EmptyName);
        }
        if changes.is_empty() {
            return Err(ProposalError::NoChanges);
        }
        Ok(Self { prose, changes })
    }

    /// Every reason this proposal cannot be applied to `project`.
    ///
    /// Returns all problems at once. A non-empty result means the command layer
    /// must not apply any part of the proposal.
    pub fn validate(&self, project: &Project) -> Vec<ProposalError> {
        let mut errors = Vec::new();
        if self.prose.trim().is_empty() {
            errors.push(ProposalError::EmptyName);
        }
        for change in &self.changes {
            errors.extend(change.validate_against_project(project));
        }
        errors
    }
}
