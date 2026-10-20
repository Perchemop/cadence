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

use cadence_core::model::{Activity, ActivityId, LinkType, Project, Relationship, RelationshipId};

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
}
