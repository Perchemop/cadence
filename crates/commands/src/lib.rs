//! Typed command layer: the single mutation path for a Cadence project.
//!
//! Every mutation of project state is expressed as a [`Command`], applied
//! through a [`CommandStack`] that supports undo and redo. This keeps the
//! engine free of UI concerns and makes every mutation testable and (later)
//! scriptable.
//!
//! Commands are deliberately small value types. Applying a command returns the
//! command that reverses it; the stack keeps both directions so undo and redo
//! never need to inspect project state heuristically.

use cadence_core::calendar_set::{CalendarId, NamedCalendar};
use cadence_core::engine::{capture_baseline, capture_lookahead_snapshot, schedule};
use cadence_core::model::{
    Activity, ActivityId, Baseline, BaselineId, Calendar, Constraint, LinkType, LookaheadSnapshot,
    LookaheadSnapshotEntry, LookaheadSnapshotId, Project, Relationship, RelationshipId, Resource,
    ResourceId, ResourceKind,
};
use cadence_core::wbs::{WbsId, WbsNode};
use time::Date;

/// A reversible mutation of a [`Project`].
pub trait Command {
    /// Human-readable description of what this command does.
    fn description(&self) -> String;

    /// Apply the command, returning the inverse command for undo.
    ///
    /// Implementations must validate before mutating so a failed command does
    /// not leave the project partially changed.
    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String>;
}

/// One reversible operation in the history.
struct HistoryEntry {
    forward: Box<dyn Command>,
    inverse: Box<dyn Command>,
}

/// Undo/redo stack over a [`Project`].
pub struct CommandStack {
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    /// Maximum undo depth; `None` means unlimited.
    limit: Option<usize>,
}

impl CommandStack {
    /// Create a stack with the default undo depth of 200 operations.
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            limit: Some(200),
        }
    }

    /// Create a stack with an explicit undo depth.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            limit: Some(limit),
        }
    }

    /// Apply a command, clearing the redo stack.
    ///
    /// # Errors
    ///
    /// Returns the command's error string and leaves the project unchanged
    /// (commands validate before mutating).
    pub fn execute(
        &mut self,
        project: &mut Project,
        mut command: Box<dyn Command>,
    ) -> Result<(), String> {
        let inverse = command.apply(project)?;
        self.undo_stack.push(HistoryEntry {
            forward: command,
            inverse,
        });
        if self
            .limit
            .is_some_and(|limit| self.undo_stack.len() > limit)
        {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        Ok(())
    }

    /// Whether there is an operation to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there is an operation to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Undo the most recent operation.
    ///
    /// # Errors
    ///
    /// Returns an error when there is nothing to undo or the inverse command
    /// rejects the current project state.
    pub fn undo(&mut self, project: &mut Project) -> Result<String, String> {
        let Some(mut entry) = self.undo_stack.pop() else {
            return Err("nothing to undo".to_string());
        };
        let description = entry.forward.description();
        let result = entry.inverse.apply(project);
        match result {
            Ok(inverse_of_undo) => {
                self.redo_stack.push(HistoryEntry {
                    forward: inverse_of_undo,
                    inverse: entry.inverse,
                });
                Ok(description)
            }
            Err(error) => {
                // Keep the history intact so a later undo can succeed once the
                // project state permits it again.
                self.undo_stack.push(entry);
                Err(error)
            }
        }
    }

    /// Redo the most recently undone operation.
    ///
    /// # Errors
    ///
    /// Returns an error when there is nothing to redo or the command rejects
    /// the current project state.
    pub fn redo(&mut self, project: &mut Project) -> Result<String, String> {
        let Some(mut entry) = self.redo_stack.pop() else {
            return Err("nothing to redo".to_string());
        };
        let description = entry.forward.description();
        let result = entry.forward.apply(project);
        match result {
            Ok(inverse_of_redo) => {
                self.undo_stack.push(HistoryEntry {
                    forward: entry.forward,
                    inverse: inverse_of_redo,
                });
                Ok(description)
            }
            Err(error) => {
                self.redo_stack.push(entry);
                Err(error)
            }
        }
    }

    /// Descriptions of the operations that would be undone, oldest first.
    pub fn undo_descriptions(&self) -> Vec<String> {
        self.undo_stack
            .iter()
            .map(|entry| entry.forward.description())
            .collect()
    }
}

impl Default for CommandStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateActivity {
    /// Identifier to assign.
    pub id: ActivityId,
    /// Activity name.
    pub name: String,
    /// Duration in working days.
    pub duration: u32,
}

impl CreateActivity {
    /// Construct a create-activity command.
    pub fn new(id: ActivityId, name: impl Into<String>, duration: u32) -> Self {
        Self {
            id,
            name: name.into(),
            duration,
        }
    }
}

impl Command for CreateActivity {
    fn description(&self) -> String {
        format!("create activity \"{}\"", self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if project.activity(self.id).is_some() {
            return Err(format!("activity {} already exists", self.id));
        }
        project
            .activities
            .push(Activity::new(self.id, self.name.clone(), self.duration));
        Ok(Box::new(DeleteActivity { id: self.id }))
    }
}

/// Delete an activity and any relationships that reference it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteActivity {
    /// Identifier of the activity to delete.
    pub id: ActivityId,
}

impl Command for DeleteActivity {
    fn description(&self) -> String {
        format!("delete activity {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(position) = project
            .activities
            .iter()
            .position(|activity| activity.id == self.id)
        else {
            return Err(format!("activity {} does not exist", self.id));
        };

        let activity = project.activities.remove(position);
        let mut relationships = Vec::new();
        let mut kept = Vec::with_capacity(project.relationships.len());
        for relationship in project.relationships.drain(..) {
            let relationship_position = kept.len();
            if relationship.predecessor == self.id || relationship.successor == self.id {
                relationships.push((relationship_position, relationship));
            } else {
                kept.push(relationship);
            }
        }
        project.relationships = kept;

        Ok(Box::new(RestoreActivity {
            activity,
            position,
            relationships,
        }))
    }
}

/// Restore a previously deleted activity and its relationships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreActivity {
    /// Activity snapshot being restored.
    pub activity: Activity,
    /// Original position in the activity list.
    pub position: usize,
    /// Relationships removed with the activity, in removal order with the
    /// position each occupied when it was removed.
    pub relationships: Vec<(usize, Relationship)>,
}

impl Command for RestoreActivity {
    fn description(&self) -> String {
        format!("restore activity \"{}\"", self.activity.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = self.activity.id;
        if project.activity(id).is_some() {
            return Err(format!("activity {} already exists", id));
        }
        for (_, relationship) in &self.relationships {
            if project.relationship(relationship.id).is_some() {
                return Err(format!("relationship {} already exists", relationship.id));
            }
            let predecessor_exists = relationship.predecessor == id
                || project.activity(relationship.predecessor).is_some();
            let successor_exists =
                relationship.successor == id || project.activity(relationship.successor).is_some();
            if !predecessor_exists {
                return Err(format!(
                    "predecessor activity {} does not exist",
                    relationship.predecessor
                ));
            }
            if !successor_exists {
                return Err(format!(
                    "successor activity {} does not exist",
                    relationship.successor
                ));
            }
        }

        let position = self.position.min(project.activities.len());
        project.activities.insert(position, self.activity.clone());
        // Removal happened left-to-right, so reinsertion must happen
        // right-to-left to recover the original ordering.
        for (position, relationship) in self.relationships.iter().rev() {
            let position = (*position).min(project.relationships.len());
            project
                .relationships
                .insert(position, (*relationship).clone());
        }

        Ok(Box::new(DeleteActivity { id }))
    }
}

/// Create a precedence relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRelationship {
    /// Identifier to assign.
    pub id: RelationshipId,
    /// Predecessor activity.
    pub predecessor: ActivityId,
    /// Successor activity.
    pub successor: ActivityId,
    /// Relationship type.
    pub link_type: LinkType,
    /// Lag in working days; negative values are leads.
    pub lag: i64,
}

impl CreateRelationship {
    /// Construct a create-relationship command.
    pub fn new(
        id: RelationshipId,
        predecessor: ActivityId,
        successor: ActivityId,
        link_type: LinkType,
        lag: i64,
    ) -> Self {
        Self {
            id,
            predecessor,
            successor,
            link_type,
            lag,
        }
    }
}

impl Command for CreateRelationship {
    fn description(&self) -> String {
        format!(
            "create {} relationship {} -> {}",
            self.link_type, self.predecessor, self.successor
        )
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if project.activity(self.predecessor).is_none() {
            return Err(format!(
                "predecessor activity {} does not exist",
                self.predecessor
            ));
        }
        if project.activity(self.successor).is_none() {
            return Err(format!(
                "successor activity {} does not exist",
                self.successor
            ));
        }
        if project.relationship(self.id).is_some() {
            return Err(format!("relationship {} already exists", self.id));
        }

        project.relationships.push(Relationship::new(
            self.id,
            self.predecessor,
            self.successor,
            self.link_type,
            self.lag,
        ));
        Ok(Box::new(DeleteRelationship { id: self.id }))
    }
}

/// Delete a relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteRelationship {
    /// Identifier of the relationship to delete.
    pub id: RelationshipId,
}

impl Command for DeleteRelationship {
    fn description(&self) -> String {
        format!("delete relationship {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(position) = project
            .relationships
            .iter()
            .position(|relationship| relationship.id == self.id)
        else {
            return Err(format!("relationship {} does not exist", self.id));
        };

        let relationship = project.relationships.remove(position);
        Ok(Box::new(RestoreRelationship {
            relationship,
            position,
        }))
    }
}

/// Restore a previously deleted relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreRelationship {
    /// Relationship snapshot being restored.
    pub relationship: Relationship,
    /// Original position in the relationship list.
    pub position: usize,
}

impl Command for RestoreRelationship {
    fn description(&self) -> String {
        format!("restore relationship {}", self.relationship.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = self.relationship.id;
        if project.relationship(id).is_some() {
            return Err(format!("relationship {} already exists", id));
        }
        if project.activity(self.relationship.predecessor).is_none() {
            return Err(format!(
                "predecessor activity {} does not exist",
                self.relationship.predecessor
            ));
        }
        if project.activity(self.relationship.successor).is_none() {
            return Err(format!(
                "successor activity {} does not exist",
                self.relationship.successor
            ));
        }

        let position = self.position.min(project.relationships.len());
        project
            .relationships
            .insert(position, self.relationship.clone());

        Ok(Box::new(DeleteRelationship { id }))
    }
}

/// Change an activity's duration in working days.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityDuration {
    /// Activity to update.
    pub id: ActivityId,
    /// New duration in working days.
    pub duration: u32,
}

impl Command for SetActivityDuration {
    fn description(&self) -> String {
        format!("set duration of activity {} to {}", self.id, self.duration)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        let previous = activity.original_duration;
        activity.original_duration = self.duration;
        activity.clear_schedule();
        Ok(Box::new(SetActivityDuration {
            id: self.id,
            duration: previous,
        }))
    }
}

/// Change an activity's scheduling constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityConstraint {
    /// Activity to update.
    pub id: ActivityId,
    /// New constraint.
    pub constraint: Constraint,
}

impl Command for SetActivityConstraint {
    fn description(&self) -> String {
        format!(
            "set constraint of activity {} to {}",
            self.id, self.constraint
        )
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        let previous = std::mem::replace(&mut activity.constraint, self.constraint);
        activity.clear_schedule();
        Ok(Box::new(SetActivityConstraint {
            id: self.id,
            constraint: previous,
        }))
    }
}

/// Set the budget at completion of an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityBudget {
    /// Activity to update.
    pub id: ActivityId,
    /// Budget in minor currency units, or `None` to leave it unpriced.
    pub budget_at_completion: Option<i64>,
}

impl Command for SetActivityBudget {
    fn description(&self) -> String {
        match self.budget_at_completion {
            Some(budget) => format!("set budget of activity {} to {}", self.id, budget),
            None => format!("clear the budget of activity {}", self.id),
        }
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        let previous = std::mem::replace(
            &mut activity.budget_at_completion,
            self.budget_at_completion,
        );
        Ok(Box::new(SetActivityBudget {
            id: self.id,
            budget_at_completion: previous,
        }))
    }
}

/// Record how far an activity has progressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityProgress {
    /// Activity to update.
    pub id: ActivityId,
    /// Percent complete in basis points (0 to 10000), or `None` if unknown.
    pub percent_complete: Option<i64>,
}

impl Command for SetActivityProgress {
    fn description(&self) -> String {
        match self.percent_complete {
            Some(percent) => {
                format!(
                    "set activity {} to {percent} basis points complete",
                    self.id
                )
            }
            None => format!("clear the progress of activity {}", self.id),
        }
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if let Some(percent) = self.percent_complete {
            if !(0..=10_000).contains(&percent) {
                return Err(format!(
                    "percent complete must be between 0 and 10000 basis points, got {percent}"
                ));
            }
        }
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        let previous = std::mem::replace(&mut activity.percent_complete, self.percent_complete);
        Ok(Box::new(SetActivityProgress {
            id: self.id,
            percent_complete: previous,
        }))
    }
}

/// Record the actual cost incurred on an activity so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityActualCost {
    /// Activity to update.
    pub id: ActivityId,
    /// Actual cost in minor currency units, or `None` if unknown.
    pub actual_cost: Option<i64>,
}

impl Command for SetActivityActualCost {
    fn description(&self) -> String {
        match self.actual_cost {
            Some(cost) => format!("set actual cost of activity {} to {cost}", self.id),
            None => format!("clear the actual cost of activity {}", self.id),
        }
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        let previous = std::mem::replace(&mut activity.actual_cost, self.actual_cost);
        Ok(Box::new(SetActivityActualCost {
            id: self.id,
            actual_cost: previous,
        }))
    }
}

/// Record the dates an activity actually started and finished.
///
/// Actual dates are recorded progress facts. They are not scheduling
/// constraints: the engine still computes dates from logic, calendars and
/// constraints, and the variance views compare the plan against reality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityActualDates {
    /// Activity to update.
    pub id: ActivityId,
    /// Date work actually started, or `None` if it has not started.
    pub actual_start: Option<Date>,
    /// Date work actually finished, or `None` if it has not finished.
    pub actual_finish: Option<Date>,
}

impl Command for SetActivityActualDates {
    fn description(&self) -> String {
        format!(
            "set actual dates of activity {} to {} – {}",
            self.id,
            date_label(self.actual_start),
            date_label(self.actual_finish),
        )
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if let (Some(start), Some(finish)) = (self.actual_start, self.actual_finish) {
            if finish < start {
                return Err(format!(
                    "actual finish {} is before actual start {finish}",
                    start
                ));
            }
        }
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        let previous_start = std::mem::replace(&mut activity.actual_start, self.actual_start);
        let previous_finish = std::mem::replace(&mut activity.actual_finish, self.actual_finish);
        Ok(Box::new(SetActivityActualDates {
            id: self.id,
            actual_start: previous_start,
            actual_finish: previous_finish,
        }))
    }
}

/// Record who is responsible for an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityOwner {
    /// Activity to update.
    pub id: ActivityId,
    /// The responsible person or crew, or `None` to clear it.
    pub owner: Option<String>,
}

impl Command for SetActivityOwner {
    fn description(&self) -> String {
        match &self.owner {
            Some(owner) => format!("set owner of activity {id} to {owner}", id = self.id),
            None => format!("clear the owner of activity {}", self.id),
        }
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| owner.trim().is_empty())
        {
            return Err("the owner must not be empty".to_string());
        }
        let previous = std::mem::replace(&mut activity.owner, self.owner.clone());
        Ok(Box::new(SetActivityOwner {
            id: self.id,
            owner: previous,
        }))
    }
}

/// Promise a near-term activity in the lookahead, optionally recording why its
/// dates moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityPromise {
    /// Activity to update.
    pub id: ActivityId,
    /// True when the activity is a promised near-term task.
    pub promised: bool,
    /// The delay reason recorded against a promised activity, if any.
    pub delay_reason: Option<String>,
}

impl Command for SetActivityPromise {
    fn description(&self) -> String {
        match (self.promised, &self.delay_reason) {
            (true, Some(reason)) => {
                format!("promise activity {} (delay: {reason})", self.id)
            }
            (true, None) => format!("promise activity {}", self.id),
            (false, _) => format!("withdraw the promise on activity {}", self.id),
        }
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        if let Some(reason) = &self.delay_reason {
            if reason.trim().is_empty() {
                return Err("the delay reason must not be empty".to_string());
            }
        }
        let previous_promised = activity.promised;
        let previous_reason = activity.delay_reason.clone();
        activity.promised = self.promised;
        activity.delay_reason = self.delay_reason.clone();
        Ok(Box::new(SetActivityPromise {
            id: self.id,
            promised: previous_promised,
            delay_reason: previous_reason,
        }))
    }
}

/// Render an optional date for a command description.
fn date_label(date: Option<Date>) -> String {
    date.map_or_else(|| "unknown".to_string(), |date| date.to_string())
}

/// Add a resource to an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddResourceToActivity {
    /// Activity to which the resource is added.
    pub activity_id: ActivityId,
    /// Resource to add.
    pub resource_id: ResourceId,
}

impl Command for AddResourceToActivity {
    fn description(&self) -> String {
        format!(
            "add resource {} to activity {}",
            self.resource_id, self.activity_id
        )
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.activity_id) else {
            return Err(format!("activity {} does not exist", self.activity_id));
        };
        if activity.resources.contains(&self.resource_id) {
            return Err(format!(
                "activity {} already has resource {}",
                self.activity_id, self.resource_id
            ));
        }
        activity.resources.push(self.resource_id);
        activity.clear_schedule();
        Ok(Box::new(RemoveResourceFromActivity {
            activity_id: self.activity_id,
            resource_id: self.resource_id,
        }))
    }
}

/// Remove a resource from an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveResourceFromActivity {
    /// Activity from which the resource is removed.
    pub activity_id: ActivityId,
    /// Resource to remove.
    pub resource_id: ResourceId,
}

impl Command for RemoveResourceFromActivity {
    fn description(&self) -> String {
        format!(
            "remove resource {} from activity {}",
            self.resource_id, self.activity_id
        )
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.activity_id) else {
            return Err(format!("activity {} does not exist", self.activity_id));
        };
        let Some(index) = activity
            .resources
            .iter()
            .position(|&id| id == self.resource_id)
        else {
            return Err(format!(
                "activity {} does not have resource {}",
                self.activity_id, self.resource_id
            ));
        };
        activity.resources.remove(index);
        activity.clear_schedule();
        Ok(Box::new(RestoreResourceOnActivity {
            activity_id: self.activity_id,
            resource_id: self.resource_id,
            index,
        }))
    }
}

/// Put a previously removed resource assignment back where it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreResourceOnActivity {
    /// Activity the resource belongs to.
    pub activity_id: ActivityId,
    /// Resource to restore.
    pub resource_id: ResourceId,
    /// Index the assignment occupied before it was removed.
    pub index: usize,
}

impl Command for RestoreResourceOnActivity {
    fn description(&self) -> String {
        format!(
            "restore resource {} on activity {}",
            self.resource_id, self.activity_id
        )
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.activity_id) else {
            return Err(format!("activity {} does not exist", self.activity_id));
        };
        if activity.resources.contains(&self.resource_id) {
            return Err(format!(
                "activity {} already has resource {}",
                self.activity_id, self.resource_id
            ));
        }
        let index = self.index.min(activity.resources.len());
        activity.resources.insert(index, self.resource_id);
        activity.clear_schedule();
        Ok(Box::new(RemoveResourceFromActivity {
            activity_id: self.activity_id,
            resource_id: self.resource_id,
        }))
    }
}

/// One assignment that was removed together with a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedAssignment {
    /// Activity the resource was assigned to.
    pub activity_id: ActivityId,
    /// Index the assignment occupied in the activity's resource list.
    pub index: usize,
}

/// Add a resource to the project dictionary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateResource {
    /// Identifier of the new resource.
    pub id: ResourceId,
    /// Name of the new resource.
    pub name: String,
    /// Kind of the new resource.
    pub kind: ResourceKind,
    /// Units the resource can supply on one working day.
    pub capacity: u32,
}

impl Command for CreateResource {
    fn description(&self) -> String {
        format!("create resource '{}'", self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if project.resource(self.id).is_some() {
            return Err(format!("resource {} already exists", self.id));
        }
        let name = self.name.trim().to_string();
        ensure_resource_name_is_free(project, self.id, &name)?;
        project.resources.push(Resource {
            id: self.id,
            name,
            kind: self.kind,
            capacity: self.capacity.max(1),
        });
        Ok(Box::new(DeleteResource { id: self.id }))
    }
}

/// Remove a resource from the dictionary and from every activity it is
/// assigned to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteResource {
    /// Resource to remove.
    pub id: ResourceId,
}

impl Command for DeleteResource {
    fn description(&self) -> String {
        format!("delete resource {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(position) = project.resources.iter().position(|item| item.id == self.id) else {
            return Err(format!("resource {} does not exist", self.id));
        };
        let resource = project.resources.remove(position);

        // Detach the resource from every activity, remembering where it was so
        // that undo puts it back exactly where it was.
        let mut assignments = Vec::new();
        for activity in &mut project.activities {
            if let Some(index) = activity.resources.iter().position(|&id| id == self.id) {
                activity.resources.remove(index);
                activity.clear_schedule();
                assignments.push(RemovedAssignment {
                    activity_id: activity.id,
                    index,
                });
            }
        }

        Ok(Box::new(RestoreResource {
            resource,
            position,
            assignments,
        }))
    }
}

/// Put a previously deleted resource back into the dictionary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreResource {
    /// The resource to restore.
    pub resource: Resource,
    /// Index the resource occupied in the dictionary.
    pub position: usize,
    /// Assignments that were removed together with the resource.
    pub assignments: Vec<RemovedAssignment>,
}

impl Command for RestoreResource {
    fn description(&self) -> String {
        format!("restore resource {}", self.resource.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = self.resource.id;
        let resource = self.resource.clone();
        let position = self.position.min(project.resources.len());
        project.resources.insert(position, resource);

        for assignment in &self.assignments {
            let Some(activity) = project.activity_mut(assignment.activity_id) else {
                continue;
            };
            if activity.resources.contains(&id) {
                continue;
            }
            let index = assignment.index.min(activity.resources.len());
            activity.resources.insert(index, id);
            activity.clear_schedule();
        }

        Ok(Box::new(DeleteResource { id }))
    }
}

/// Rename a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameResource {
    /// Resource to rename.
    pub id: ResourceId,
    /// New name.
    pub name: String,
}

impl Command for RenameResource {
    fn description(&self) -> String {
        format!("rename resource {} to '{}'", self.id, self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(resource) = project.resource_mut(self.id) else {
            return Err(format!("resource {} does not exist", self.id));
        };
        let previous = resource.name.clone();
        let name = self.name.trim().to_string();
        ensure_resource_name_is_free(project, self.id, &name)?;
        if let Some(resource) = project.resource_mut(self.id) {
            resource.name = name;
        }
        Ok(Box::new(RenameResource {
            id: self.id,
            name: previous,
        }))
    }
}

/// Change the capacity of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetResourceCapacity {
    /// Resource to update.
    pub id: ResourceId,
    /// New capacity in units per working day.
    pub capacity: u32,
}

impl Command for SetResourceCapacity {
    fn description(&self) -> String {
        format!("set capacity of resource {} to {}", self.id, self.capacity)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(resource) = project.resource_mut(self.id) else {
            return Err(format!("resource {} does not exist", self.id));
        };
        let previous = resource.capacity;
        resource.capacity = self.capacity.max(1);
        Ok(Box::new(SetResourceCapacity {
            id: self.id,
            capacity: previous,
        }))
    }
}

/// Change the kind of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetResourceKind {
    /// Resource to update.
    pub id: ResourceId,
    /// New kind.
    pub kind: ResourceKind,
}

impl Command for SetResourceKind {
    fn description(&self) -> String {
        format!("set kind of resource {} to {:?}", self.id, self.kind)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(resource) = project.resource_mut(self.id) else {
            return Err(format!("resource {} does not exist", self.id));
        };
        let previous = resource.kind;
        resource.kind = self.kind;
        Ok(Box::new(SetResourceKind {
            id: self.id,
            kind: previous,
        }))
    }
}

/// Reject a resource name that another resource already uses.
///
/// Resource names are compared case-insensitively, matching the project file
/// format.
fn ensure_resource_name_is_free(
    project: &Project,
    id: ResourceId,
    name: &str,
) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("resource name must not be empty".to_string());
    }
    if project
        .resources
        .iter()
        .any(|resource| resource.id != id && resource.name.eq_ignore_ascii_case(trimmed))
    {
        return Err(format!("a resource named '{trimmed}' already exists"));
    }
    Ok(())
}

/// Capture a baseline snapshot of the current schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureBaseline {
    /// Name of the baseline (e.g. "Baseline 1", "Board-approved").
    pub name: String,
    /// Date the baseline is captured as of.
    pub created_at: Date,
}

impl Command for CaptureBaseline {
    fn description(&self) -> String {
        format!("capture baseline '{}'", self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let computed = schedule(project).map_err(|error| error.to_string())?;
        let id = project.next_baseline_id();
        let baseline = capture_baseline(project, &computed, id, self.name.clone(), self.created_at);
        project.baselines.push(baseline);
        Ok(Box::new(RemoveBaseline { id }))
    }
}

/// Remove a baseline from the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveBaseline {
    /// Baseline to remove.
    pub id: BaselineId,
}

impl Command for RemoveBaseline {
    fn description(&self) -> String {
        format!("remove baseline {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(position) = project.baselines.iter().position(|item| item.id == self.id) else {
            return Err(format!("baseline {} does not exist", self.id));
        };
        let baseline = project.baselines.remove(position);
        Ok(Box::new(RestoreBaseline { baseline, position }))
    }
}

/// Put a previously removed baseline back where it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreBaseline {
    /// The baseline to restore.
    pub baseline: Baseline,
    /// Index the baseline occupied before it was removed.
    pub position: usize,
}

impl Command for RestoreBaseline {
    fn description(&self) -> String {
        format!("restore baseline {}", self.baseline.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = self.baseline.id;
        let baseline = self.baseline.clone();
        let position = self.position.min(project.baselines.len());
        project.baselines.insert(position, baseline);
        Ok(Box::new(RemoveBaseline { id }))
    }
}

/// Publish a lookahead window as a named, read-only snapshot.
///
/// The snapshot captures the current lookahead entries for the given window
/// so that a later window can be compared against it to detect drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureLookaheadSnapshot {
    /// Name of the snapshot (e.g. "Week 1-2", "Pre-mobilisation").
    pub name: String,
    /// Date the snapshot is captured as of.
    pub created_at: time::Date,
    /// First day of the window that was published.
    pub window_start: time::Date,
    /// Length of the window in calendar weeks.
    pub window_weeks: u32,
}

impl Command for CaptureLookaheadSnapshot {
    fn description(&self) -> String {
        format!(
            "capture lookahead snapshot '{}' ({} weeks from {})",
            self.name, self.window_weeks, self.window_start
        )
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = project.next_lookahead_snapshot_id();
        let snapshot = capture_lookahead_snapshot(
            project,
            &cadence_core::engine::schedule(project).map_err(|error| error.to_string())?,
            id,
            self.name.clone(),
            self.created_at,
            self.window_start,
            self.window_weeks,
        );
        project.lookahead_snapshots.push(snapshot);
        Ok(Box::new(RemoveLookaheadSnapshot { id }))
    }
}

/// Remove a lookahead snapshot from the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveLookaheadSnapshot {
    /// Snapshot to remove.
    pub id: LookaheadSnapshotId,
}

impl Command for RemoveLookaheadSnapshot {
    fn description(&self) -> String {
        format!("remove lookahead snapshot {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(position) = project
            .lookahead_snapshots
            .iter()
            .position(|item| item.id == self.id)
        else {
            return Err(format!("lookahead snapshot {} does not exist", self.id));
        };
        let snapshot = project.lookahead_snapshots.remove(position);
        Ok(Box::new(RestoreLookaheadSnapshot {
            snapshot,
            position,
        }))
    }
}

/// Put a previously removed lookahead snapshot back where it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreLookaheadSnapshot {
    /// The snapshot to restore.
    pub snapshot: LookaheadSnapshot,
    /// Index the snapshot occupied before it was removed.
    pub position: usize,
}

impl Command for RestoreLookaheadSnapshot {
    fn description(&self) -> String {
        format!("restore lookahead snapshot {}", self.snapshot.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = self.snapshot.id;
        let snapshot = self.snapshot.clone();
        let position = self.position.min(project.lookahead_snapshots.len());
        project.lookahead_snapshots.insert(position, snapshot);
        Ok(Box::new(RemoveLookaheadSnapshot { id }))
    }
}

/// Rename an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameActivity {
    /// Activity to update.
    pub id: ActivityId,
    /// New name.
    pub name: String,
}

impl Command for RenameActivity {
    fn description(&self) -> String {
        format!("rename activity {} to \"{}\"", self.id, self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(activity) = project.activity_mut(self.id) else {
            return Err(format!("activity {} does not exist", self.id));
        };
        let previous = std::mem::replace(&mut activity.name, self.name.clone());
        Ok(Box::new(RenameActivity {
            id: self.id,
            name: previous,
        }))
    }
}

/// Create a work-breakdown-structure node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateWbsNode {
    /// Identifier to assign.
    pub id: WbsId,
    /// Node name.
    pub name: String,
    /// Parent node.
    pub parent: WbsId,
}

impl CreateWbsNode {
    /// Construct a create-node command.
    pub fn new(id: WbsId, name: impl Into<String>, parent: WbsId) -> Self {
        Self {
            id,
            name: name.into(),
            parent,
        }
    }
}

impl Command for CreateWbsNode {
    fn description(&self) -> String {
        format!("create WBS node \"{}\"", self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        project
            .wbs
            .add(self.id, self.name.clone(), self.parent)
            .map_err(|error| error.to_string())?;
        Ok(Box::new(DeleteWbsNode { id: self.id }))
    }
}

/// Delete a leaf WBS node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteWbsNode {
    /// Identifier of the node to delete.
    pub id: WbsId,
}

impl Command for DeleteWbsNode {
    fn description(&self) -> String {
        format!("delete WBS node {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if project
            .activities
            .iter()
            .any(|activity| activity.wbs == Some(self.id))
        {
            return Err(format!(
                "WBS node {} still has activities assigned",
                self.id
            ));
        }
        let node = project
            .wbs
            .delete(self.id)
            .map_err(|error| error.to_string())?;
        Ok(Box::new(RestoreWbsNode { node }))
    }
}

/// Restore a deleted WBS node snapshot, used by undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreWbsNode {
    /// Snapshot of the deleted node.
    pub node: WbsNode,
}

impl Command for RestoreWbsNode {
    fn description(&self) -> String {
        format!("restore WBS node \"{}\"", self.node.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = self.node.id;
        project
            .wbs
            .restore(self.node.clone())
            .map_err(|error| error.to_string())?;
        Ok(Box::new(DeleteWbsNode { id }))
    }
}

/// Rename a WBS node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameWbsNode {
    /// Node to rename.
    pub id: WbsId,
    /// New name.
    pub name: String,
}

impl Command for RenameWbsNode {
    fn description(&self) -> String {
        format!("rename WBS node {} to \"{}\"", self.id, self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let previous = project
            .wbs
            .rename(self.id, self.name.clone())
            .map_err(|error| error.to_string())?;
        Ok(Box::new(RenameWbsNode {
            id: self.id,
            name: previous,
        }))
    }
}

/// Move a WBS node (and its subtree) under a new parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveWbsNode {
    /// Node to move.
    pub id: WbsId,
    /// Requested parent.
    pub parent: WbsId,
}

impl Command for MoveWbsNode {
    fn description(&self) -> String {
        format!("move WBS node {} under {}", self.id, self.parent)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let previous = project
            .wbs
            .move_node(self.id, self.parent)
            .map_err(|error| error.to_string())?;
        let previous = previous.ok_or_else(|| "the WBS root cannot be moved".to_string())?;
        Ok(Box::new(MoveWbsNode {
            id: self.id,
            parent: previous,
        }))
    }
}

/// Assign an activity to a WBS node (or clear the assignment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignActivityWbs {
    /// Activity to update.
    pub activity: ActivityId,
    /// New WBS node, or `None` to clear.
    pub wbs: Option<WbsId>,
}

impl Command for AssignActivityWbs {
    fn description(&self) -> String {
        match self.wbs {
            Some(wbs) => format!("assign activity {} to WBS node {}", self.activity, wbs),
            None => format!("clear WBS assignment for activity {}", self.activity),
        }
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if let Some(wbs) = self.wbs {
            if project.wbs.node(wbs).is_none() {
                return Err(format!("WBS node {wbs} does not exist"));
            }
        }
        let activity = project
            .activity_mut(self.activity)
            .ok_or_else(|| format!("activity {} does not exist", self.activity))?;
        let previous = std::mem::replace(&mut activity.wbs, self.wbs);
        Ok(Box::new(AssignActivityWbs {
            activity: self.activity,
            wbs: previous,
        }))
    }
}

/// Create a named calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCalendar {
    /// Identifier to assign.
    pub id: CalendarId,
    /// Calendar name.
    pub name: String,
    /// Working-time definition.
    pub calendar: Calendar,
}

impl CreateCalendar {
    /// Construct a create-calendar command.
    pub fn new(id: CalendarId, name: impl Into<String>, calendar: Calendar) -> Self {
        Self {
            id,
            name: name.into(),
            calendar,
        }
    }
}

impl Command for CreateCalendar {
    fn description(&self) -> String {
        format!("create calendar \"{}\"", self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        project
            .calendars
            .add(self.id, self.name.clone(), self.calendar.clone())
            .map_err(|error| error.to_string())?;
        Ok(Box::new(DeleteCalendar { id: self.id }))
    }
}

/// Delete a named calendar that no activity uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteCalendar {
    /// Identifier of the calendar to delete.
    pub id: CalendarId,
}

impl Command for DeleteCalendar {
    fn description(&self) -> String {
        format!("delete calendar {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if project
            .activities
            .iter()
            .any(|activity| activity.calendar == Some(self.id))
        {
            return Err(format!(
                "calendar {} is still selected by an activity",
                self.id
            ));
        }
        let position = project
            .calendars
            .calendars()
            .iter()
            .position(|entry| entry.id == self.id)
            .ok_or_else(|| format!("calendar {} does not exist", self.id))?;
        let calendar = project
            .calendars
            .remove(self.id)
            .map_err(|error| error.to_string())?;
        Ok(Box::new(RestoreCalendar { calendar, position }))
    }
}

/// Restore a deleted calendar snapshot, used by undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreCalendar {
    /// Snapshot of the deleted calendar.
    pub calendar: NamedCalendar,
    /// Original position in the calendar list.
    pub position: usize,
}

impl Command for RestoreCalendar {
    fn description(&self) -> String {
        format!("restore calendar \"{}\"", self.calendar.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let id = self.calendar.id;
        project
            .calendars
            .restore(self.calendar.clone(), self.position)
            .map_err(|error| error.to_string())?;
        Ok(Box::new(DeleteCalendar { id }))
    }
}

/// Rename a named calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameCalendar {
    /// Calendar to rename.
    pub id: CalendarId,
    /// New name.
    pub name: String,
}

impl Command for RenameCalendar {
    fn description(&self) -> String {
        format!("rename calendar {} to \"{}\"", self.id, self.name)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let previous = project
            .calendars
            .rename(self.id, self.name.clone())
            .map_err(|error| error.to_string())?;
        Ok(Box::new(RenameCalendar {
            id: self.id,
            name: previous,
        }))
    }
}

/// Select the calendar an activity uses (or fall back to the project default).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActivityCalendar {
    /// Activity to update.
    pub activity: ActivityId,
    /// Named calendar, or `None` for the project default.
    pub calendar: Option<CalendarId>,
}

/// Replace a named calendar's working-time definition.
///
/// Activities referencing the calendar keep their reference — only the
/// definition (working weekdays and holidays) changes. The identifier and name
/// are untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCalendarDefinition {
    /// Calendar to update.
    pub id: CalendarId,
    /// New working-time definition.
    pub calendar: Calendar,
}

impl Command for SetCalendarDefinition {
    fn description(&self) -> String {
        format!("redefine calendar {}", self.id)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let position = project
            .calendars
            .calendars()
            .iter()
            .position(|entry| entry.id == self.id)
            .ok_or_else(|| format!("calendar {} does not exist", self.id))?;
        let previous = project
            .calendars
            .remove(self.id)
            .map_err(|error| error.to_string())?;
        if let Err(error) =
            project
                .calendars
                .add(self.id, previous.name.clone(), self.calendar.clone())
        {
            // Put the old definition back at its position so a failed command
            // changes nothing.
            project
                .calendars
                .restore(previous, position)
                .map_err(|rollback| format!("{error}; rollback also failed: {rollback}"))?;
            return Err(error.to_string());
        }
        Ok(Box::new(SetCalendarDefinition {
            id: self.id,
            calendar: previous.calendar,
        }))
    }
}

impl Command for SetActivityCalendar {
    fn description(&self) -> String {
        match self.calendar {
            Some(calendar) => {
                format!("set calendar of activity {} to {}", self.activity, calendar)
            }
            None => format!("reset calendar of activity {} to default", self.activity),
        }
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        if let Some(calendar) = self.calendar {
            if project.calendars.named(calendar).is_none() {
                return Err(format!("calendar {calendar} does not exist"));
            }
        }
        let activity = project
            .activity_mut(self.activity)
            .ok_or_else(|| format!("activity {} does not exist", self.activity))?;
        let previous = std::mem::replace(&mut activity.calendar, self.calendar);
        Ok(Box::new(SetActivityCalendar {
            activity: self.activity,
            calendar: previous,
        }))
    }
}

/// Change a relationship's lag in working days.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRelationshipLag {
    /// Relationship to update.
    pub id: RelationshipId,
    /// New lag in working days; negative values are leads.
    pub lag: i64,
}

impl Command for SetRelationshipLag {
    fn description(&self) -> String {
        format!("set lag of relationship {} to {}", self.id, self.lag)
    }

    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String> {
        let Some(relationship) = project.relationship_mut(self.id) else {
            return Err(format!("relationship {} does not exist", self.id));
        };
        let previous = relationship.lag;
        relationship.lag = self.lag;
        for activity in &mut project.activities {
            activity.clear_schedule();
        }
        Ok(Box::new(SetRelationshipLag {
            id: self.id,
            lag: previous,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Date, Month};

    fn jan(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::January, day).expect("valid test date")
    }

    fn project() -> Project {
        Project::new("Command test", jan(5))
    }

    fn execute(stack: &mut CommandStack, project: &mut Project, command: Box<dyn Command>) {
        stack
            .execute(project, command)
            .expect("command should succeed");
    }

    fn add_activity(
        stack: &mut CommandStack,
        project: &mut Project,
        id: u64,
        name: &str,
        duration: u32,
    ) -> ActivityId {
        let activity_id = ActivityId(id);
        execute(
            stack,
            project,
            Box::new(CreateActivity::new(activity_id, name, duration)),
        );
        activity_id
    }

    fn add_relationship(
        stack: &mut CommandStack,
        project: &mut Project,
        id: u64,
        predecessor: ActivityId,
        successor: ActivityId,
        link_type: LinkType,
        lag: i64,
    ) -> RelationshipId {
        let relationship_id = RelationshipId(id);
        execute(
            stack,
            project,
            Box::new(CreateRelationship::new(
                relationship_id,
                predecessor,
                successor,
                link_type,
                lag,
            )),
        );
        relationship_id
    }

    #[test]
    fn create_activity_undo_and_redo() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 2);

        assert!(project.activity(a).is_some());
        stack.undo(&mut project).expect("undo create");
        assert!(project.activity(a).is_none());

        stack.redo(&mut project).expect("redo create");
        assert!(project.activity(a).is_some());
        assert_eq!(
            project
                .activity(a)
                .map(|activity| activity.original_duration),
            Some(2)
        );
    }

    #[test]
    fn progress_budget_cost_and_actual_dates_undo_and_redo() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 3);
        let pristine = project.clone();

        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityBudget {
                id: a,
                budget_at_completion: Some(150_000),
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityProgress {
                id: a,
                percent_complete: Some(5000),
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityActualCost {
                id: a,
                actual_cost: Some(75_050),
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityActualDates {
                id: a,
                actual_start: Some(jan(5)),
                actual_finish: Some(jan(6)),
            }),
        );

        let activity = project.activity(a).expect("activity");
        assert_eq!(activity.budget_at_completion, Some(150_000));
        assert_eq!(activity.percent_complete, Some(5000));
        assert_eq!(activity.actual_cost, Some(75_050));
        assert_eq!(activity.actual_start, Some(jan(5)));
        assert_eq!(activity.actual_finish, Some(jan(6)));
        let updated = project.clone();
        assert_ne!(pristine, updated);

        for _ in 0..4 {
            stack.undo(&mut project).expect("undo a progress edit");
        }
        assert_eq!(project, pristine);

        for _ in 0..4 {
            stack.redo(&mut project).expect("redo a progress edit");
        }
        assert_eq!(project, updated);
    }

    #[test]
    fn progress_outside_basis_point_range_is_rejected() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 3);

        let error = stack
            .execute(
                &mut project,
                Box::new(SetActivityProgress {
                    id: a,
                    percent_complete: Some(12_345),
                }),
            )
            .expect_err("out-of-range progress must be rejected");
        assert!(error.contains("basis points"));
        // A rejected command must not change the activity.
        assert_eq!(project.activity(a).unwrap().percent_complete, None);
    }

    #[test]
    fn actual_dates_finish_before_start_is_rejected() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 3);

        let error = stack
            .execute(
                &mut project,
                Box::new(SetActivityActualDates {
                    id: a,
                    actual_start: Some(jan(6)),
                    actual_finish: Some(jan(5)),
                }),
            )
            .expect_err("finish before start must be rejected");
        assert!(error.contains("before"));
        assert_eq!(project.activity(a).unwrap().actual_start, None);
        assert_eq!(project.activity(a).unwrap().actual_finish, None);
    }

    #[test]
    fn undo_restores_prior_state_after_an_edit() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 2);
        let b = add_activity(&mut stack, &mut project, 2, "B", 3);
        add_relationship(&mut stack, &mut project, 1, a, b, LinkType::Fs, 1);

        let snapshot = project.clone();
        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityDuration { id: a, duration: 7 }),
        );
        assert_eq!(
            project
                .activity(a)
                .map(|activity| activity.original_duration),
            Some(7)
        );

        stack.undo(&mut project).expect("undo duration change");
        assert_eq!(project, snapshot);

        stack.redo(&mut project).expect("redo duration change");
        assert_eq!(
            project
                .activity(a)
                .map(|activity| activity.original_duration),
            Some(7)
        );
    }

    #[test]
    fn constraint_change_is_undoable_and_clears_cached_schedule() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 2);
        let constraint = Constraint::StartNoEarlierThan(jan(7));

        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityConstraint { id: a, constraint }),
        );

        let activity = project.activity(a).expect("activity exists");
        assert_eq!(activity.constraint, constraint);
        assert_eq!(activity.early_start, None);

        stack.undo(&mut project).expect("undo constraint change");
        assert_eq!(
            project.activity(a).expect("activity exists").constraint,
            Constraint::AsSoonAsPossible
        );

        stack.redo(&mut project).expect("redo constraint change");
        assert_eq!(
            project.activity(a).expect("activity exists").constraint,
            constraint
        );
    }

    #[test]
    fn deleting_activity_undo_restores_its_relationships() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 2);
        let b = add_activity(&mut stack, &mut project, 2, "B", 3);
        add_relationship(&mut stack, &mut project, 1, a, b, LinkType::Fs, 0);
        let snapshot = project.clone();

        execute(&mut stack, &mut project, Box::new(DeleteActivity { id: b }));
        assert!(project.activity(b).is_none());
        assert!(project.relationships.is_empty());

        stack.undo(&mut project).expect("undo delete");
        assert_eq!(project, snapshot);
    }

    #[test]
    fn deleting_activity_restores_relationship_order() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 1);
        let b = add_activity(&mut stack, &mut project, 2, "B", 1);
        let c = add_activity(&mut stack, &mut project, 3, "C", 1);
        let d = add_activity(&mut stack, &mut project, 4, "D", 1);
        add_relationship(&mut stack, &mut project, 1, a, b, LinkType::Fs, 0);
        add_relationship(&mut stack, &mut project, 2, b, c, LinkType::Ss, 0);
        add_relationship(&mut stack, &mut project, 3, a, c, LinkType::Ff, 0);
        add_relationship(&mut stack, &mut project, 4, c, d, LinkType::Fs, 0);
        let snapshot = project.clone();

        execute(&mut stack, &mut project, Box::new(DeleteActivity { id: b }));

        assert_eq!(project.relationships.len(), 2);
        stack.undo(&mut project).expect("undo delete");
        assert_eq!(project, snapshot);
    }

    #[test]
    fn deleting_relationship_undo_restores_it() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 2);
        let b = add_activity(&mut stack, &mut project, 2, "B", 3);
        let relationship = add_relationship(&mut stack, &mut project, 1, a, b, LinkType::Ss, 2);
        let snapshot = project.clone();

        execute(
            &mut stack,
            &mut project,
            Box::new(DeleteRelationship { id: relationship }),
        );
        assert!(project.relationship(relationship).is_none());

        stack.undo(&mut project).expect("undo relationship delete");
        assert_eq!(project, snapshot);
    }

    #[test]
    fn rename_activity_round_trips_through_undo() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "Old", 2);

        execute(
            &mut stack,
            &mut project,
            Box::new(RenameActivity {
                id: a,
                name: "New".to_string(),
            }),
        );
        assert_eq!(
            project.activity(a).map(|activity| activity.name.as_str()),
            Some("New")
        );

        stack.undo(&mut project).expect("undo rename");
        assert_eq!(
            project.activity(a).map(|activity| activity.name.as_str()),
            Some("Old")
        );
    }

    #[test]
    fn failed_command_leaves_project_unchanged() {
        let mut project = project();
        let mut stack = CommandStack::new();
        add_activity(&mut stack, &mut project, 1, "A", 2);
        let snapshot = project.clone();

        let error = stack.execute(
            &mut project,
            Box::new(CreateActivity::new(ActivityId(1), "Duplicate", 1)),
        );

        assert!(error.is_err());
        assert_eq!(project, snapshot);
        assert!(stack.can_undo());
    }

    #[test]
    fn owner_and_promise_are_recorded_and_reversible() {
        let mut project = project();
        project
            .activities
            .push(Activity::new(ActivityId::new(1), "A", 3));
        let id = project.activities[0].id;

        let mut command = SetActivityOwner {
            id,
            owner: Some("Crew 3".to_string()),
        };
        let mut reverse = command.apply(&mut project).expect("owner applies");
        assert_eq!(project.activities[0].owner, Some("Crew 3".to_string()));
        reverse.apply(&mut project).expect("owner undoes");
        assert_eq!(project.activities[0].owner, None);

        let mut command = SetActivityPromise {
            id,
            promised: true,
            delay_reason: Some("waiting on a permit".to_string()),
        };
        let mut reverse = command.apply(&mut project).expect("promise applies");
        assert!(project.activities[0].promised);
        assert_eq!(
            project.activities[0].delay_reason,
            Some("waiting on a permit".to_string())
        );
        reverse.apply(&mut project).expect("promise undoes");
        assert!(!project.activities[0].promised);
    }

    #[test]
    fn an_empty_owner_or_delay_reason_is_rejected() {
        let mut project = project();
        project
            .activities
            .push(Activity::new(ActivityId::new(1), "A", 3));
        let id = project.activities[0].id;

        let mut command = SetActivityOwner {
            id,
            owner: Some("   ".to_string()),
        };
        assert!(command.apply(&mut project).is_err());

        let mut command = SetActivityPromise {
            id,
            promised: true,
            delay_reason: Some("  ".to_string()),
        };
        assert!(command.apply(&mut project).is_err());
    }

    #[test]
    fn undo_and_redo_on_empty_stacks_return_errors() {
        let mut project = project();
        let mut stack = CommandStack::new();

        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert!(stack.undo(&mut project).is_err());
        assert!(stack.redo(&mut project).is_err());
    }

    #[test]
    fn undo_descriptions_are_in_order() {
        let mut project = project();
        let mut stack = CommandStack::new();
        add_activity(&mut stack, &mut project, 1, "A", 2);
        add_activity(&mut stack, &mut project, 2, "B", 3);

        assert_eq!(
            stack.undo_descriptions(),
            vec![
                "create activity \"A\"".to_string(),
                "create activity \"B\"".to_string(),
            ]
        );
    }

    fn six_day_week() -> Calendar {
        Calendar::new([true, true, true, true, true, true, false]).expect("valid six-day week")
    }

    #[test]
    fn wbs_create_rename_move_and_undo() {
        let mut project = project();
        let root = project.wbs.root();
        let mut stack = CommandStack::new();

        execute(
            &mut stack,
            &mut project,
            Box::new(CreateWbsNode::new(WbsId(2), "Design", root)),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(RenameWbsNode {
                id: WbsId(2),
                name: "Engineering".to_string(),
            }),
        );
        assert_eq!(
            project.wbs.node(WbsId(2)).expect("node").name,
            "Engineering"
        );

        stack.undo(&mut project).expect("undo rename");
        assert_eq!(project.wbs.node(WbsId(2)).expect("node").name, "Design");

        execute(
            &mut stack,
            &mut project,
            Box::new(CreateWbsNode::new(WbsId(3), "Details", root)),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(MoveWbsNode {
                id: WbsId(3),
                parent: WbsId(2),
            }),
        );
        assert_eq!(
            project.wbs.node(WbsId(3)).expect("node").parent,
            Some(WbsId(2))
        );

        stack.undo(&mut project).expect("undo move");
        assert_eq!(project.wbs.node(WbsId(3)).expect("node").parent, Some(root));
    }

    #[test]
    fn wbs_delete_is_rejected_while_activities_are_assigned() {
        let mut project = project();
        let root = project.wbs.root();
        let mut stack = CommandStack::new();

        execute(
            &mut stack,
            &mut project,
            Box::new(CreateWbsNode::new(WbsId(2), "Design", root)),
        );
        add_activity(&mut stack, &mut project, 1, "A", 1);
        execute(
            &mut stack,
            &mut project,
            Box::new(AssignActivityWbs {
                activity: ActivityId(1),
                wbs: Some(WbsId(2)),
            }),
        );
        assert_eq!(
            project.activity(ActivityId(1)).expect("activity").wbs,
            Some(WbsId(2))
        );

        let error = stack
            .execute(&mut project, Box::new(DeleteWbsNode { id: WbsId(2) }))
            .expect_err("delete must be rejected");
        assert!(error.contains("still has activities"));
        assert!(project.wbs.node(WbsId(2)).is_some());

        execute(
            &mut stack,
            &mut project,
            Box::new(AssignActivityWbs {
                activity: ActivityId(1),
                wbs: None,
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(DeleteWbsNode { id: WbsId(2) }),
        );
        assert!(project.wbs.node(WbsId(2)).is_none());

        stack.undo(&mut project).expect("undo delete");
        assert!(project.wbs.node(WbsId(2)).is_some());
    }

    #[test]
    fn calendar_definition_changes_are_undoable_and_revert_on_failure() {
        let mut project = project();
        let mut stack = CommandStack::new();
        let a = add_activity(&mut stack, &mut project, 1, "A", 2);

        let week = [true, true, true, true, true, false, false];
        let calendar = Calendar::new(week)
            .expect("valid week")
            .with_holiday(jan(25));
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateCalendar::new(CalendarId(1), "Six day", calendar)),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityCalendar {
                activity: a,
                calendar: Some(CalendarId(1)),
            }),
        );

        // Redefine: Saturday becomes a working day, the holiday moves.
        let redefined = Calendar::new([true, true, true, true, true, true, false])
            .expect("valid week")
            .with_holiday(jan(26));
        execute(
            &mut stack,
            &mut project,
            Box::new(SetCalendarDefinition {
                id: CalendarId(1),
                calendar: redefined,
            }),
        );

        // The activity keeps its reference; the definition changed under it.
        let named = project.calendars.named(CalendarId(1)).expect("calendar");
        assert!(named.calendar.working_weekdays()[5]);
        assert!(named.calendar.holidays().contains(&jan(26)));
        assert_eq!(
            project.activity(a).expect("activity").calendar,
            Some(CalendarId(1))
        );

        // Undo puts the original definition back, reference still intact.
        stack.undo(&mut project).expect("undo redefine");
        let named = project.calendars.named(CalendarId(1)).expect("calendar");
        assert!(!named.calendar.working_weekdays()[5]);
        assert!(named.calendar.holidays().contains(&jan(25)));
        assert_eq!(
            project.activity(a).expect("activity").calendar,
            Some(CalendarId(1))
        );
    }

    #[test]
    fn redefining_an_unknown_calendar_is_rejected() {
        let mut project = project();
        let mut stack = CommandStack::new();

        let error = stack
            .execute(
                &mut project,
                Box::new(SetCalendarDefinition {
                    id: CalendarId(9),
                    calendar: Calendar::weekend(),
                }),
            )
            .expect_err("unknown calendar must be rejected");
        assert!(error.contains("does not exist"));
    }

    #[test]
    fn calendar_commands_undo_assignment_and_creation() {
        let mut project = project();
        let mut stack = CommandStack::new();
        add_activity(&mut stack, &mut project, 1, "A", 1);

        execute(
            &mut stack,
            &mut project,
            Box::new(CreateCalendar::new(
                CalendarId(1),
                "Six day",
                six_day_week(),
            )),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(RenameCalendar {
                id: CalendarId(1),
                name: "Site".to_string(),
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityCalendar {
                activity: ActivityId(1),
                calendar: Some(CalendarId(1)),
            }),
        );
        assert_eq!(
            project.activity(ActivityId(1)).expect("activity").calendar,
            Some(CalendarId(1))
        );

        stack.undo(&mut project).expect("undo assignment");
        assert_eq!(
            project.activity(ActivityId(1)).expect("activity").calendar,
            None
        );
        stack.undo(&mut project).expect("undo rename");
        assert_eq!(
            project
                .calendars
                .named(CalendarId(1))
                .expect("calendar")
                .name,
            "Six day"
        );
        stack.undo(&mut project).expect("undo creation");
        assert!(project.calendars.is_empty());

        stack.redo(&mut project).expect("redo creation");
        assert_eq!(project.calendars.len(), 1);
    }

    #[test]
    fn calendar_delete_is_rejected_while_in_use_and_undo_restores_position() {
        let mut project = project();
        let mut stack = CommandStack::new();
        add_activity(&mut stack, &mut project, 1, "A", 1);
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateCalendar::new(CalendarId(1), "First", six_day_week())),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateCalendar::new(CalendarId(2), "Second", six_day_week())),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityCalendar {
                activity: ActivityId(1),
                calendar: Some(CalendarId(2)),
            }),
        );

        let error = stack
            .execute(&mut project, Box::new(DeleteCalendar { id: CalendarId(2) }))
            .expect_err("delete must be rejected");
        assert!(error.contains("still selected"));

        execute(
            &mut stack,
            &mut project,
            Box::new(SetActivityCalendar {
                activity: ActivityId(1),
                calendar: None,
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(DeleteCalendar { id: CalendarId(2) }),
        );
        assert_eq!(project.calendars.len(), 1);

        stack.undo(&mut project).expect("undo delete");
        assert_eq!(project.calendars.calendars()[1].id, CalendarId(2));
    }

    #[test]
    fn adding_and_removing_a_resource_assignment_is_reversible() {
        let mut stack = CommandStack::new();
        let mut project = project();
        let activity = add_activity(&mut stack, &mut project, 1, "Task", 2);

        execute(
            &mut stack,
            &mut project,
            Box::new(AddResourceToActivity {
                activity_id: activity,
                resource_id: ResourceId(7),
            }),
        );
        assert_eq!(project.activities[0].resources, vec![ResourceId(7)]);

        stack.undo(&mut project).expect("undo add");
        assert!(project.activities[0].resources.is_empty());

        stack.redo(&mut project).expect("redo add");
        assert_eq!(project.activities[0].resources, vec![ResourceId(7)]);

        execute(
            &mut stack,
            &mut project,
            Box::new(RemoveResourceFromActivity {
                activity_id: activity,
                resource_id: ResourceId(7),
            }),
        );
        assert!(project.activities[0].resources.is_empty());

        stack.undo(&mut project).expect("undo remove");
        assert_eq!(project.activities[0].resources, vec![ResourceId(7)]);
    }

    #[test]
    fn removing_a_resource_assignment_keeps_the_order_of_the_others() {
        let mut stack = CommandStack::new();
        let mut project = project();
        let activity = add_activity(&mut stack, &mut project, 1, "Task", 2);

        for resource in [7, 8, 9] {
            execute(
                &mut stack,
                &mut project,
                Box::new(AddResourceToActivity {
                    activity_id: activity,
                    resource_id: ResourceId(resource),
                }),
            );
        }
        assert_eq!(
            project.activities[0].resources,
            vec![ResourceId(7), ResourceId(8), ResourceId(9)]
        );

        execute(
            &mut stack,
            &mut project,
            Box::new(RemoveResourceFromActivity {
                activity_id: activity,
                resource_id: ResourceId(8),
            }),
        );
        assert_eq!(
            project.activities[0].resources,
            vec![ResourceId(7), ResourceId(9)]
        );

        stack.undo(&mut project).expect("undo remove");
        assert_eq!(
            project.activities[0].resources,
            vec![ResourceId(7), ResourceId(8), ResourceId(9)]
        );
    }

    #[test]
    fn removing_a_resource_that_is_not_assigned_is_rejected() {
        let mut stack = CommandStack::new();
        let mut project = project();
        let activity = add_activity(&mut stack, &mut project, 1, "Task", 2);

        let error = stack
            .execute(
                &mut project,
                Box::new(RemoveResourceFromActivity {
                    activity_id: activity,
                    resource_id: ResourceId(7),
                }),
            )
            .expect_err("missing assignment must be rejected");
        assert!(error.contains("does not have resource"), "{error}");
    }

    #[test]
    fn adding_a_resource_twice_is_rejected() {
        let mut stack = CommandStack::new();
        let mut project = project();
        let activity = add_activity(&mut stack, &mut project, 1, "Task", 2);

        execute(
            &mut stack,
            &mut project,
            Box::new(AddResourceToActivity {
                activity_id: activity,
                resource_id: ResourceId(7),
            }),
        );

        let error = stack
            .execute(
                &mut project,
                Box::new(AddResourceToActivity {
                    activity_id: activity,
                    resource_id: ResourceId(7),
                }),
            )
            .expect_err("duplicate assignment must be rejected");
        assert!(error.contains("already has resource"), "{error}");
        assert_eq!(project.activities[0].resources, vec![ResourceId(7)]);
    }

    #[test]
    fn capturing_a_baseline_records_the_scheduled_dates() {
        let mut stack = CommandStack::new();
        let mut project = project();
        add_activity(&mut stack, &mut project, 1, "Task", 3);

        execute(
            &mut stack,
            &mut project,
            Box::new(CaptureBaseline {
                name: "Baseline 1".to_string(),
                created_at: jan(5),
            }),
        );

        assert_eq!(project.baselines.len(), 1);
        let baseline = &project.baselines[0];
        assert_eq!(baseline.id, BaselineId(1));
        assert_eq!(baseline.name, "Baseline 1");
        assert_eq!(baseline.created_at, jan(5));
        assert_eq!(baseline.snapshots.len(), 1);
        let snapshot = &baseline.snapshots[0];
        assert_eq!(snapshot.activity_id, ActivityId(1));
        assert_eq!(snapshot.baseline_duration, 3);
        assert_eq!(snapshot.baseline_start, Some(jan(5)));
        assert_eq!(snapshot.baseline_finish, Some(jan(7)));
    }

    #[test]
    fn capturing_a_baseline_is_reversible() {
        let mut stack = CommandStack::new();
        let mut project = project();
        add_activity(&mut stack, &mut project, 1, "Task", 3);

        execute(
            &mut stack,
            &mut project,
            Box::new(CaptureBaseline {
                name: "Baseline 1".to_string(),
                created_at: jan(5),
            }),
        );
        let captured = project.baselines[0].clone();

        stack.undo(&mut project).expect("undo capture");
        assert!(project.baselines.is_empty());

        stack.redo(&mut project).expect("redo capture");
        assert_eq!(project.baselines, vec![captured.clone()]);

        stack.undo(&mut project).expect("undo capture again");
        assert!(project.baselines.is_empty());

        stack.redo(&mut project).expect("redo capture again");
        assert_eq!(project.baselines, vec![captured]);
    }

    #[test]
    fn removing_a_baseline_is_reversible_and_keeps_its_position() {
        let mut stack = CommandStack::new();
        let mut project = project();
        add_activity(&mut stack, &mut project, 1, "Task", 3);

        execute(
            &mut stack,
            &mut project,
            Box::new(CaptureBaseline {
                name: "First".to_string(),
                created_at: jan(5),
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(CaptureBaseline {
                name: "Second".to_string(),
                created_at: jan(6),
            }),
        );
        assert_eq!(project.baselines.len(), 2);

        execute(
            &mut stack,
            &mut project,
            Box::new(RemoveBaseline { id: BaselineId(1) }),
        );
        assert_eq!(project.baselines.len(), 1);
        assert_eq!(project.baselines[0].name, "Second");

        stack.undo(&mut project).expect("undo remove");
        assert_eq!(project.baselines.len(), 2);
        assert_eq!(project.baselines[0].name, "First");
        assert_eq!(project.baselines[1].name, "Second");
    }

    #[test]
    fn removing_an_unknown_baseline_is_rejected() {
        let mut stack = CommandStack::new();
        let mut project = project();

        let error = stack
            .execute(&mut project, Box::new(RemoveBaseline { id: BaselineId(9) }))
            .expect_err("missing baseline must be rejected");
        assert!(error.contains("does not exist"), "{error}");
    }

    #[test]
    fn creating_a_resource_is_reversible() {
        let mut stack = CommandStack::new();
        let mut project = project();

        execute(
            &mut stack,
            &mut project,
            Box::new(CreateResource {
                id: ResourceId(1),
                name: "Crane".to_string(),
                kind: ResourceKind::Equipment,
                capacity: 2,
            }),
        );
        assert_eq!(project.resources.len(), 1);
        assert_eq!(project.resources[0].name, "Crane");
        assert_eq!(project.resources[0].kind, ResourceKind::Equipment);
        assert_eq!(project.resources[0].capacity, 2);

        stack.undo(&mut project).expect("undo create");
        assert!(project.resources.is_empty());

        stack.redo(&mut project).expect("redo create");
        assert_eq!(project.resources.len(), 1);
    }

    #[test]
    fn creating_a_resource_with_a_duplicate_name_is_rejected() {
        let mut stack = CommandStack::new();
        let mut project = project();
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateResource {
                id: ResourceId(1),
                name: "Crane".to_string(),
                kind: ResourceKind::Equipment,
                capacity: 1,
            }),
        );

        let error = stack
            .execute(
                &mut project,
                Box::new(CreateResource {
                    id: ResourceId(2),
                    name: "crane".to_string(),
                    kind: ResourceKind::Equipment,
                    capacity: 1,
                }),
            )
            .expect_err("duplicate name must be rejected");
        assert!(error.contains("already exists"), "{error}");
        assert_eq!(project.resources.len(), 1);
    }

    #[test]
    fn creating_a_resource_without_a_name_is_rejected() {
        let mut stack = CommandStack::new();
        let mut project = project();

        let error = stack
            .execute(
                &mut project,
                Box::new(CreateResource {
                    id: ResourceId(1),
                    name: "   ".to_string(),
                    kind: ResourceKind::Labour,
                    capacity: 1,
                }),
            )
            .expect_err("empty name must be rejected");
        assert!(error.contains("must not be empty"), "{error}");
    }

    #[test]
    fn deleting_a_resource_detaches_it_from_activities_and_is_reversible() {
        let mut stack = CommandStack::new();
        let mut project = project();
        let activity = add_activity(&mut stack, &mut project, 1, "Task", 2);
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateResource {
                id: ResourceId(1),
                name: "Crane".to_string(),
                kind: ResourceKind::Equipment,
                capacity: 1,
            }),
        );
        execute(
            &mut stack,
            &mut project,
            Box::new(AddResourceToActivity {
                activity_id: activity,
                resource_id: ResourceId(1),
            }),
        );
        assert_eq!(project.activities[0].resources, vec![ResourceId(1)]);

        execute(
            &mut stack,
            &mut project,
            Box::new(DeleteResource { id: ResourceId(1) }),
        );
        assert!(project.resources.is_empty());
        assert!(project.activities[0].resources.is_empty());

        stack.undo(&mut project).expect("undo delete");
        assert_eq!(project.resources.len(), 1);
        assert_eq!(project.resources[0].name, "Crane");
        assert_eq!(project.activities[0].resources, vec![ResourceId(1)]);
    }

    #[test]
    fn renaming_a_resource_is_reversible() {
        let mut stack = CommandStack::new();
        let mut project = project();
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateResource {
                id: ResourceId(1),
                name: "Crane".to_string(),
                kind: ResourceKind::Equipment,
                capacity: 1,
            }),
        );

        execute(
            &mut stack,
            &mut project,
            Box::new(RenameResource {
                id: ResourceId(1),
                name: "Tower crane".to_string(),
            }),
        );
        assert_eq!(project.resources[0].name, "Tower crane");

        stack.undo(&mut project).expect("undo rename");
        assert_eq!(project.resources[0].name, "Crane");
    }

    #[test]
    fn changing_resource_capacity_and_kind_is_reversible() {
        let mut stack = CommandStack::new();
        let mut project = project();
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateResource {
                id: ResourceId(1),
                name: "Crane".to_string(),
                kind: ResourceKind::Labour,
                capacity: 1,
            }),
        );

        execute(
            &mut stack,
            &mut project,
            Box::new(SetResourceCapacity {
                id: ResourceId(1),
                capacity: 4,
            }),
        );
        assert_eq!(project.resources[0].capacity, 4);

        execute(
            &mut stack,
            &mut project,
            Box::new(SetResourceKind {
                id: ResourceId(1),
                kind: ResourceKind::Equipment,
            }),
        );
        assert_eq!(project.resources[0].kind, ResourceKind::Equipment);

        stack.undo(&mut project).expect("undo kind");
        assert_eq!(project.resources[0].kind, ResourceKind::Labour);

        stack.undo(&mut project).expect("undo capacity");
        assert_eq!(project.resources[0].capacity, 1);
    }

    #[test]
    fn a_resource_capacity_below_one_is_raised() {
        let mut stack = CommandStack::new();
        let mut project = project();
        execute(
            &mut stack,
            &mut project,
            Box::new(CreateResource {
                id: ResourceId(1),
                name: "Crane".to_string(),
                kind: ResourceKind::Equipment,
                capacity: 0,
            }),
        );
        assert_eq!(project.resources[0].capacity, 1);
    }
}
