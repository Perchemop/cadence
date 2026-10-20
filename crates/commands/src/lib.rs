//! Typed command layer: the single mutation path for a Cadence project.
//!
//! Every mutation of project state is expressed as a [`Command`], applied
//! through a [`CommandStack`] that supports undo and redo. This keeps the
//! engine free of UI concerns and makes every mutation testable and (later)
//! scriptable.

use cadence_core::model::Project;

/// A reversible mutation of a [`Project`].
pub trait Command {
    /// Human-readable description of what this command does.
    fn description(&self) -> String;

    /// Apply the command, returning the inverse command for undo.
    fn apply(&mut self, project: &mut Project) -> Result<Box<dyn Command>, String>;
}

/// Undo/redo stack over a [`Project`].
pub struct CommandStack {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    /// Maximum undo depth; `None` means unlimited.
    limit: Option<usize>,
}

impl CommandStack {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            limit: Some(200),
        }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            limit: Some(limit),
        }
    }

    /// Apply a command, clearing the redo stack.
    pub fn execute(
        &mut self,
        project: &mut Project,
        mut command: Box<dyn Command>,
    ) -> Result<(), String> {
        // Applying produces the inverse command; the inverse reaches the redo
        // stack only when this command is undone.
        let _inverse = command.apply(project)?;
        self.undo_stack.push(command);
        if let Some(limit) = self.limit {
            if self.undo_stack.len() > limit {
                self.undo_stack.remove(0);
            }
        }
        self.redo_stack.clear();
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self, project: &mut Project) -> Result<String, String> {
        let mut command = self.undo_stack.pop().expect("undo called with empty stack");
        let description = command.description();
        let inverse = command.apply(project)?;
        self.redo_stack.push(inverse);
        Ok(description)
    }

    pub fn redo(&mut self, project: &mut Project) -> Result<String, String> {
        let mut command = self.redo_stack.pop().expect("redo called with empty stack");
        let description = command.description();
        let inverse = command.apply(project)?;
        self.undo_stack.push(inverse);
        Ok(description)
    }

    pub fn undo_descriptions(&self) -> Vec<String> {
        self.undo_stack.iter().map(|c| c.description()).collect()
    }
}

impl Default for CommandStack {
    fn default() -> Self {
        Self::new()
    }
}
