//! SQLite project files for Cadence.
//!
//! A Cadence project file is an ordinary SQLite database. The schema is
//! versioned with `PRAGMA user_version`; this build reads and writes version 9
//! and migrates version 1 and later files in place.
//!
//! Saving runs in a single transaction that replaces the previous contents, so
//! a failed save never corrupts an existing project.
//!
//! Cached schedule fields on activities are deliberately not stored: schedules
//! are derived data and are recomputed from the project graph on load.

use std::collections::BTreeSet;
use std::path::Path;

use cadence_core::calendar_set::{CalendarId, CalendarSet};
use cadence_core::model::{
    Activity, ActivityBaselineSnapshot, ActivityId, Baseline, BaselineId, Calendar, Constraint,
    DailyNote, DailyNoteId, LinkType, LookaheadSnapshot, LookaheadSnapshotEntry,
    LookaheadSnapshotId, ProductionEntry, ProductionEntryId, Project, Relationship,
    RelationshipId, Resource, ResourceId, ResourceKind,
};
use cadence_core::wbs::{WbsId, WbsTree};
use rusqlite::{params, Connection};
use thiserror::Error;
use time::{Date, Month};

/// Project-file schema version written by this build.
pub const SCHEMA_VERSION: i64 = 9;

const SCHEMA: &str = r#"
CREATE TABLE project (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    name        TEXT    NOT NULL,
    start_date  TEXT    NOT NULL,
    workweek    INTEGER NOT NULL
);

CREATE TABLE project_holidays (
    date TEXT PRIMARY KEY
);

CREATE TABLE calendars (
    id       INTEGER PRIMARY KEY,
    name     TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    workweek INTEGER NOT NULL,
    position INTEGER NOT NULL
);

CREATE TABLE calendar_holidays (
    calendar_id INTEGER NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    date        TEXT    NOT NULL,
    PRIMARY KEY (calendar_id, date)
);

CREATE TABLE wbs_nodes (
    id       INTEGER PRIMARY KEY,
    name     TEXT    NOT NULL,
    parent_id INTEGER REFERENCES wbs_nodes(id),
    preorder INTEGER NOT NULL UNIQUE
);

CREATE TABLE activities (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    duration   INTEGER NOT NULL,
    wbs_id     INTEGER REFERENCES wbs_nodes(id),
    calendar_id INTEGER REFERENCES calendars(id),
    constraint_kind TEXT NOT NULL DEFAULT 'asap',
    constraint_date TEXT,
    budget_at_completion INTEGER,
    actual_cost INTEGER,
    percent_complete INTEGER,
    actual_start TEXT,
    actual_finish TEXT,
    owner TEXT,
    promised INTEGER NOT NULL DEFAULT 0,
    delay_reason TEXT,
    location TEXT,
    position   INTEGER NOT NULL
);

CREATE TABLE relationships (
    id          INTEGER PRIMARY KEY,
    predecessor INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    successor   INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    link_type   TEXT    NOT NULL CHECK (link_type IN ('FS', 'SS', 'FF', 'SF')),
    lag         INTEGER NOT NULL,
    position    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS resources (
    id      INTEGER PRIMARY KEY,
    name    TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    kind    TEXT    NOT NULL DEFAULT 'labour',
    capacity INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS activity_resources (
    activity_id   INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    resource_id   INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    PRIMARY KEY (activity_id, resource_id)
);

CREATE TABLE IF NOT EXISTS baselines (
    id          INTEGER PRIMARY KEY,
    name        TEXT    NOT NULL,
    created_at  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS baseline_snapshots (
    baseline_id     INTEGER NOT NULL REFERENCES baselines(id) ON DELETE CASCADE,
    activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    baseline_duration INTEGER NOT NULL,
    baseline_start  TEXT,
    baseline_finish TEXT,
    PRIMARY KEY (baseline_id, activity_id)
);

CREATE TABLE IF NOT EXISTS lookahead_snapshots (
    id          INTEGER PRIMARY KEY,
    name        TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    window_start TEXT   NOT NULL,
    window_weeks INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS lookahead_snapshot_entries (
    snapshot_id     INTEGER NOT NULL REFERENCES lookahead_snapshots(id) ON DELETE CASCADE,
    activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    scheduled_start TEXT    NOT NULL,
    scheduled_finish TEXT   NOT NULL,
    owner           TEXT,
    promised        INTEGER NOT NULL DEFAULT 0,
    percent_complete INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (snapshot_id, activity_id)
);

CREATE TABLE IF NOT EXISTS daily_notes (
    id                   INTEGER PRIMARY KEY,
    date                 TEXT    NOT NULL,
    content              TEXT    NOT NULL,
    activity_id          INTEGER REFERENCES activities(id) ON DELETE CASCADE,
    lookahead_window_start TEXT
);

CREATE TABLE IF NOT EXISTS production_entries (
    id              INTEGER PRIMARY KEY,
    activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    date            TEXT    NOT NULL,
    percent_complete INTEGER NOT NULL,
    UNIQUE (activity_id, date)
);
"#;

/// Errors produced by project-file operations.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// The requested project file does not exist.
    #[error("project file not found: {0}")]
    NotFound(String),
    /// The underlying database rejected an operation.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// The file was written by a newer (or unknown) schema version.
    #[error("unsupported project schema version {found} (this build supports {supported})")]
    UnsupportedVersion {
        /// Version found in the file.
        found: i64,
        /// Version supported by this build.
        supported: i64,
    },
    /// The file exists but its contents are not a valid Cadence project.
    #[error("invalid project data: {0}")]
    InvalidData(String),
}

/// Result alias for persistence operations.
pub type Result<T> = std::result::Result<T, PersistenceError>;

/// Save `project` to the SQLite file at `path`, replacing any existing content.
///
/// The file is created when it does not exist. The write is transactional: on
/// any error the previous contents of an existing file are preserved.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidData`] when the project contains dangling
/// references or out-of-range identifiers, and [`PersistenceError::Database`]
/// for SQLite failures.
pub fn save_project(project: &Project, path: impl AsRef<Path>) -> Result<()> {
    validate(project)?;
    let mut connection = Connection::open(path)?;
    connection.pragma_update(None, "foreign_keys", true)?;
    ensure_schema(&connection)?;

    let transaction = connection.transaction()?;
    clear_project(&transaction)?;
    write_project(&transaction, project)?;
    transaction.commit()?;
    Ok(())
}

/// Load the project stored at `path`.
///
/// # Errors
///
/// Returns [`PersistenceError::NotFound`] when the file is missing,
/// [`PersistenceError::UnsupportedVersion`] for newer schema versions, and
/// [`PersistenceError::InvalidData`] when the contents are malformed.
pub fn load_project(path: impl AsRef<Path>) -> Result<Project> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(PersistenceError::NotFound(path.display().to_string()));
    }
    let connection = Connection::open(path)?;
    connection.pragma_update(None, "foreign_keys", true)?;
    ensure_schema(&connection)?;
    read_project(&connection)
}

fn ensure_schema(connection: &Connection) -> Result<()> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            // Create the schema and stamp the version in one transaction so an
            // interrupted save can never leave a half-created file behind.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(SCHEMA)?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            transaction.commit()?;
            Ok(())
        }
        1 => {
            // Version 2 adds activity constraints. Existing activities keep
            // the historical behaviour (as soon as possible).
            // Version 3 adds the resource dictionary, the activity cost fields
            // used by earned value management, and baselines.
            // Version 4 adds the recorded actual dates. Migrate straight
            // through to the current schema version.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "ALTER TABLE activities ADD COLUMN constraint_kind TEXT NOT NULL DEFAULT 'asap';
                 ALTER TABLE activities ADD COLUMN constraint_date TEXT;
                 ALTER TABLE activities ADD COLUMN budget_at_completion INTEGER;
                 ALTER TABLE activities ADD COLUMN actual_cost INTEGER;
                 ALTER TABLE activities ADD COLUMN percent_complete INTEGER;
                 ALTER TABLE activities ADD COLUMN actual_start TEXT;
                 ALTER TABLE activities ADD COLUMN actual_finish TEXT;
                 PRAGMA user_version = 2;
                 CREATE TABLE IF NOT EXISTS resources (
                     id      INTEGER PRIMARY KEY,
                     name    TEXT    NOT NULL UNIQUE COLLATE NOCASE,
                     kind    TEXT    NOT NULL DEFAULT 'labour',
                     capacity INTEGER NOT NULL DEFAULT 1
                 );
                 CREATE TABLE IF NOT EXISTS activity_resources (
                     activity_id   INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     resource_id   INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
                     PRIMARY KEY (activity_id, resource_id)
                 );
                 CREATE TABLE IF NOT EXISTS baselines (
                     id          INTEGER PRIMARY KEY,
                     name        TEXT    NOT NULL,
                     created_at  TEXT    NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS baseline_snapshots (
                     baseline_id     INTEGER NOT NULL REFERENCES baselines(id) ON DELETE CASCADE,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     baseline_duration INTEGER NOT NULL,
                     baseline_start  TEXT,
                     baseline_finish TEXT,
                     PRIMARY KEY (baseline_id, activity_id)
                 );
                 ALTER TABLE activities ADD COLUMN owner TEXT;
                 ALTER TABLE activities ADD COLUMN promised INTEGER NOT NULL DEFAULT 0;
                 ALTER TABLE activities ADD COLUMN delay_reason TEXT;
                 CREATE TABLE IF NOT EXISTS lookahead_snapshots (
                     id          INTEGER PRIMARY KEY,
                     name        TEXT    NOT NULL,
                     created_at  TEXT    NOT NULL,
                     window_start TEXT   NOT NULL,
                     window_weeks INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS lookahead_snapshot_entries (
                     snapshot_id     INTEGER NOT NULL REFERENCES lookahead_snapshots(id) ON DELETE CASCADE,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     name            TEXT    NOT NULL,
                     scheduled_start TEXT    NOT NULL,
                     scheduled_finish TEXT   NOT NULL,
                     owner           TEXT,
                     promised        INTEGER NOT NULL DEFAULT 0,
                     percent_complete INTEGER NOT NULL DEFAULT 0,
                     PRIMARY KEY (snapshot_id, activity_id)
                 );
                 CREATE TABLE IF NOT EXISTS daily_notes (
                     id                   INTEGER PRIMARY KEY,
                     date                 TEXT    NOT NULL,
                     content              TEXT    NOT NULL,
                     activity_id          INTEGER REFERENCES activities(id) ON DELETE CASCADE,
                     lookahead_window_start TEXT
                 );
                 ALTER TABLE activities ADD COLUMN location TEXT;
                 CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        2 => {
            // Version 3 adds the resource dictionary and activity-resource
            // assignments, the activity cost fields used by earned value
            // management, and baselines.
            // Version 4 adds the recorded actual dates.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "ALTER TABLE activities ADD COLUMN budget_at_completion INTEGER;
                 ALTER TABLE activities ADD COLUMN actual_cost INTEGER;
                 ALTER TABLE activities ADD COLUMN percent_complete INTEGER;
                 ALTER TABLE activities ADD COLUMN actual_start TEXT;
                 ALTER TABLE activities ADD COLUMN actual_finish TEXT;
                 CREATE TABLE IF NOT EXISTS resources (
                    id      INTEGER PRIMARY KEY,
                    name    TEXT    NOT NULL UNIQUE COLLATE NOCASE,
                    kind    TEXT    NOT NULL DEFAULT 'labour',
                    capacity INTEGER NOT NULL DEFAULT 1
                );
                 CREATE TABLE IF NOT EXISTS activity_resources (
                    activity_id   INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                    resource_id   INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
                    PRIMARY KEY (activity_id, resource_id)
                );
                 CREATE TABLE IF NOT EXISTS baselines (
                     id          INTEGER PRIMARY KEY,
                     name        TEXT    NOT NULL,
                     created_at  TEXT    NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS baseline_snapshots (
                     baseline_id     INTEGER NOT NULL REFERENCES baselines(id) ON DELETE CASCADE,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     baseline_duration INTEGER NOT NULL,
                     baseline_start  TEXT,
                     baseline_finish TEXT,
                     PRIMARY KEY (baseline_id, activity_id)
                 );
                 ALTER TABLE activities ADD COLUMN owner TEXT;
                 ALTER TABLE activities ADD COLUMN promised INTEGER NOT NULL DEFAULT 0;
                 ALTER TABLE activities ADD COLUMN delay_reason TEXT;
                 CREATE TABLE IF NOT EXISTS lookahead_snapshots (
                     id          INTEGER PRIMARY KEY,
                     name        TEXT    NOT NULL,
                     created_at  TEXT    NOT NULL,
                     window_start TEXT   NOT NULL,
                     window_weeks INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS lookahead_snapshot_entries (
                     snapshot_id     INTEGER NOT NULL REFERENCES lookahead_snapshots(id) ON DELETE CASCADE,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     name            TEXT    NOT NULL,
                     scheduled_start TEXT    NOT NULL,
                     scheduled_finish TEXT   NOT NULL,
                     owner           TEXT,
                     promised        INTEGER NOT NULL DEFAULT 0,
                     percent_complete INTEGER NOT NULL DEFAULT 0,
                     PRIMARY KEY (snapshot_id, activity_id)
                 );
                 CREATE TABLE IF NOT EXISTS daily_notes (
                     id                   INTEGER PRIMARY KEY,
                     date                 TEXT    NOT NULL,
                     content              TEXT    NOT NULL,
                     activity_id          INTEGER REFERENCES activities(id) ON DELETE CASCADE,
                     lookahead_window_start TEXT
                 );
                 ALTER TABLE activities ADD COLUMN location TEXT;
                 CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        3 => {
            // Version 4 adds the recorded actual start and finish dates.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "ALTER TABLE activities ADD COLUMN actual_start TEXT;
                 ALTER TABLE activities ADD COLUMN actual_finish TEXT;
                 ALTER TABLE activities ADD COLUMN owner TEXT;
                 ALTER TABLE activities ADD COLUMN promised INTEGER NOT NULL DEFAULT 0;
                 ALTER TABLE activities ADD COLUMN delay_reason TEXT;
                 CREATE TABLE IF NOT EXISTS lookahead_snapshots (
                     id          INTEGER PRIMARY KEY,
                     name        TEXT    NOT NULL,
                     created_at  TEXT    NOT NULL,
                     window_start TEXT   NOT NULL,
                     window_weeks INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS lookahead_snapshot_entries (
                     snapshot_id     INTEGER NOT NULL REFERENCES lookahead_snapshots(id) ON DELETE CASCADE,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     name            TEXT    NOT NULL,
                     scheduled_start TEXT    NOT NULL,
                     scheduled_finish TEXT   NOT NULL,
                     owner           TEXT,
                     promised        INTEGER NOT NULL DEFAULT 0,
                     percent_complete INTEGER NOT NULL DEFAULT 0,
                     PRIMARY KEY (snapshot_id, activity_id)
                 );
                 CREATE TABLE IF NOT EXISTS daily_notes (
                     id                   INTEGER PRIMARY KEY,
                     date                 TEXT    NOT NULL,
                     content              TEXT    NOT NULL,
                     activity_id          INTEGER REFERENCES activities(id) ON DELETE CASCADE,
                     lookahead_window_start TEXT
                 );
                 ALTER TABLE activities ADD COLUMN location TEXT;
                 CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        4 => {
            // Version 5 adds field-planning fields: the activity owner, the
            // promised flag, and the delay reason recorded against a
            // promised activity.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "ALTER TABLE activities ADD COLUMN owner TEXT;
                 ALTER TABLE activities ADD COLUMN promised INTEGER NOT NULL DEFAULT 0;
                 ALTER TABLE activities ADD COLUMN delay_reason TEXT;
                 CREATE TABLE IF NOT EXISTS lookahead_snapshots (
                     id          INTEGER PRIMARY KEY,
                     name        TEXT    NOT NULL,
                     created_at  TEXT    NOT NULL,
                     window_start TEXT   NOT NULL,
                     window_weeks INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS lookahead_snapshot_entries (
                     snapshot_id     INTEGER NOT NULL REFERENCES lookahead_snapshots(id) ON DELETE CASCADE,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     name            TEXT    NOT NULL,
                     scheduled_start TEXT    NOT NULL,
                     scheduled_finish TEXT   NOT NULL,
                     owner           TEXT,
                     promised        INTEGER NOT NULL DEFAULT 0,
                     percent_complete INTEGER NOT NULL DEFAULT 0,
                     PRIMARY KEY (snapshot_id, activity_id)
                 );
                 CREATE TABLE IF NOT EXISTS daily_notes (
                     id                   INTEGER PRIMARY KEY,
                     date                 TEXT    NOT NULL,
                     content              TEXT    NOT NULL,
                     activity_id          INTEGER REFERENCES activities(id) ON DELETE CASCADE,
                     lookahead_window_start TEXT
                 );
                 ALTER TABLE activities ADD COLUMN location TEXT;
                 CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        5 => {
            // Version 6 adds lookahead snapshots: published, read-only views
            // of the near-term plan.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS lookahead_snapshots (
                    id          INTEGER PRIMARY KEY,
                    name        TEXT    NOT NULL,
                    created_at  TEXT    NOT NULL,
                    window_start TEXT   NOT NULL,
                    window_weeks INTEGER NOT NULL
                );
                 CREATE TABLE IF NOT EXISTS lookahead_snapshot_entries (
                    snapshot_id     INTEGER NOT NULL REFERENCES lookahead_snapshots(id) ON DELETE CASCADE,
                    activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                    name            TEXT    NOT NULL,
                    scheduled_start TEXT    NOT NULL,
                    scheduled_finish TEXT   NOT NULL,
                    owner           TEXT,
                    promised        INTEGER NOT NULL DEFAULT 0,
                    percent_complete INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (snapshot_id, activity_id)
                );
                 CREATE TABLE IF NOT EXISTS daily_notes (
                    id                   INTEGER PRIMARY KEY,
                    date                 TEXT    NOT NULL,
                    content              TEXT    NOT NULL,
                    activity_id          INTEGER REFERENCES activities(id) ON DELETE CASCADE,
                    lookahead_window_start TEXT
                );
                 ALTER TABLE activities ADD COLUMN location TEXT;
                 CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        6 => {
            // Version 7 adds daily notes: dated diary entries linked to
            // activities or lookahead windows. Version 8 adds the optional
            // work location. Migrate straight through to the current version.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS daily_notes (
                    id                   INTEGER PRIMARY KEY,
                    date                 TEXT    NOT NULL,
                    content              TEXT    NOT NULL,
                    activity_id          INTEGER REFERENCES activities(id) ON DELETE CASCADE,
                    lookahead_window_start TEXT
                );
                 ALTER TABLE activities ADD COLUMN location TEXT;
                 CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        7 => {
            // Version 8 adds the optional activity work location. Version 9
            // adds production entries. Migrate straight through to the
            // current schema version.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "ALTER TABLE activities ADD COLUMN location TEXT;
                 CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        8 => {
            // Version 9 adds production entries: dated actual progress
            // recorded against activities.
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS production_entries (
                     id              INTEGER PRIMARY KEY,
                     activity_id     INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
                     date            TEXT    NOT NULL,
                     percent_complete INTEGER NOT NULL,
                     UNIQUE (activity_id, date)
                 );
                 PRAGMA user_version = 9;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        SCHEMA_VERSION => Ok(()),
        found => Err(PersistenceError::UnsupportedVersion {
            found,
            supported: SCHEMA_VERSION,
        }),
    }
}

fn validate(project: &Project) -> Result<()> {
    for activity in &project.activities {
        if let Some(wbs) = activity.wbs {
            if project.wbs.node(wbs).is_none() {
                return Err(PersistenceError::InvalidData(format!(
                    "activity {} references missing WBS node {wbs}",
                    activity.id
                )));
            }
        }
        if let Some(calendar) = activity.calendar {
            if project.calendars.named(calendar).is_none() {
                return Err(PersistenceError::InvalidData(format!(
                    "activity {} references missing calendar {calendar}",
                    activity.id
                )));
            }
        }
        for resource in &activity.resources {
            if !project.resources.iter().any(|known| known.id == *resource) {
                return Err(PersistenceError::InvalidData(format!(
                    "activity {} references missing resource {resource}",
                    activity.id
                )));
            }
        }
    }
    for relationship in &project.relationships {
        if project.activity(relationship.predecessor).is_none() {
            return Err(PersistenceError::InvalidData(format!(
                "relationship {} references missing predecessor {}",
                relationship.id, relationship.predecessor
            )));
        }
        if project.activity(relationship.successor).is_none() {
            return Err(PersistenceError::InvalidData(format!(
                "relationship {} references missing successor {}",
                relationship.id, relationship.successor
            )));
        }
    }
    for note in &project.daily_notes {
        if let Some(activity) = note.activity_id {
            if project.activity(activity).is_none() {
                return Err(PersistenceError::InvalidData(format!(
                    "daily note {} references missing activity {activity}",
                    note.id
                )));
            }
        }
    }
    for baseline in &project.baselines {
        for snapshot in &baseline.snapshots {
            if project.activity(snapshot.activity_id).is_none() {
                return Err(PersistenceError::InvalidData(format!(
                    "baseline {} snapshot references missing activity {}",
                    baseline.id, snapshot.activity_id
                )));
            }
        }
    }
    for snapshot in &project.lookahead_snapshots {
        for entry in &snapshot.entries {
            if project.activity(entry.activity_id).is_none() {
                return Err(PersistenceError::InvalidData(format!(
                    "lookahead snapshot {} references missing activity {}",
                    snapshot.id, entry.activity_id
                )));
            }
        }
    }
    for entry in &project.production_entries {
        if !(0..=10_000).contains(&entry.percent_complete) {
            return Err(PersistenceError::InvalidData(format!(
                "production entry {} has percent complete {} out of range",
                entry.id, entry.percent_complete
            )));
        }
        if project.activity(entry.activity_id).is_none() {
            return Err(PersistenceError::InvalidData(format!(
                "production entry {} references missing activity {}",
                entry.id, entry.activity_id
            )));
        }
    }
    Ok(())
}

fn clear_project(transaction: &rusqlite::Transaction<'_>) -> Result<()> {
    transaction.execute_batch(
        "DELETE FROM lookahead_snapshot_entries;
         DELETE FROM lookahead_snapshots;
         DELETE FROM daily_notes;
         DELETE FROM production_entries;
         DELETE FROM baseline_snapshots;
         DELETE FROM baselines;
         DELETE FROM activity_resources;
         DELETE FROM resources;
         DELETE FROM relationships;
         DELETE FROM activities;
         DELETE FROM wbs_nodes;
         DELETE FROM calendar_holidays;
         DELETE FROM calendars;
         DELETE FROM project_holidays;
         DELETE FROM project;",
    )?;
    Ok(())
}

fn write_project(transaction: &rusqlite::Transaction<'_>, project: &Project) -> Result<()> {
    transaction.execute(
        "INSERT INTO project (id, name, start_date, workweek) VALUES (1, ?1, ?2, ?3)",
        params![
            project.name,
            project.start_date.to_string(),
            encode_week(project.calendar.working_weekdays())
        ],
    )?;

    for holiday in project.calendar.holidays() {
        transaction.execute(
            "INSERT INTO project_holidays (date) VALUES (?1)",
            params![holiday.to_string()],
        )?;
    }

    for (position, named) in project.calendars.calendars().iter().enumerate() {
        transaction.execute(
            "INSERT INTO calendars (id, name, workweek, position) VALUES (?1, ?2, ?3, ?4)",
            params![
                to_sql_id(named.id.0, "calendar")?,
                named.name,
                encode_week(named.calendar.working_weekdays()),
                position as i64,
            ],
        )?;
        for holiday in named.calendar.holidays() {
            transaction.execute(
                "INSERT INTO calendar_holidays (calendar_id, date) VALUES (?1, ?2)",
                params![to_sql_id(named.id.0, "calendar")?, holiday.to_string()],
            )?;
        }
    }

    for (preorder, id) in preorder_nodes(&project.wbs).into_iter().enumerate() {
        let node = project
            .wbs
            .node(id)
            .ok_or_else(|| PersistenceError::InvalidData("WBS node vanished".to_string()))?;
        transaction.execute(
            "INSERT INTO wbs_nodes (id, name, parent_id, preorder) VALUES (?1, ?2, ?3, ?4)",
            params![
                to_sql_id(id.0, "WBS node")?,
                node.name,
                node.parent
                    .map(|parent| to_sql_id(parent.0, "WBS node"))
                    .transpose()?,
                preorder as i64,
            ],
        )?;
    }

    for (position, activity) in project.activities.iter().enumerate() {
        let constraint_date = activity.constraint.date().map(|date| date.to_string());
        transaction.execute(
            "INSERT INTO activities
                (id, name, duration, wbs_id, calendar_id, constraint_kind, constraint_date,
                 budget_at_completion, actual_cost, percent_complete, actual_start,
                 actual_finish, owner, promised, delay_reason, location, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                to_sql_id(activity.id.0, "activity")?,
                activity.name,
                i64::from(activity.original_duration),
                activity
                    .wbs
                    .map(|wbs| to_sql_id(wbs.0, "WBS node"))
                    .transpose()?,
                activity
                    .calendar
                    .map(|calendar| to_sql_id(calendar.0, "calendar"))
                    .transpose()?,
                activity.constraint.code(),
                constraint_date,
                activity.budget_at_completion,
                activity.actual_cost,
                activity.percent_complete,
                activity.actual_start.map(|date| date.to_string()),
                activity.actual_finish.map(|date| date.to_string()),
                activity.owner,
                activity.promised,
                activity.delay_reason,
                activity.location,
                position as i64,
            ],
        )?;
    }

    for (position, relationship) in project.relationships.iter().enumerate() {
        transaction.execute(
            "INSERT INTO relationships (id, predecessor, successor, link_type, lag, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                to_sql_id(relationship.id.0, "relationship")?,
                to_sql_id(relationship.predecessor.0, "activity")?,
                to_sql_id(relationship.successor.0, "activity")?,
                relationship.link_type.abbreviation(),
                relationship.lag,
                position as i64,
            ],
        )?;
    }

    for resource in project.resources.iter() {
        transaction.execute(
            "INSERT INTO resources (id, name, kind, capacity) VALUES (?1, ?2, ?3, ?4)",
            params![
                to_sql_id(resource.id.0, "resource")?,
                resource.name,
                match resource.kind {
                    ResourceKind::Labour => "labour",
                    ResourceKind::Equipment => "equipment",
                    ResourceKind::Material => "material",
                },
                i64::from(resource.capacity),
            ],
        )?;
    }

    for activity in &project.activities {
        for &resource_id in &activity.resources {
            transaction.execute(
                "INSERT INTO activity_resources (activity_id, resource_id) VALUES (?1, ?2)",
                params![
                    to_sql_id(activity.id.0, "activity")?,
                    to_sql_id(resource_id.0, "resource")?,
                ],
            )?;
        }
    }

    for baseline in &project.baselines {
        transaction.execute(
            "INSERT INTO baselines (id, name, created_at) VALUES (?1, ?2, ?3)",
            params![
                to_sql_id(baseline.id.0, "baseline")?,
                baseline.name,
                baseline.created_at.to_string(),
            ],
        )?;
        for snapshot in &baseline.snapshots {
            transaction.execute(
                "INSERT INTO baseline_snapshots (baseline_id, activity_id, baseline_duration, baseline_start, baseline_finish)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    to_sql_id(baseline.id.0, "baseline")?,
                    to_sql_id(snapshot.activity_id.0, "activity")?,
                    i64::from(snapshot.baseline_duration),
                    snapshot.baseline_start.map(|date| date.to_string()),
                    snapshot.baseline_finish.map(|date| date.to_string()),
                ],
            )?;
        }
    }

    for snapshot in &project.lookahead_snapshots {
        transaction.execute(
            "INSERT INTO lookahead_snapshots (id, name, created_at, window_start, window_weeks)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                to_sql_id(snapshot.id.0, "lookahead_snapshot")?,
                snapshot.name,
                snapshot.created_at.to_string(),
                snapshot.window_start.to_string(),
                i64::from(snapshot.window_weeks),
            ],
        )?;
        for entry in &snapshot.entries {
            transaction.execute(
                "INSERT INTO lookahead_snapshot_entries (snapshot_id, activity_id, name, scheduled_start, scheduled_finish, owner, promised, percent_complete)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    to_sql_id(snapshot.id.0, "lookahead_snapshot")?,
                    to_sql_id(entry.activity_id.0, "activity")?,
                    entry.name,
                    entry.scheduled_start.to_string(),
                    entry.scheduled_finish.to_string(),
                    entry.owner,
                    i64::from(entry.promised),
                    entry.percent_complete,
                ],
            )?;
        }
    }

    for note in &project.daily_notes {
        let activity_id_sql = note
            .activity_id
            .map(|id| to_sql_id(id.0, "activity"))
            .transpose()?;
        transaction.execute(
            "INSERT INTO daily_notes (id, date, content, activity_id, lookahead_window_start)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                to_sql_id(note.id.0, "daily_note")?,
                note.date.to_string(),
                note.content,
                activity_id_sql,
                note.lookahead_window_start.map(|date| date.to_string()),
            ],
        )?;
    }

    for entry in &project.production_entries {
        transaction.execute(
            "INSERT INTO production_entries (id, activity_id, date, percent_complete)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                to_sql_id(entry.id.0, "production_entry")?,
                to_sql_id(entry.activity_id.0, "activity")?,
                entry.date.to_string(),
                entry.percent_complete,
            ],
        )?;
    }

    Ok(())
}

fn read_project(connection: &Connection) -> Result<Project> {
    let (name, start_date, workweek) = connection
        .query_row(
            "SELECT name, start_date, workweek FROM project WHERE id = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                PersistenceError::InvalidData("project row is missing".to_string())
            }
            other => PersistenceError::Database(other),
        })?;

    let default_calendar = read_default_calendar(connection, workweek)?;
    let calendars = read_calendars(connection)?;
    let wbs = read_wbs(connection)?;
    let activities = read_activities(connection)?;
    let relationships = read_relationships(connection)?;
    let resources = read_resources(connection)?;
    let activity_resources = read_activity_resources(connection)?;
    let baselines = read_baselines(connection)?;
    let baseline_snapshots = read_baseline_snapshots(connection)?;

    let mut project = Project::new(name, parse_date(&start_date)?);
    project.calendar = default_calendar;
    project.calendars = calendars;
    project.wbs = wbs;
    project.activities = activities;
    project.relationships = relationships;
    project.resources = resources;
    for (activity_id, resource_id) in activity_resources {
        if let Some(activity) = project.activity_mut(activity_id) {
            activity.resources.push(resource_id);
        }
    }
    project.baselines = baselines;
    for row in baseline_snapshots {
        if let Some(baseline) = project
            .baselines
            .iter_mut()
            .find(|candidate| candidate.id == BaselineId(row.baseline_id))
        {
            baseline.snapshots.push(ActivityBaselineSnapshot {
                activity_id: ActivityId(row.activity_id),
                baseline_duration: row.duration,
                baseline_start: row.start,
                baseline_finish: row.finish,
            });
        }
    }

    let lookahead_snapshots = read_lookahead_snapshots(connection)?;
    let lookahead_snapshot_entries = read_lookahead_snapshot_entries(connection)?;
    project.lookahead_snapshots = lookahead_snapshots;
    for row in lookahead_snapshot_entries {
        if let Some(snapshot) = project
            .lookahead_snapshots
            .iter_mut()
            .find(|candidate| candidate.id == LookaheadSnapshotId(row.snapshot_id))
        {
            snapshot.entries.push(LookaheadSnapshotEntry::new(
                ActivityId(row.activity_id),
                row.name,
                row.scheduled_start,
                row.scheduled_finish,
                row.owner,
                row.promised,
                row.percent_complete,
            ));
        }
    }
    project
        .lookahead_snapshots
        .sort_by_key(|snapshot| snapshot.id.0);
    for snapshot in &mut project.lookahead_snapshots {
        snapshot.entries.sort_by_key(|entry| entry.activity_id);
    }

    let daily_notes = read_daily_notes(connection)?;
    project.daily_notes = daily_notes
        .into_iter()
        .map(|row| {
            DailyNote::new(
                DailyNoteId(row.id),
                row.date,
                row.content,
                row.activity_id,
                row.lookahead_window_start,
            )
        })
        .collect();
    project.daily_notes.sort_by_key(|note| note.id.0);

    project.production_entries = read_production_entries(connection)?;
    project.production_entries.sort_by_key(|entry| entry.id.0);
    Ok(project)
}

fn read_production_entries(connection: &Connection) -> Result<Vec<ProductionEntry>> {
    let rows: Vec<_> = collect_rows(
        connection,
        "SELECT id, activity_id, date, percent_complete FROM production_entries",
        |row| {
            let id = row.get::<_, i64>(0)?;
            let activity_id = row.get::<_, i64>(1)?;
            let date_str = row.get::<_, String>(2)?;
            let percent_complete = row.get::<_, i64>(3)?;
            let date = parse_date(&date_str).map_err(|e| {
                rusqlite::Error::InvalidQuery(format!("invalid date: {e}"))
            })?;
            Ok(ProductionEntry::new(
                ProductionEntryId(id as u64),
                ActivityId(activity_id as u64),
                date,
                percent_complete,
            ))
        },
    )?;
    Ok(rows)
}

fn read_default_calendar(connection: &Connection, workweek: i64) -> Result<Calendar> {
    let mut calendar = Calendar::new(decode_week(workweek)?)
        .map_err(|error| PersistenceError::InvalidData(format!("default calendar: {error}")))?;
    for holiday in read_holidays(connection, "SELECT date FROM project_holidays")? {
        calendar = calendar.with_holiday(holiday);
    }
    Ok(calendar)
}

fn read_calendars(connection: &Connection) -> Result<CalendarSet> {
    let rows = collect_rows(
        connection,
        "SELECT id, name, workweek FROM calendars ORDER BY position",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        },
    )?;

    let mut calendars = CalendarSet::new();
    for (id, name, workweek) in rows {
        let mut calendar = Calendar::new(decode_week(workweek)?).map_err(|error| {
            PersistenceError::InvalidData(format!("calendar \"{name}\": {error}"))
        })?;
        let mut statement =
            connection.prepare("SELECT date FROM calendar_holidays WHERE calendar_id = ?1")?;
        let holidays = statement
            .query_map(params![id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for holiday in holidays {
            calendar = calendar.with_holiday(parse_date(&holiday)?);
        }
        calendars
            .add(CalendarId(from_sql_id(id, "calendar")?), name, calendar)
            .map_err(|error| PersistenceError::InvalidData(error.to_string()))?;
    }
    Ok(calendars)
}

fn read_wbs(connection: &Connection) -> Result<WbsTree> {
    let rows = collect_rows(
        connection,
        "SELECT id, name, parent_id, preorder FROM wbs_nodes ORDER BY preorder",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        },
    )?;

    let mut tree = WbsTree::new();
    let mut root_seen = false;
    for (id, name, parent, preorder) in rows {
        if preorder < 0 {
            return Err(PersistenceError::InvalidData(
                "negative WBS preorder".to_string(),
            ));
        }
        let id = WbsId(from_sql_id(id, "WBS node")?);
        match parent {
            None => {
                if root_seen {
                    return Err(PersistenceError::InvalidData(
                        "multiple WBS roots".to_string(),
                    ));
                }
                if id != tree.root() {
                    return Err(PersistenceError::InvalidData(format!(
                        "WBS root must have id {}, found {id}",
                        tree.root()
                    )));
                }
                tree.rename(id, name)
                    .map_err(|error| PersistenceError::InvalidData(error.to_string()))?;
                root_seen = true;
            }
            Some(parent) => {
                if !root_seen {
                    return Err(PersistenceError::InvalidData(
                        "WBS root must be the first node".to_string(),
                    ));
                }
                let parent = WbsId(from_sql_id(parent, "WBS node")?);
                tree.add(id, name, parent)
                    .map_err(|error| PersistenceError::InvalidData(error.to_string()))?;
            }
        }
    }
    if !root_seen {
        return Err(PersistenceError::InvalidData(
            "WBS root is missing".to_string(),
        ));
    }
    Ok(tree)
}

fn read_activities(connection: &Connection) -> Result<Vec<Activity>> {
    let rows = collect_rows(
        connection,
        "SELECT id, name, duration, wbs_id, calendar_id, constraint_kind, constraint_date,
                budget_at_completion, actual_cost, percent_complete, actual_start, actual_finish,
                owner, promised, delay_reason, location
         FROM activities ORDER BY position",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<i64>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, i64>(13)?,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, Option<String>>(15)?,
            ))
        },
    )?;

    let mut activities = Vec::with_capacity(rows.len());
    for (
        id,
        name,
        duration,
        wbs,
        calendar,
        constraint_code,
        constraint_date,
        budget,
        cost,
        percent,
        actual_start,
        actual_finish,
        owner,
        promised,
        delay_reason,
        location,
    ) in rows
    {
        let duration = u32::try_from(duration).map_err(|_| {
            PersistenceError::InvalidData(format!("activity {id} has invalid duration {duration}"))
        })?;
        let constraint_date = constraint_date
            .map(|value| parse_date(&value))
            .transpose()?;
        let constraint =
            Constraint::from_parts(&constraint_code, constraint_date).ok_or_else(|| {
                PersistenceError::InvalidData(format!(
                    "activity {id} has invalid constraint \"{constraint_code}\""
                ))
            })?;
        let mut activity = Activity::new(ActivityId(from_sql_id(id, "activity")?), name, duration);
        activity.constraint = constraint;
        activity.wbs = wbs
            .map(|wbs| from_sql_id(wbs, "WBS node").map(WbsId))
            .transpose()?;
        activity.calendar = calendar
            .map(|calendar| from_sql_id(calendar, "calendar").map(CalendarId))
            .transpose()?;
        activity.budget_at_completion = budget;
        activity.actual_cost = cost;
        activity.percent_complete = percent;
        activity.actual_start = actual_start.map(|value| parse_date(&value)).transpose()?;
        activity.actual_finish = actual_finish.map(|value| parse_date(&value)).transpose()?;
        activity.owner = owner;
        activity.promised = promised != 0;
        activity.delay_reason = delay_reason;
        activity.location = location;
        activities.push(activity);
    }
    Ok(activities)
}

fn read_relationships(connection: &Connection) -> Result<Vec<Relationship>> {
    let rows = collect_rows(
        connection,
        "SELECT id, predecessor, successor, link_type, lag
         FROM relationships ORDER BY position",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        },
    )?;

    let mut relationships = Vec::with_capacity(rows.len());
    for (id, predecessor, successor, link_type, lag) in rows {
        let link_type = LinkType::from_abbreviation(&link_type).ok_or_else(|| {
            PersistenceError::InvalidData(format!(
                "relationship {id} has unknown link type \"{link_type}\""
            ))
        })?;
        relationships.push(Relationship::new(
            RelationshipId(from_sql_id(id, "relationship")?),
            ActivityId(from_sql_id(predecessor, "activity")?),
            ActivityId(from_sql_id(successor, "activity")?),
            link_type,
            lag,
        ));
    }
    Ok(relationships)
}

fn read_resources(connection: &Connection) -> Result<Vec<Resource>> {
    let rows = collect_rows(
        connection,
        "SELECT id, name, kind, capacity FROM resources ORDER BY id",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        },
    )?;

    let mut resources = Vec::with_capacity(rows.len());
    for (id, name, kind, capacity) in rows {
        let kind = match kind.as_str() {
            "labour" => ResourceKind::Labour,
            "equipment" => ResourceKind::Equipment,
            "material" => ResourceKind::Material,
            other => {
                return Err(PersistenceError::InvalidData(format!(
                    "resource {id} has unknown kind \"{other}\""
                )))
            }
        };
        let capacity = u32::try_from(capacity).map_err(|_| {
            PersistenceError::InvalidData(format!("resource {id} has invalid capacity {capacity}"))
        })?;
        resources.push(Resource {
            id: ResourceId(from_sql_id(id, "resource")?),
            name,
            kind,
            capacity: capacity.max(1),
        });
    }
    Ok(resources)
}

fn read_activity_resources(connection: &Connection) -> Result<Vec<(ActivityId, ResourceId)>> {
    let rows = collect_rows(
        connection,
        "SELECT activity_id, resource_id FROM activity_resources ORDER BY activity_id, resource_id",
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )?;

    let mut assignments = Vec::with_capacity(rows.len());
    for (activity_id, resource_id) in rows {
        assignments.push((
            ActivityId(from_sql_id(activity_id, "activity")?),
            ResourceId(from_sql_id(resource_id, "resource")?),
        ));
    }
    Ok(assignments)
}

fn read_baselines(connection: &Connection) -> Result<Vec<Baseline>> {
    let rows = collect_rows(
        connection,
        "SELECT id, name, created_at FROM baselines ORDER BY id",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    )?;

    let mut baselines = Vec::with_capacity(rows.len());
    for (id, name, created_at) in rows {
        baselines.push(Baseline {
            id: BaselineId(from_sql_id(id, "baseline")?),
            name,
            created_at: parse_date(&created_at)?,
            snapshots: Vec::new(),
        });
    }
    Ok(baselines)
}

/// One baseline snapshot row as stored in the database.
struct BaselineSnapshotRow {
    /// Baseline the snapshot belongs to.
    baseline_id: u64,
    /// Activity the snapshot describes.
    activity_id: u64,
    /// Planned duration in working days.
    duration: u32,
    /// Planned start date.
    start: Option<Date>,
    /// Planned finish date.
    finish: Option<Date>,
}

fn read_baseline_snapshots(connection: &Connection) -> Result<Vec<BaselineSnapshotRow>> {
    let rows = collect_rows(
        connection,
        "SELECT baseline_id, activity_id, baseline_duration, baseline_start, baseline_finish
         FROM baseline_snapshots ORDER BY baseline_id, activity_id",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        },
    )?;

    let mut snapshots = Vec::with_capacity(rows.len());
    for (baseline_id, activity_id, duration, start, finish) in rows {
        let duration = u32::try_from(duration).map_err(|_| {
            PersistenceError::InvalidData(format!(
                "baseline {baseline_id} has invalid duration {duration}"
            ))
        })?;
        let start = start.as_deref().map(parse_date).transpose()?;
        let finish = finish.as_deref().map(parse_date).transpose()?;
        snapshots.push(BaselineSnapshotRow {
            baseline_id: from_sql_id(baseline_id, "baseline")?,
            activity_id: from_sql_id(activity_id, "activity")?,
            duration,
            start,
            finish,
        });
    }
    Ok(snapshots)
}

fn read_lookahead_snapshots(connection: &Connection) -> Result<Vec<LookaheadSnapshot>> {
    let rows = collect_rows(
        connection,
        "SELECT id, name, created_at, window_start, window_weeks FROM lookahead_snapshots ORDER BY id",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        },
    )?;

    let mut snapshots = Vec::with_capacity(rows.len());
    for (id, name, created_at, window_start, window_weeks) in rows {
        let window_weeks = u32::try_from(window_weeks).map_err(|_| {
            PersistenceError::InvalidData(format!(
                "lookahead snapshot {id} has invalid window_weeks {window_weeks}"
            ))
        })?;
        snapshots.push(LookaheadSnapshot::from_entries(
            LookaheadSnapshotId(from_sql_id(id, "lookahead_snapshot")?),
            name,
            parse_date(&created_at)?,
            parse_date(&window_start)?,
            window_weeks,
            Vec::new(),
        ));
    }
    Ok(snapshots)
}

/// One lookahead snapshot entry row as stored in the database.
struct LookaheadSnapshotEntryRow {
    /// Snapshot the entry belongs to.
    snapshot_id: u64,
    /// Activity the entry describes.
    activity_id: u64,
    /// Activity name at publish time.
    name: String,
    /// Scheduled start date.
    scheduled_start: Date,
    /// Scheduled finish date.
    scheduled_finish: Date,
    /// Field-planning owner at publish time.
    owner: Option<String>,
    /// True when the activity was promised at publish time.
    promised: bool,
    /// Percent complete in basis points at publish time.
    percent_complete: i64,
}

fn read_lookahead_snapshot_entries(
    connection: &Connection,
) -> Result<Vec<LookaheadSnapshotEntryRow>> {
    let rows = collect_rows(
        connection,
        "SELECT snapshot_id, activity_id, name, scheduled_start, scheduled_finish, owner, promised, percent_complete
         FROM lookahead_snapshot_entries ORDER BY snapshot_id, activity_id",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
            ))
        },
    )?;

    let mut entries = Vec::with_capacity(rows.len());
    for (
        snapshot_id,
        activity_id,
        name,
        scheduled_start,
        scheduled_finish,
        owner,
        promised,
        percent_complete,
    ) in rows
    {
        let promised = promised != 0;
        entries.push(LookaheadSnapshotEntryRow {
            snapshot_id: from_sql_id(snapshot_id, "lookahead_snapshot")?,
            activity_id: from_sql_id(activity_id, "activity")?,
            name,
            scheduled_start: parse_date(&scheduled_start)?,
            scheduled_finish: parse_date(&scheduled_finish)?,
            owner,
            promised,
            percent_complete,
        });
    }
    Ok(entries)
}

/// One daily note row as stored in the database.
struct DailyNoteRow {
    /// Note identifier.
    id: u64,
    /// Date of the diary entry.
    date: Date,
    /// Note content.
    content: String,
    /// Optional activity the note concerns.
    activity_id: Option<ActivityId>,
    /// Optional lookahead window start the note concerns.
    lookahead_window_start: Option<Date>,
}

fn read_daily_notes(connection: &Connection) -> Result<Vec<DailyNoteRow>> {
    let rows = collect_rows(
        connection,
        "SELECT id, date, content, activity_id, lookahead_window_start FROM daily_notes ORDER BY date",
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        },
    )?;

    let mut notes = Vec::with_capacity(rows.len());
    for (id, date, content, activity_id, lookahead_window_start) in rows {
        let activity_id = activity_id
            .map(|id| from_sql_id(id, "activity").map(ActivityId))
            .transpose()?;
        let lookahead_window_start = lookahead_window_start
            .as_deref()
            .map(parse_date)
            .transpose()?;
        notes.push(DailyNoteRow {
            id: from_sql_id(id, "daily_note")?,
            date: parse_date(&date)?,
            content,
            activity_id,
            lookahead_window_start,
        });
    }
    Ok(notes)
}

fn read_holidays(connection: &Connection, query: &str) -> Result<BTreeSet<Date>> {
    let values = collect_rows(connection, query, |row| row.get::<_, String>(0))?;
    let mut holidays = BTreeSet::new();
    for value in values {
        holidays.insert(parse_date(&value)?);
    }
    Ok(holidays)
}

fn collect_rows<T>(
    connection: &Connection,
    query: &str,
    map: impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    let mut statement = connection.prepare(query)?;
    let rows = statement.query_map([], map)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Depth-first pre-order identifiers, which preserve sibling order on load.
fn preorder_nodes(tree: &WbsTree) -> Vec<WbsId> {
    let mut order = Vec::new();
    let mut stack = vec![tree.root()];
    while let Some(id) = stack.pop() {
        order.push(id);
        if let Some(node) = tree.node(id) {
            stack.extend(node.children.iter().rev().copied());
        }
    }
    order
}

fn encode_week(days: [bool; 7]) -> i64 {
    let mut mask = 0i64;
    for (index, working) in days.iter().enumerate() {
        if *working {
            mask |= 1i64 << index;
        }
    }
    mask
}

fn decode_week(mask: i64) -> Result<[bool; 7]> {
    if !(0..=0b111_1111).contains(&mask) {
        return Err(PersistenceError::InvalidData(format!(
            "invalid workweek mask {mask}"
        )));
    }
    let mut days = [false; 7];
    for (index, day) in days.iter_mut().enumerate() {
        *day = mask & (1i64 << index) != 0;
    }
    Ok(days)
}

fn parse_date(value: &str) -> Result<Date> {
    let mut parts = value.split('-');
    let year = parts.next().and_then(|part| part.parse::<i32>().ok());
    let month = parts.next().and_then(|part| part.parse::<u8>().ok());
    let day = parts.next().and_then(|part| part.parse::<u8>().ok());
    let extra = parts.next();

    let (Some(year), Some(month), Some(day), None) = (year, month, day, extra) else {
        return Err(PersistenceError::InvalidData(format!(
            "invalid date \"{value}\""
        )));
    };
    let month = Month::try_from(month)
        .map_err(|_| PersistenceError::InvalidData(format!("invalid month in date \"{value}\"")))?;
    Date::from_calendar_date(year, month, day)
        .map_err(|_| PersistenceError::InvalidData(format!("invalid calendar date \"{value}\"")))
}

fn to_sql_id(value: u64, what: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| {
        PersistenceError::InvalidData(format!("{what} id {value} does not fit in SQLite"))
    })
}

fn from_sql_id(value: i64, what: &str) -> Result<u64> {
    u64::try_from(value)
        .map_err(|_| PersistenceError::InvalidData(format!("{what} id {value} is negative")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadence_core::model::LinkType;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "cadence-persist-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn date(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::January, day).expect("valid test date")
    }

    fn sample_project() -> Project {
        let mut project = Project::new("Riverside Bridge", date(5));
        project.calendar = Calendar::weekend().with_holiday(date(6));

        project
            .calendars
            .add(
                CalendarId(1),
                "Six day",
                Calendar::new([true, true, true, true, true, true, false])
                    .expect("six-day week")
                    .with_holiday(date(10)),
            )
            .expect("add calendar");

        let root = project.wbs.root();
        project.wbs.rename(root, "Programme").expect("rename root");
        project.wbs.add(WbsId(2), "Design", root).expect("add node");
        project
            .wbs
            .add(WbsId(3), "Construction", root)
            .expect("add node");
        project
            .wbs
            .add(WbsId(4), "Concrete works", WbsId(3))
            .expect("add node");

        let mut a = Activity::new(ActivityId(1), "Site preparation", 5);
        a.wbs = Some(WbsId(2));
        a.constraint = Constraint::StartNoEarlierThan(date(7));
        let mut b = Activity::new(ActivityId(2), "Foundation", 2);
        b.wbs = Some(WbsId(4));
        b.calendar = Some(CalendarId(1));
        b.constraint = Constraint::AsLateAsPossible;
        let mut c = Activity::new(ActivityId(3), "Milestone", 0);
        c.wbs = Some(WbsId(3));
        c.constraint = Constraint::FinishNoLaterThan(date(20));
        let mut d = Activity::new(ActivityId(4), "Inspection", 1);
        d.wbs = Some(WbsId(2));
        d.constraint = Constraint::StartNoLaterThan(date(16));

        project.activities = vec![a, b, c, d];
        project.relationships = vec![
            Relationship::new(
                RelationshipId(1),
                ActivityId(1),
                ActivityId(2),
                LinkType::Fs,
                0,
            ),
            Relationship::new(
                RelationshipId(2),
                ActivityId(2),
                ActivityId(3),
                LinkType::Ss,
                1,
            ),
            Relationship::new(
                RelationshipId(3),
                ActivityId(1),
                ActivityId(4),
                LinkType::Ff,
                2,
            ),
            Relationship::new(
                RelationshipId(4),
                ActivityId(2),
                ActivityId(4),
                LinkType::Sf,
                -1,
            ),
        ];

        let mut crew = Resource::new(ResourceId(1), "Crew A");
        crew.kind = ResourceKind::Labour;
        let mut crane = Resource::with_capacity(ResourceId(2), "Crane", 3);
        crane.kind = ResourceKind::Equipment;
        project.resources = vec![crew, crane];

        project.activities[0].resources = vec![ResourceId(1)];
        project.activities[1].resources = vec![ResourceId(1), ResourceId(2)];

        project.activities[0].budget_at_completion = Some(12_500);
        project.activities[0].actual_cost = Some(11_000);
        project.activities[0].percent_complete = Some(4_000);
        project.activities[0].actual_start = Some(date(5));
        project.activities[0].actual_finish = Some(date(6));

        project.baselines = vec![Baseline {
            id: BaselineId(1),
            name: "Baseline 1".to_string(),
            created_at: date(5),
            snapshots: vec![
                ActivityBaselineSnapshot {
                    activity_id: ActivityId(1),
                    baseline_duration: 5,
                    baseline_start: Some(date(5)),
                    baseline_finish: Some(date(12)),
                },
                ActivityBaselineSnapshot {
                    activity_id: ActivityId(2),
                    baseline_duration: 2,
                    baseline_start: Some(date(12)),
                    baseline_finish: Some(date(13)),
                },
            ],
        }];

        project
    }

    #[test]
    fn round_trips_every_entity() {
        let dir = TempDir::new();
        let path = dir.file("project.cadence");
        let project = sample_project();

        save_project(&project, &path).expect("save project");
        let loaded = load_project(&path).expect("load project");

        assert_eq!(loaded, project);
    }

    #[test]
    fn resource_dictionary_and_assignments_round_trip() {
        let dir = TempDir::new();
        let path = dir.file("resources.cadence");
        let project = sample_project();

        save_project(&project, &path).expect("save project");
        let loaded = load_project(&path).expect("load project");

        assert_eq!(loaded.resources.len(), 2);
        assert_eq!(loaded.resources[0].id, ResourceId(1));
        assert_eq!(loaded.resources[0].name, "Crew A");
        assert_eq!(loaded.resources[0].kind, ResourceKind::Labour);
        assert_eq!(loaded.resources[0].capacity, 1);
        assert_eq!(loaded.resources[1].id, ResourceId(2));
        assert_eq!(loaded.resources[1].name, "Crane");
        assert_eq!(loaded.resources[1].kind, ResourceKind::Equipment);
        assert_eq!(loaded.resources[1].capacity, 3);

        assert_eq!(loaded.activities[0].resources, vec![ResourceId(1)]);
        assert_eq!(
            loaded.activities[1].resources,
            vec![ResourceId(1), ResourceId(2)]
        );
        assert!(loaded.activities[2].resources.is_empty());
        assert!(loaded.activities[3].resources.is_empty());
    }

    #[test]
    fn activity_cost_fields_round_trip() {
        let dir = TempDir::new();
        let path = dir.file("costs.cadence");
        let project = sample_project();

        save_project(&project, &path).expect("save project");
        let loaded = load_project(&path).expect("load project");

        assert_eq!(loaded.activities[0].budget_at_completion, Some(12_500));
        assert_eq!(loaded.activities[0].actual_cost, Some(11_000));
        assert_eq!(loaded.activities[0].percent_complete, Some(4_000));
        assert_eq!(loaded.activities[0].actual_start, Some(date(5)));
        assert_eq!(loaded.activities[0].actual_finish, Some(date(6)));
        assert_eq!(loaded.activities[1].budget_at_completion, None);
        assert_eq!(loaded.activities[1].actual_cost, None);
        assert_eq!(loaded.activities[1].percent_complete, None);
        assert_eq!(loaded.activities[1].actual_start, None);
        assert_eq!(loaded.activities[1].actual_finish, None);
    }

    #[test]
    fn baselines_round_trip_with_their_snapshots() {
        let dir = TempDir::new();
        let path = dir.file("baselines.cadence");
        let project = sample_project();

        save_project(&project, &path).expect("save project");
        let loaded = load_project(&path).expect("load project");

        assert_eq!(loaded.baselines.len(), 1);
        let baseline = &loaded.baselines[0];
        assert_eq!(baseline.id, BaselineId(1));
        assert_eq!(baseline.name, "Baseline 1");
        assert_eq!(baseline.created_at, date(5));
        assert_eq!(baseline.snapshots.len(), 2);
        assert_eq!(baseline.snapshots[0].activity_id, ActivityId(1));
        assert_eq!(baseline.snapshots[0].baseline_duration, 5);
        assert_eq!(baseline.snapshots[0].baseline_start, Some(date(5)));
        assert_eq!(baseline.snapshots[0].baseline_finish, Some(date(12)));
        assert_eq!(baseline.snapshots[1].activity_id, ActivityId(2));
        assert_eq!(baseline.snapshots[1].baseline_duration, 2);
        assert_eq!(baseline.snapshots[1].baseline_start, Some(date(12)));
        assert_eq!(baseline.snapshots[1].baseline_finish, Some(date(13)));
    }

    #[test]
    fn round_trips_an_empty_project() {
        let dir = TempDir::new();
        let path = dir.file("empty.cadence");
        let project = Project::new("Empty", date(5));

        save_project(&project, &path).expect("save project");
        let loaded = load_project(&path).expect("load project");

        assert_eq!(loaded, project);
    }

    #[test]
    fn saving_replaces_previous_contents() {
        let dir = TempDir::new();
        let path = dir.file("project.cadence");

        save_project(&sample_project(), &path).expect("first save");
        let replacement = Project::new("Replacement", date(12));
        save_project(&replacement, &path).expect("second save");

        let loaded = load_project(&path).expect("load project");
        assert_eq!(loaded, replacement);
    }

    #[test]
    fn loading_a_missing_file_reports_not_found() {
        let dir = TempDir::new();
        let path = dir.file("missing.cadence");

        match load_project(&path) {
            Err(PersistenceError::NotFound(_)) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn newer_schema_versions_are_rejected() {
        let dir = TempDir::new();
        let path = dir.file("future.cadence");
        {
            let connection = Connection::open(&path).expect("create database");
            connection
                .pragma_update(None, "user_version", 99)
                .expect("set version");
        }

        match load_project(&path) {
            Err(PersistenceError::UnsupportedVersion { found: 99, .. }) => {}
            other => panic!("expected UnsupportedVersion, got {other:?}"),
        }
    }

    #[test]
    fn version_one_files_are_migrated_in_place() {
        let dir = TempDir::new();
        let path = dir.file("v1.cadence");
        {
            let connection = Connection::open(&path).expect("create database");
            connection.execute_batch(SCHEMA).expect("create schema");
            // Turn the current schema into the version 1 shape.
            connection
                .execute_batch(
                    "ALTER TABLE activities DROP COLUMN constraint_kind;
                     ALTER TABLE activities DROP COLUMN constraint_date;
                     ALTER TABLE activities DROP COLUMN budget_at_completion;
                     ALTER TABLE activities DROP COLUMN actual_cost;
                     ALTER TABLE activities DROP COLUMN percent_complete;
                     ALTER TABLE activities DROP COLUMN actual_start;
                     ALTER TABLE activities DROP COLUMN actual_finish;
                     ALTER TABLE activities DROP COLUMN owner;
                     ALTER TABLE activities DROP COLUMN promised;
                     ALTER TABLE activities DROP COLUMN delay_reason;
                     ALTER TABLE activities DROP COLUMN location;
                     PRAGMA user_version = 1;",
                )
                .expect("downgrade schema");
            connection
                .execute(
                    "INSERT INTO project (id, name, start_date, workweek) VALUES (1, ?1, ?2, ?3)",
                    params!["Legacy", "2026-01-05", 0b001_1111],
                )
                .expect("insert project");
            connection
                .execute(
                    "INSERT INTO wbs_nodes (id, name, parent_id, preorder)
                     VALUES (1, 'Project', NULL, 0)",
                    [],
                )
                .expect("insert root");
            connection
                .execute(
                    "INSERT INTO activities (id, name, duration, wbs_id, calendar_id, position)
                     VALUES (1, 'Legacy activity', 2, NULL, NULL, 0)",
                    [],
                )
                .expect("insert activity");
        }

        let project = load_project(&path).expect("load and migrate");
        assert_eq!(project.name, "Legacy");
        assert_eq!(project.activities.len(), 1);
        assert_eq!(
            project.activities[0].constraint,
            Constraint::AsSoonAsPossible
        );

        {
            let connection = Connection::open(&path).expect("reopen database");
            let version: i64 = connection
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .expect("read version");
            assert_eq!(version, SCHEMA_VERSION);
        }

        save_project(&project, &path).expect("save after migration");
        assert_eq!(load_project(&path).expect("reload"), project);
    }

    #[test]
    fn version_two_files_are_migrated_in_place() {
        let dir = TempDir::new();
        let path = dir.file("v2.cadence");
        {
            let connection = Connection::open(&path).expect("create database");
            connection.execute_batch(SCHEMA).expect("create schema");
            // Turn the current schema into the version 2 shape.
            connection
                .execute_batch(
                    "ALTER TABLE activities DROP COLUMN budget_at_completion;
                     ALTER TABLE activities DROP COLUMN actual_cost;
                     ALTER TABLE activities DROP COLUMN percent_complete;
                     ALTER TABLE activities DROP COLUMN actual_start;
                     ALTER TABLE activities DROP COLUMN actual_finish;
                     ALTER TABLE activities DROP COLUMN owner;
                     ALTER TABLE activities DROP COLUMN promised;
                     ALTER TABLE activities DROP COLUMN delay_reason;
                     ALTER TABLE activities DROP COLUMN location;
                     DROP TABLE baseline_snapshots;
                     DROP TABLE baselines;
                     DROP TABLE activity_resources;
                     DROP TABLE resources;
                     PRAGMA user_version = 2;",
                )
                .expect("downgrade schema");
            connection
                .execute(
                    "INSERT INTO project (id, name, start_date, workweek) VALUES (1, ?1, ?2, ?3)",
                    params!["Legacy", "2026-01-05", 0b001_1111],
                )
                .expect("insert project");
            connection
                .execute(
                    "INSERT INTO wbs_nodes (id, name, parent_id, preorder)
                     VALUES (1, 'Project', NULL, 0)",
                    [],
                )
                .expect("insert root");
            connection
                .execute(
                    "INSERT INTO activities (id, name, duration, wbs_id, calendar_id, position)
                     VALUES (1, 'Legacy activity', 2, NULL, NULL, 0)",
                    [],
                )
                .expect("insert activity");
        }

        let project = load_project(&path).expect("load and migrate");
        assert_eq!(project.name, "Legacy");
        assert_eq!(project.activities.len(), 1);
        assert!(project.resources.is_empty());
        assert!(project.baselines.is_empty());
        assert_eq!(project.activities[0].budget_at_completion, None);
        assert_eq!(project.activities[0].actual_cost, None);
        assert_eq!(project.activities[0].percent_complete, None);

        {
            let connection = Connection::open(&path).expect("reopen database");
            let version: i64 = connection
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .expect("read version");
            assert_eq!(version, SCHEMA_VERSION);
        }

        save_project(&project, &path).expect("save after migration");
        assert_eq!(load_project(&path).expect("reload"), project);
    }

    #[test]
    fn dangling_references_are_rejected_before_writing() {
        let dir = TempDir::new();
        let path = dir.file("dangling.cadence");
        let mut project = Project::new("Broken", date(5));
        let mut activity = Activity::new(ActivityId(1), "A", 1);
        activity.wbs = Some(WbsId(99));
        project.activities.push(activity);

        match save_project(&project, &path) {
            Err(PersistenceError::InvalidData(message)) => {
                assert!(message.contains("missing WBS node"), "{message}");
            }
            other => panic!("expected InvalidData, got {other:?}"),
        }
    }

    #[test]
    fn relationship_endpoints_are_validated_on_save() {
        let dir = TempDir::new();
        let path = dir.file("endpoint.cadence");
        let mut project = Project::new("Broken", date(5));
        project
            .activities
            .push(Activity::new(ActivityId(1), "A", 1));
        project.relationships.push(Relationship::new(
            RelationshipId(1),
            ActivityId(1),
            ActivityId(2),
            LinkType::Fs,
            0,
        ));

        match save_project(&project, &path) {
            Err(PersistenceError::InvalidData(message)) => {
                assert!(message.contains("missing successor"), "{message}");
            }
            other => panic!("expected InvalidData, got {other:?}"),
        }
    }

    #[test]
    fn daily_notes_with_missing_activities_are_rejected_on_save() {
        let dir = TempDir::new();
        let path = dir.file("dangling-note.cadence");

        let mut broken = sample_project();
        broken.daily_notes.push(DailyNote::new(
            DailyNoteId(1),
            date(6),
            "Note about a removed activity".to_string(),
            Some(ActivityId(999)),
            None,
        ));

        match save_project(&broken, &path) {
            Err(PersistenceError::InvalidData(message)) => {
                assert!(message.contains("missing activity"), "{message}");
            }
            other => panic!("expected InvalidData, got {other:?}"),
        }

        // A normal project with a note linked to a real activity is unaffected
        // and still round-trips.
        let mut healthy = sample_project();
        healthy.daily_notes.push(DailyNote::new(
            DailyNoteId(1),
            date(6),
            "Crane late; resequenced pour".to_string(),
            Some(ActivityId(1)),
            None,
        ));
        save_project(&healthy, &path).expect("save healthy project");
        let loaded = load_project(&path).expect("load healthy project");
        assert_eq!(loaded, healthy);
    }

    #[test]
    fn workweek_masks_round_trip() {
        for mask in 0..=0b111_1111 {
            assert_eq!(encode_week(decode_week(mask).expect("valid mask")), mask);
        }
        assert!(decode_week(0b1000_0000).is_err());
        assert!(decode_week(-1).is_err());
    }

    #[test]
    fn dates_round_trip_and_invalid_values_are_rejected() {
        assert_eq!(parse_date("2026-01-05").expect("parse"), date(5));
        assert!(parse_date("2026-13-05").is_err());
        assert!(parse_date("2026-01-32").is_err());
        assert!(parse_date("2026-01").is_err());
        assert!(parse_date("2026-01-05-01").is_err());
        assert!(parse_date("not-a-date").is_err());
    }

    #[test]
    fn field_planning_fields_round_trip() {
        let mut project = sample_project();
        project.activities[0].owner = Some("Crew 3".to_string());
        project.activities[0].promised = true;
        project.activities[0].delay_reason = Some("waiting on a permit".to_string());
        project.activities[0].location = Some("Level 1".to_string());

        let dir = TempDir::new();
        let path = dir.file("field.cadence");
        save_project(&project, &path).expect("save");
        let loaded = load_project(&path).expect("load");

        assert_eq!(loaded.activities[0].owner, Some("Crew 3".to_string()));
        assert!(loaded.activities[0].promised);
        assert_eq!(
            loaded.activities[0].delay_reason,
            Some("waiting on a permit".to_string())
        );
        assert_eq!(loaded.activities[0].location, Some("Level 1".to_string()));
        assert_eq!(loaded.activities[1].owner, None);
        assert!(!loaded.activities[1].promised);
        assert_eq!(loaded.activities[1].location, None);
    }

    #[test]
    fn daily_notes_round_trip() {
        let mut project = sample_project();
        project.daily_notes.push(DailyNote::new(
            DailyNoteId(1),
            date(6),
            "Crane late; resequenced pour".to_string(),
            Some(ActivityId(1)),
            None,
        ));
        project.daily_notes.push(DailyNote::new(
            DailyNoteId(2),
            date(7),
            "Week 1 huddle briefed".to_string(),
            None,
            Some(date(5)),
        ));
        project
            .lookahead_snapshots
            .push(LookaheadSnapshot::from_entries(
                LookaheadSnapshotId(1),
                "Week 1-2",
                date(5),
                date(5),
                2,
                Vec::new(),
            ));

        let dir = TempDir::new();
        let path = dir.file("diary.cadence");
        save_project(&project, &path).expect("first save");
        let loaded = load_project(&path).expect("load");
        assert_eq!(loaded.daily_notes, project.daily_notes);
        assert_eq!(loaded.lookahead_snapshots, project.lookahead_snapshots);

        // Saving a second time must replace the snapshot and note rows rather
        // than colliding with them.
        save_project(&loaded, &path).expect("re-save");
        let reloaded = load_project(&path).expect("reload");
        assert_eq!(reloaded.daily_notes, project.daily_notes);
        assert_eq!(reloaded.lookahead_snapshots, project.lookahead_snapshots);
    }

    #[test]
    fn a_version_4_file_migrates_to_field_planning() {
        let project = sample_project();
        let dir = TempDir::new();
        let path = dir.file("v4.cadence");
        save_project(&project, &path).expect("save");
        // Recreate exactly what a v4 writer left: no new columns, user_version 4.
        let connection = Connection::open(&path).expect("open");
        connection
            .execute_batch(
                "ALTER TABLE activities DROP COLUMN owner;
                 ALTER TABLE activities DROP COLUMN promised;
                 ALTER TABLE activities DROP COLUMN delay_reason;
                 ALTER TABLE activities DROP COLUMN location;
                 PRAGMA user_version = 4;",
            )
            .expect("downgrade");
        drop(connection);

        let loaded = load_project(&path).expect("migrating load");
        assert_eq!(loaded.activities.len(), project.activities.len());
        assert_eq!(loaded.activities[0].owner, None);
        assert!(!loaded.activities[0].promised);
    }
}
