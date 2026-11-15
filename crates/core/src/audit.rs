//! Programme audit: schedule-quality and health checks over a scheduled
//! project.
//!
//! The audit is a pure function of a project that has already been scheduled.
//! It runs a battery of independent checks — structure and logic, float,
//! constraints, dates, resources and baseline performance — and reports, per
//! check, whether the programme passes, with the measured value, the threshold,
//! a plain-language sentence and the offending activities or relationships.
//!
//! The checks and their default thresholds follow public schedule-quality
//! practice (DCMA 14-point assessment, GAO schedule-assessment best practices)
//! restated in Cadence's own words; see `docs/research/schedule-audit.md` for
//! the public sources and the full design. Everything is offline and
//! deterministic: no I/O, no clock, no randomness.

use std::collections::{BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use time::Date;

use crate::engine::{schedule_with_minimum_starts, Schedule};
use crate::model::{
    Activity, ActivityId, Baseline, Calendar, Constraint, LinkType, Project, RelationshipId,
};

/// Thresholds and context for an audit.
///
/// Basis points are one hundredth of a percent: `500` is `5%`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Status date for date and baseline-performance checks. `None` disables
    /// the checks that need it, which then report [`CheckOutcome::NotApplicable`].
    pub status_date: Option<Date>,
    /// Percentage limit for percentage checks (logic, lags, hard constraints,
    /// unnecessary constraints, high float, high duration, missed tasks).
    /// Default 500 (5%).
    pub percentage_limit_bp: u32,
    /// Total float above this many working days counts as high. Default 44.
    pub high_float_days: i64,
    /// Duration above this many working days counts as high. Default 44.
    pub high_duration_days: u32,
    /// A positive lag above this many working days counts as a long lag.
    /// Default 5, one working week.
    #[serde(default = "default_long_lag_days")]
    pub long_lag_days: i64,
    /// Minimum share of finish-to-start relationships. Default 9000 (90%).
    pub min_finish_to_start_bp: u32,
    /// Minimum baseline execution index. Default 9500 (95%).
    pub bei_min_bp: u32,
    /// Minimum critical path length index. Default 9500 (95%).
    pub cpli_min_bp: u32,
    /// Working days the critical-path test delays one critical activity by.
    /// Default 600, the DCMA convention.
    pub critical_path_nudge_days: u32,
}

/// Default long-lag threshold: one working week.
const fn default_long_lag_days() -> i64 {
    5
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            status_date: None,
            percentage_limit_bp: 500,
            high_float_days: 44,
            high_duration_days: 44,
            long_lag_days: default_long_lag_days(),
            min_finish_to_start_bp: 9000,
            bei_min_bp: 9500,
            cpli_min_bp: 9500,
            critical_path_nudge_days: 600,
        }
    }
}

/// Stable identity of a check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckId {
    /// Open activities missing a predecessor or successor.
    LogicOpenEnds,
    /// Relationships with a negative lag.
    Leads,
    /// Relationships with a positive lag.
    Lags,
    /// Share of finish-to-start relationships.
    RelationshipTypes,
    /// Open activities held by a hard date constraint.
    HardConstraints,
    /// Open activities with more than the high-float threshold.
    HighFloat,
    /// Activities with negative total float.
    NegativeFloat,
    /// Activities longer than the high-duration threshold.
    HighDuration,
    /// Actual dates after, or forecast dates before, the status date.
    InvalidDates,
    /// Open activities with a duration but no resource.
    MissingResources,
    /// Activities not finished by their baseline finish.
    MissedTasks,
    /// Baseline execution index.
    BaselineExecutionIndex,
    /// Delay one critical activity and check the finish moves with it.
    CriticalPathTest,
    /// Critical path length index against the baseline deadline.
    CriticalPathLengthIndex,
    /// Progress posted on an activity whose finish-to-start predecessor is not
    /// finished.
    OutOfSequenceProgress,
    /// A successor reached by more than one path through the logic.
    RedundantLogic,
    /// A hard date constraint on an activity that is not on the critical path.
    UnnecessaryConstraints,
    /// A relationship with a lag longer than the long-lag threshold.
    LongLags,
}

impl CheckId {
    /// Stable code string, e.g. `AUDIT-LOGIC`.
    pub const fn code(self) -> &'static str {
        match self {
            Self::LogicOpenEnds => "AUDIT-LOGIC",
            Self::Leads => "AUDIT-LEADS",
            Self::Lags => "AUDIT-LAGS",
            Self::RelationshipTypes => "AUDIT-RELTYPES",
            Self::HardConstraints => "AUDIT-HARDCONST",
            Self::HighFloat => "AUDIT-HIGHFLOAT",
            Self::NegativeFloat => "AUDIT-NEGFLOAT",
            Self::HighDuration => "AUDIT-HIGHDUR",
            Self::InvalidDates => "AUDIT-INVALIDDATES",
            Self::MissingResources => "AUDIT-RESOURCES",
            Self::MissedTasks => "AUDIT-MISSED",
            Self::BaselineExecutionIndex => "AUDIT-BEI",
            Self::CriticalPathTest => "AUDIT-CPT",
            Self::CriticalPathLengthIndex => "AUDIT-CPLI",
            Self::OutOfSequenceProgress => "AUDIT-OOS",
            Self::RedundantLogic => "AUDIT-REDUNDANT",
            Self::UnnecessaryConstraints => "AUDIT-UNNECESSARY",
            Self::LongLags => "AUDIT-LONGLAG",
        }
    }

    /// Stable human title for the check.
    pub const fn default_title(self) -> &'static str {
        match self {
            Self::LogicOpenEnds => "Logic — open ends",
            Self::Leads => "Logic — leads",
            Self::Lags => "Logic — lags",
            Self::RelationshipTypes => "Logic — relationship types",
            Self::HardConstraints => "Constraints — hard constraints",
            Self::HighFloat => "Float — high float",
            Self::NegativeFloat => "Float — negative float",
            Self::HighDuration => "Structure — high duration",
            Self::InvalidDates => "Dates — invalid dates",
            Self::MissingResources => "Resources — missing resources",
            Self::MissedTasks => "Baseline — missed tasks",
            Self::BaselineExecutionIndex => "Baseline — execution index",
            Self::CriticalPathTest => "Critical path — critical path test",
            Self::CriticalPathLengthIndex => "Critical path — length index",
            Self::OutOfSequenceProgress => "Logic — out-of-sequence progress",
            Self::RedundantLogic => "Logic — redundant logic",
            Self::UnnecessaryConstraints => "Constraints — unnecessary constraints",
            Self::LongLags => "Logic — long lags",
        }
    }
}

/// Grouping of a check, for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckCategory {
    /// Structure: durations and level of detail.
    Structure,
    /// Logic: relationships and their types.
    Logic,
    /// Float.
    Float,
    /// Date constraints.
    Constraints,
    /// Dates and status.
    Dates,
    /// Resource assignments.
    Resources,
    /// Baseline performance.
    Baseline,
    /// Critical-path integrity.
    CriticalPath,
}

/// Result of one check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckOutcome {
    /// The programme is within the threshold.
    Pass,
    /// The programme is outside the threshold.
    Fail,
    /// The check cannot run with the data available.
    NotApplicable,
}

/// One item a check flagged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum Finding {
    /// An activity-level finding.
    Activity {
        /// Activity identifier.
        id: ActivityId,
        /// Activity name, copied for display.
        name: String,
        /// What the check found.
        detail: String,
    },
    /// A relationship-level finding.
    Relationship {
        /// Relationship identifier.
        id: RelationshipId,
        /// What the check found.
        detail: String,
    },
}

/// One check's result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    /// Stable identity of the check.
    pub id: CheckId,
    /// Display grouping.
    pub category: CheckCategory,
    /// Human title.
    pub title: String,
    /// Pass, fail or not applicable.
    pub outcome: CheckOutcome,
    /// Measured value, e.g. "3 of 40 open activities".
    pub measured: Option<String>,
    /// The limit the measurement is compared with.
    pub threshold: Option<String>,
    /// One plain sentence describing the result.
    pub summary: String,
    /// The flagged activities and relationships.
    pub findings: Vec<Finding>,
}

/// The whole audit, one entry per check in a fixed order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditReport {
    /// Check results, in catalog order.
    pub checks: Vec<CheckResult>,
}

impl AuditReport {
    /// Look up one check by identity.
    pub fn check(&self, id: CheckId) -> Option<&CheckResult> {
        self.checks.iter().find(|check| check.id == id)
    }
}

/// Run every check over a scheduled project.
///
/// `baseline` is the baseline to compare against (for example the most recent
/// one); checks that need a baseline report [`CheckOutcome::NotApplicable`]
/// when it is absent, and checks that need a status date do the same when the
/// configuration has none.
pub fn audit(
    project: &Project,
    schedule: &Schedule,
    baseline: Option<&Baseline>,
    config: &AuditConfig,
) -> AuditReport {
    let mut checks = vec![
        check_logic_open_ends(project, config),
        check_leads(project),
        check_lags(project, config),
        check_long_lags(project, config),
        check_relationship_types(project, config),
        check_redundant_logic(project),
        check_out_of_sequence_progress(project),
        check_hard_constraints(project, config),
        check_unnecessary_constraints(project, schedule, config),
        check_high_float(project, schedule, config),
        check_negative_float(project, schedule),
        check_high_duration(project, config),
        check_invalid_dates(project, schedule, config),
        check_missing_resources(project),
        check_missed_tasks(project, baseline, config),
        check_baseline_execution_index(project, baseline, config),
        check_critical_path_test(project, schedule, config),
        check_critical_path_length_index(project, schedule, baseline, config),
    ];
    for check in &mut checks {
        sort_findings(&mut check.findings);
    }
    AuditReport { checks }
}

/// True when an activity is not yet finished, by recorded progress or actual
/// finish.
fn is_open(activity: &Activity) -> bool {
    activity.actual_finish.is_none() && activity.percent_complete.unwrap_or(0) < 10_000
}

/// True when the activity is finished by the status date, by recorded progress
/// or an actual finish on or before it.
fn is_finished_by(activity: &Activity, status: Date) -> bool {
    activity.percent_complete.unwrap_or(0) >= 10_000
        || activity
            .actual_finish
            .is_some_and(|finish| finish <= status)
}

/// The project's activity with the given id.
fn activity(project: &Project, id: ActivityId) -> Option<&Activity> {
    project.activities.iter().find(|activity| activity.id == id)
}

/// Open activities, in project order.
fn open_activities(project: &Project) -> Vec<&Activity> {
    project
        .activities
        .iter()
        .filter(|activity| is_open(activity))
        .collect()
}

/// `count / total` in basis points, rounded down; zero when `total` is zero.
fn ratio_bp(count: usize, total: usize) -> u32 {
    if total == 0 {
        return 0;
    }
    let bp = count.saturating_mul(10_000) / total;
    u32::try_from(bp).unwrap_or(u32::MAX)
}

/// Format basis points as a percentage with one decimal place.
fn percent(bp: u32) -> String {
    format!("{:.1}%", f64::from(bp) / 100.0)
}

/// Sort findings by kind and identifier for a deterministic report.
fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by_key(|finding| match finding {
        Finding::Activity { id, .. } => (0u8, id.0, 0u64),
        Finding::Relationship { id, .. } => (1u8, 0, id.0),
    });
}

/// Build a check result.
fn make(
    id: CheckId,
    category: CheckCategory,
    outcome: CheckOutcome,
    measured: Option<String>,
    threshold: Option<String>,
    summary: String,
    findings: Vec<Finding>,
) -> CheckResult {
    CheckResult {
        id,
        category,
        title: id.default_title().to_string(),
        outcome,
        measured,
        threshold,
        summary,
        findings,
    }
}

/// A check that could not run, with the reason in the summary.
fn not_applicable(id: CheckId, category: CheckCategory, summary: &str) -> CheckResult {
    make(
        id,
        category,
        CheckOutcome::NotApplicable,
        None,
        None,
        summary.to_string(),
        Vec::new(),
    )
}

/// Outcome for a percentage check.
fn percentage_outcome(bp: u32, total: usize, limit: u32) -> CheckOutcome {
    if total == 0 {
        CheckOutcome::NotApplicable
    } else if bp > limit {
        CheckOutcome::Fail
    } else {
        CheckOutcome::Pass
    }
}

fn check_logic_open_ends(project: &Project, config: &AuditConfig) -> CheckResult {
    let open = open_activities(project);
    let mut findings = Vec::new();
    for activity in &open {
        let has_predecessor = project
            .relationships
            .iter()
            .any(|relationship| relationship.successor == activity.id);
        let has_successor = project
            .relationships
            .iter()
            .any(|relationship| relationship.predecessor == activity.id);
        // A zero-duration activity at a network boundary is a milestone: an
        // open start or open finish is legitimate for it, the same exemption
        // the DCMA assessment makes for project start and finish milestones.
        let milestone = activity.original_duration == 0;
        let missing_predecessor = !has_predecessor && !(milestone && has_successor);
        let missing_successor = !has_successor && !(milestone && has_predecessor);
        if missing_predecessor || missing_successor {
            let detail = if missing_predecessor && missing_successor {
                "no predecessor and no successor"
            } else if missing_predecessor {
                "no predecessor"
            } else {
                "no successor"
            };
            findings.push(Finding::Activity {
                id: activity.id,
                name: activity.name.clone(),
                detail: detail.to_string(),
            });
        }
    }
    let total = open.len();
    let bp = ratio_bp(findings.len(), total);
    let outcome = percentage_outcome(bp, total, config.percentage_limit_bp);
    make(
        CheckId::LogicOpenEnds,
        CheckCategory::Structure,
        outcome,
        Some(format!("{} of {} open activities", findings.len(), total)),
        Some(format!(
            "no more than {}",
            percent(config.percentage_limit_bp)
        )),
        format!(
            "{} of {total} open activities are missing a predecessor or successor ({}).",
            findings.len(),
            percent(bp)
        ),
        findings,
    )
}

fn check_leads(project: &Project) -> CheckResult {
    let mut findings = Vec::new();
    for relationship in &project.relationships {
        if relationship.lag >= 0 {
            continue;
        }
        if let Some(successor) = activity(project, relationship.successor) {
            if is_open(successor) {
                findings.push(Finding::Relationship {
                    id: relationship.id,
                    detail: format!(
                        "lead of {} working days into {}",
                        relationship.lag, successor.name
                    ),
                });
            }
        }
    }
    let outcome = if findings.is_empty() {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::Leads,
        CheckCategory::Logic,
        outcome,
        Some(format!("{} lead relationships", findings.len())),
        Some("none allowed".to_string()),
        format!(
            "{} relationships are leads (negative lag); leads distort float and the critical path.",
            findings.len()
        ),
        findings,
    )
}

fn check_lags(project: &Project, config: &AuditConfig) -> CheckResult {
    // Both the numerator and the denominator are every relationship in the
    // programme: the DCMA convention assesses the share of all relationships
    // that carry a lag, so the two populations must match rather than
    // comparing lagged open successors against all relationships.
    let mut findings = Vec::new();
    for relationship in &project.relationships {
        if relationship.lag <= 0 {
            continue;
        }
        let detail = match activity(project, relationship.successor) {
            Some(successor) => format!(
                "lag of {} working days into {}",
                relationship.lag, successor.name
            ),
            None => format!("lag of {} working days", relationship.lag),
        };
        findings.push(Finding::Relationship {
            id: relationship.id,
            detail,
        });
    }
    let total = project.relationships.len();
    let bp = ratio_bp(findings.len(), total);
    let outcome = percentage_outcome(bp, total, config.percentage_limit_bp);
    make(
        CheckId::Lags,
        CheckCategory::Logic,
        outcome,
        Some(format!("{} of {} relationships", findings.len(), total)),
        Some(format!(
            "no more than {}",
            percent(config.percentage_limit_bp)
        )),
        format!(
            "{} of {total} relationships carry a lag ({}); the share is counted over all relationships in the programme.",
            findings.len(),
            percent(bp)
        ),
        findings,
    )
}

fn check_relationship_types(project: &Project, config: &AuditConfig) -> CheckResult {
    let total = project.relationships.len();
    let finish_to_start = project
        .relationships
        .iter()
        .filter(|relationship| relationship.link_type == LinkType::Fs)
        .count();
    let share = ratio_bp(finish_to_start, total);
    let mut findings: Vec<Finding> = project
        .relationships
        .iter()
        .filter(|relationship| relationship.link_type != LinkType::Fs)
        .map(|relationship| Finding::Relationship {
            id: relationship.id,
            detail: format!(
                "linked with {} instead of FS",
                relationship.link_type.abbreviation()
            ),
        })
        .collect();
    sort_findings(&mut findings);
    let outcome = if total == 0 {
        CheckOutcome::NotApplicable
    } else if share < config.min_finish_to_start_bp {
        CheckOutcome::Fail
    } else {
        CheckOutcome::Pass
    };
    make(
        CheckId::RelationshipTypes,
        CheckCategory::Logic,
        outcome,
        Some(format!(
            "{finish_to_start} of {total} relationships are finish-to-start"
        )),
        Some(format!("at least {}", percent(config.min_finish_to_start_bp))),
        format!(
            "{} of {total} relationships are finish-to-start ({}); non-FS links complicate the critical path.",
            finish_to_start,
            percent(share)
        ),
        findings,
    )
}

/// True when the constraint forces a date the schedule cannot move past: the
/// DCMA hard-constraint set — no-later-than and must-on starts and finishes.
/// A no-earlier-than bound is soft and does not count.
fn is_hard_constraint(constraint: Constraint) -> bool {
    match constraint {
        Constraint::StartNoLaterThan(_)
        | Constraint::FinishNoLaterThan(_)
        | Constraint::MustStartOn(_)
        | Constraint::MustFinishOn(_) => true,
        Constraint::AsSoonAsPossible
        | Constraint::AsLateAsPossible
        | Constraint::StartNoEarlierThan(_)
        | Constraint::FinishNoEarlierThan(_) => false,
    }
}

fn check_hard_constraints(project: &Project, config: &AuditConfig) -> CheckResult {
    let open = open_activities(project);
    let mut findings = Vec::new();
    for activity in &open {
        if is_hard_constraint(activity.constraint) {
            findings.push(Finding::Activity {
                id: activity.id,
                name: activity.name.clone(),
                detail: format!("{:?}", activity.constraint),
            });
        }
    }
    let total = open.len();
    let bp = ratio_bp(findings.len(), total);
    let outcome = percentage_outcome(bp, total, config.percentage_limit_bp);
    make(
        CheckId::HardConstraints,
        CheckCategory::Constraints,
        outcome,
        Some(format!("{} of {} open activities", findings.len(), total)),
        Some(format!(
            "no more than {}",
            percent(config.percentage_limit_bp)
        )),
        format!(
            "{} of {total} open activities carry a hard date constraint ({}); hard constraints stop the schedule from moving.",
            findings.len(),
            percent(bp)
        ),
        findings,
    )
}

/// Unnecessary constraints: a hard date constraint on an open activity that is
/// not on the critical path, so the constraint is not needed to protect the
/// finish and only stops the schedule from moving.
///
/// The population is the open activities that carry a hard constraint, so a
/// programme with none reports [`CheckOutcome::NotApplicable`].
fn check_unnecessary_constraints(
    project: &Project,
    schedule: &Schedule,
    config: &AuditConfig,
) -> CheckResult {
    let mut constrained = 0usize;
    let mut findings = Vec::new();
    for activity in &project.activities {
        if !is_open(activity) || !is_hard_constraint(activity.constraint) {
            continue;
        }
        constrained += 1;
        let Some(result) = schedule.activity(activity.id) else {
            continue;
        };
        if result.total_float > 0 {
            findings.push(Finding::Activity {
                id: activity.id,
                name: activity.name.clone(),
                detail: format!(
                    "constraint {} with {} working days of float",
                    activity.constraint, result.total_float
                ),
            });
        }
    }
    let bp = ratio_bp(findings.len(), constrained);
    let outcome = percentage_outcome(bp, constrained, config.percentage_limit_bp);
    make(
        CheckId::UnnecessaryConstraints,
        CheckCategory::Constraints,
        outcome,
        Some(format!(
            "{} of {} hard-constrained activities",
            findings.len(),
            constrained
        )),
        Some(format!(
            "no more than {} on non-critical activities",
            percent(config.percentage_limit_bp)
        )),
        format!(
            "{} of {constrained} hard-constrained open activities are not critical ({}); the constraint is not needed to protect the finish.",
            findings.len(),
            percent(bp)
        ),
        findings,
    )
}

/// Long lags: a positive lag longer than the configured threshold. Long lags
/// delay the successor without representing real work and hide the true driver
/// of the dates.
fn check_long_lags(project: &Project, config: &AuditConfig) -> CheckResult {
    let mut findings = Vec::new();
    for relationship in &project.relationships {
        if relationship.lag <= config.long_lag_days {
            continue;
        }
        let detail = match activity(project, relationship.successor) {
            Some(successor) => format!(
                "lag of {} working days into {}",
                relationship.lag, successor.name
            ),
            None => format!("lag of {} working days", relationship.lag),
        };
        findings.push(Finding::Relationship {
            id: relationship.id,
            detail,
        });
    }
    let outcome = if findings.is_empty() {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::LongLags,
        CheckCategory::Logic,
        outcome,
        Some(format!("{} long lag relationships", findings.len())),
        Some(format!(
            "no lag above {} working days",
            config.long_lag_days
        )),
        format!(
            "{} relationships carry a lag longer than {} working days; a long lag delays the successor without representing work.",
            findings.len(),
            config.long_lag_days
        ),
        findings,
    )
}

/// Whether `start` reaches `target` through the logic, ignoring the single
/// relationship `skip`.
///
/// The walk is iterative, so a long programme cannot overflow the stack, and
/// the visited set bounds it to one visit per activity even when the network
/// contains a cycle.
fn reaches(
    adjacency: &HashMap<ActivityId, Vec<(RelationshipId, ActivityId)>>,
    start: ActivityId,
    target: ActivityId,
    skip: RelationshipId,
) -> bool {
    let mut visited: HashSet<ActivityId> = HashSet::new();
    let mut stack = vec![start];
    visited.insert(start);
    while let Some(node) = stack.pop() {
        let Some(edges) = adjacency.get(&node) else {
            continue;
        };
        for (id, next) in edges {
            if *id == skip {
                continue;
            }
            if *next == target {
                return true;
            }
            if visited.insert(*next) {
                stack.push(*next);
            }
        }
    }
    false
}

/// Redundant logic: the same successor is reached by more than one path, either
/// because the same predecessor is linked to it twice or because the
/// predecessor already reaches another predecessor of that successor. Redundant
/// links distort float and the critical path.
fn check_redundant_logic(project: &Project) -> CheckResult {
    // Outgoing edges for the reachability walk, and the incoming edges per
    // successor so the expensive walk runs only at merge points.
    let mut adjacency: HashMap<ActivityId, Vec<(RelationshipId, ActivityId)>> = HashMap::new();
    let mut incoming: HashMap<ActivityId, Vec<(RelationshipId, ActivityId)>> = HashMap::new();
    let mut pairs: HashMap<(ActivityId, ActivityId), Vec<RelationshipId>> = HashMap::new();
    for relationship in &project.relationships {
        adjacency
            .entry(relationship.predecessor)
            .or_default()
            .push((relationship.id, relationship.successor));
        incoming
            .entry(relationship.successor)
            .or_default()
            .push((relationship.id, relationship.predecessor));
        pairs
            .entry((relationship.predecessor, relationship.successor))
            .or_default()
            .push(relationship.id);
    }
    let mut redundant: BTreeSet<RelationshipId> = BTreeSet::new();
    // The same predecessor linked to the same successor more than once: every
    // link after the lowest id is a duplicate path.
    for ids in pairs.values() {
        let Some(&lowest) = ids.iter().min() else {
            continue;
        };
        for id in ids {
            if *id != lowest {
                redundant.insert(*id);
            }
        }
    }
    // Transitive redundancy: the predecessor already reaches another
    // predecessor of the same successor, so the direct link is a second path.
    for predecessors in incoming.values() {
        if predecessors.len() < 2 {
            continue;
        }
        for (id, predecessor) in predecessors {
            for (_, other) in predecessors {
                if *predecessor == *other {
                    continue;
                }
                if reaches(&adjacency, *predecessor, *other, *id) {
                    redundant.insert(*id);
                    break;
                }
            }
        }
    }
    let mut findings: Vec<Finding> = project
        .relationships
        .iter()
        .filter(|relationship| redundant.contains(&relationship.id))
        .map(|relationship| {
            let predecessor = activity(project, relationship.predecessor).map_or_else(
                || relationship.predecessor.to_string(),
                |item| item.name.clone(),
            );
            let successor = activity(project, relationship.successor).map_or_else(
                || relationship.successor.to_string(),
                |item| item.name.clone(),
            );
            Finding::Relationship {
                id: relationship.id,
                detail: format!("second path from {predecessor} into {successor}"),
            }
        })
        .collect();
    sort_findings(&mut findings);
    let outcome = if findings.is_empty() {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::RedundantLogic,
        CheckCategory::Logic,
        outcome,
        Some(format!("{} redundant relationships", findings.len())),
        Some("every relationship is the only path into its successor".to_string()),
        format!(
            "{} relationships are redundant: the same successor is reached by more than one path, which distorts float.",
            findings.len()
        ),
        findings,
    )
}

/// Out-of-sequence progress: an open activity that has recorded progress while
/// a finish-to-start predecessor is still unfinished. The report cites the
/// activity, never rewriting the recorded progress.
fn check_out_of_sequence_progress(project: &Project) -> CheckResult {
    let mut started = 0usize;
    let mut findings = Vec::new();
    for successor in &project.activities {
        let has_progress =
            successor.actual_start.is_some() || successor.percent_complete.unwrap_or(0) > 0;
        if !has_progress || !is_open(successor) {
            continue;
        }
        started += 1;
        let mut unfinished: Vec<ActivityId> = project
            .relationships
            .iter()
            .filter(|relationship| {
                relationship.successor == successor.id && relationship.link_type == LinkType::Fs
            })
            .filter(|relationship| activity(project, relationship.predecessor).is_some_and(is_open))
            .map(|relationship| relationship.predecessor)
            .collect();
        if unfinished.is_empty() {
            continue;
        }
        unfinished.sort_unstable();
        unfinished.dedup();
        let names = unfinished
            .iter()
            .map(|id| {
                activity(project, *id).map_or_else(|| id.to_string(), |item| item.name.clone())
            })
            .collect::<Vec<_>>()
            .join(", ");
        findings.push(Finding::Activity {
            id: successor.id,
            name: successor.name.clone(),
            detail: format!("started while {names} is not finished"),
        });
    }
    if started == 0 {
        return not_applicable(
            CheckId::OutOfSequenceProgress,
            CheckCategory::Logic,
            "No progress has been recorded, so out-of-sequence progress cannot be checked.",
        );
    }
    let outcome = if findings.is_empty() {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::OutOfSequenceProgress,
        CheckCategory::Logic,
        outcome,
        Some(format!("{} of {started} started activities", findings.len())),
        Some("progress only after the predecessor finishes".to_string()),
        format!(
            "{} of {started} activities with progress have an unfinished finish-to-start predecessor; the update was posted out of sequence.",
            findings.len()
        ),
        findings,
    )
}

fn check_high_float(project: &Project, schedule: &Schedule, config: &AuditConfig) -> CheckResult {
    let open = open_activities(project);
    let mut findings = Vec::new();
    for activity in &open {
        if let Some(result) = schedule.activity(activity.id) {
            if result.total_float > config.high_float_days {
                findings.push(Finding::Activity {
                    id: activity.id,
                    name: activity.name.clone(),
                    detail: format!("total float {} working days", result.total_float),
                });
            }
        }
    }
    let total = open.len();
    let bp = ratio_bp(findings.len(), total);
    let outcome = percentage_outcome(bp, total, config.percentage_limit_bp);
    make(
        CheckId::HighFloat,
        CheckCategory::Float,
        outcome,
        Some(format!("{} of {} open activities", findings.len(), total)),
        Some(format!(
            "no more than {} above {} working days",
            percent(config.percentage_limit_bp),
            config.high_float_days
        )),
        format!(
            "{} of {total} open activities have more than {} working days of float ({}).",
            findings.len(),
            config.high_float_days,
            percent(bp)
        ),
        findings,
    )
}

fn check_negative_float(project: &Project, schedule: &Schedule) -> CheckResult {
    if project.activities.is_empty() {
        return not_applicable(
            CheckId::NegativeFloat,
            CheckCategory::Float,
            "No activities to check.",
        );
    }
    // Negative float only matters for work still to be done: a completed
    // activity cannot be forecast to miss a date, so the check covers open
    // activities only, the same population as the high-float check.
    let open = open_activities(project);
    let mut findings = Vec::new();
    for activity in &open {
        if let Some(result) = schedule.activity(activity.id) {
            if result.total_float < 0 {
                findings.push(Finding::Activity {
                    id: activity.id,
                    name: activity.name.clone(),
                    detail: format!("total float {} working days", result.total_float),
                });
            }
        }
    }
    let outcome = if findings.is_empty() {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::NegativeFloat,
        CheckCategory::Float,
        outcome,
        Some(format!(
            "{} of {} open activities with negative float",
            findings.len(),
            open.len()
        )),
        Some("none allowed".to_string()),
        format!(
            "{} of {} open activities have negative float; the programme is forecast to miss a fixed date.",
            findings.len(),
            open.len()
        ),
        findings,
    )
}

fn check_high_duration(project: &Project, config: &AuditConfig) -> CheckResult {
    let with_duration: Vec<&Activity> = project
        .activities
        .iter()
        .filter(|activity| activity.original_duration > 0)
        .collect();
    let mut findings = Vec::new();
    for activity in &with_duration {
        if activity.original_duration > config.high_duration_days {
            findings.push(Finding::Activity {
                id: activity.id,
                name: activity.name.clone(),
                detail: format!("{} working days", activity.original_duration),
            });
        }
    }
    let total = with_duration.len();
    let bp = ratio_bp(findings.len(), total);
    let outcome = percentage_outcome(bp, total, config.percentage_limit_bp);
    make(
        CheckId::HighDuration,
        CheckCategory::Structure,
        outcome,
        Some(format!("{} of {} activities", findings.len(), total)),
        Some(format!(
            "no more than {} above {} working days",
            percent(config.percentage_limit_bp),
            config.high_duration_days
        )),
        format!(
            "{} of {total} activities are longer than {} working days ({}); long activities are hard to track.",
            findings.len(),
            config.high_duration_days,
            percent(bp)
        ),
        findings,
    )
}

fn check_invalid_dates(
    project: &Project,
    schedule: &Schedule,
    config: &AuditConfig,
) -> CheckResult {
    let Some(status) = config.status_date else {
        return not_applicable(
            CheckId::InvalidDates,
            CheckCategory::Dates,
            "No status date is set, so date checks are skipped.",
        );
    };
    let mut findings = Vec::new();
    for activity in &project.activities {
        if let Some(actual) = activity.actual_start {
            if actual > status {
                findings.push(Finding::Activity {
                    id: activity.id,
                    name: activity.name.clone(),
                    detail: format!("actual start {actual} is after the status date"),
                });
            }
        }
        if let Some(actual) = activity.actual_finish {
            if actual > status {
                findings.push(Finding::Activity {
                    id: activity.id,
                    name: activity.name.clone(),
                    detail: format!("actual finish {actual} is after the status date"),
                });
            }
        }
        if is_open(activity) {
            if let Some(result) = schedule.activity(activity.id) {
                // An activity that has started is legitimately in the past, so
                // its forecast start never counts as invalid. Only an activity
                // with no progress behind it must not be forecast to start
                // before the status date.
                let started =
                    activity.actual_start.is_some() || activity.percent_complete.unwrap_or(0) > 0;
                if !started && result.scheduled_start < status {
                    findings.push(Finding::Activity {
                        id: activity.id,
                        name: activity.name.clone(),
                        detail: format!(
                            "forecast start {} is before the status date",
                            result.scheduled_start
                        ),
                    });
                }
                // An open activity whose forecast finish has already slipped
                // behind the status date is stale regardless of progress
                // (DCMA point #9 also tests the forecast finish).
                if result.scheduled_finish < status {
                    findings.push(Finding::Activity {
                        id: activity.id,
                        name: activity.name.clone(),
                        detail: format!(
                            "forecast finish {} is before the status date",
                            result.scheduled_finish
                        ),
                    });
                }
            }
        }
    }
    let outcome = if findings.is_empty() {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::InvalidDates,
        CheckCategory::Dates,
        outcome,
        Some(format!("{} invalid dates", findings.len())),
        Some("no actual date after, or forecast date before, the status date".to_string()),
        format!(
            "{} dates are inconsistent with the status date {status}; actuals must not be in the future and forecasts must not be in the past.",
            findings.len()
        ),
        findings,
    )
}

fn check_missing_resources(project: &Project) -> CheckResult {
    if project.resources.is_empty() {
        return not_applicable(
            CheckId::MissingResources,
            CheckCategory::Resources,
            "The project has no resources loaded, so resource checks are skipped.",
        );
    }
    let mut findings = Vec::new();
    for activity in open_activities(project) {
        if activity.original_duration > 0 && activity.resources.is_empty() {
            findings.push(Finding::Activity {
                id: activity.id,
                name: activity.name.clone(),
                detail: "no resource assigned".to_string(),
            });
        }
    }
    let outcome = if findings.is_empty() {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::MissingResources,
        CheckCategory::Resources,
        outcome,
        Some(format!("{} activities missing a resource", findings.len())),
        Some("every open activity with a duration has a resource".to_string()),
        format!(
            "{} open activities with a duration have no resource assigned.",
            findings.len()
        ),
        findings,
    )
}

fn check_missed_tasks(
    project: &Project,
    baseline: Option<&Baseline>,
    config: &AuditConfig,
) -> CheckResult {
    let (Some(baseline), Some(status)) = (baseline, config.status_date) else {
        return not_applicable(
            CheckId::MissedTasks,
            CheckCategory::Baseline,
            "A baseline and a status date are needed to check for missed tasks.",
        );
    };
    let mut due = 0usize;
    let mut findings = Vec::new();
    for snapshot in &baseline.snapshots {
        let Some(baseline_finish) = snapshot.baseline_finish else {
            continue;
        };
        if baseline_finish > status {
            continue;
        }
        let Some(activity) = activity(project, snapshot.activity_id) else {
            continue;
        };
        due += 1;
        if is_open(activity) {
            findings.push(Finding::Activity {
                id: activity.id,
                name: activity.name.clone(),
                detail: format!("not finished by the baseline finish {baseline_finish}"),
            });
        } else if let Some(actual) = activity.actual_finish {
            if actual > baseline_finish {
                findings.push(Finding::Activity {
                    id: activity.id,
                    name: activity.name.clone(),
                    detail: format!(
                        "finished {actual}, after the baseline finish {baseline_finish}"
                    ),
                });
            }
        }
    }
    if due == 0 {
        return not_applicable(
            CheckId::MissedTasks,
            CheckCategory::Baseline,
            "No baseline activities were due by the status date.",
        );
    }
    let bp = ratio_bp(findings.len(), due);
    let outcome = percentage_outcome(bp, due, config.percentage_limit_bp);
    make(
        CheckId::MissedTasks,
        CheckCategory::Baseline,
        outcome,
        Some(format!("{} of {} due activities", findings.len(), due)),
        Some(format!(
            "no more than {}",
            percent(config.percentage_limit_bp)
        )),
        format!(
            "{} of {due} activities due by the status date are missed ({}).",
            findings.len(),
            percent(bp)
        ),
        findings,
    )
}

fn check_baseline_execution_index(
    project: &Project,
    baseline: Option<&Baseline>,
    config: &AuditConfig,
) -> CheckResult {
    let (Some(baseline), Some(status)) = (baseline, config.status_date) else {
        return not_applicable(
            CheckId::BaselineExecutionIndex,
            CheckCategory::Baseline,
            "A baseline and a status date are needed to calculate the execution index.",
        );
    };
    let mut due = 0usize;
    let mut completed = 0usize;
    for snapshot in &baseline.snapshots {
        let Some(baseline_finish) = snapshot.baseline_finish else {
            continue;
        };
        if baseline_finish > status {
            continue;
        }
        let Some(activity) = activity(project, snapshot.activity_id) else {
            continue;
        };
        due += 1;
        if is_finished_by(activity, status) {
            completed += 1;
        }
    }
    if due == 0 {
        return not_applicable(
            CheckId::BaselineExecutionIndex,
            CheckCategory::Baseline,
            "No baseline activities were due by the status date.",
        );
    }
    let bei = ratio_bp(completed, due);
    let outcome = if bei >= config.bei_min_bp {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::BaselineExecutionIndex,
        CheckCategory::Baseline,
        outcome,
        Some(format!("{completed} of {due} due activities complete")),
        Some(format!("at least {}", percent(config.bei_min_bp))),
        format!(
            "The baseline execution index is {} ({completed} of {due} due activities complete).",
            percent(bei)
        ),
        Vec::new(),
    )
}

/// The calendar that governs an activity's dates: its named calendar, or the
/// project default.
fn activity_calendar<'a>(project: &'a Project, activity: &Activity) -> &'a Calendar {
    match activity.calendar {
        None => &project.calendar,
        Some(id) => project.calendars.calendar(id).unwrap_or(&project.calendar),
    }
}

/// A critical-path check that cannot run.
fn critical_path_test_skip(summary: &str) -> CheckResult {
    not_applicable(
        CheckId::CriticalPathTest,
        CheckCategory::CriticalPath,
        summary,
    )
}

/// A critical-path index check that cannot run.
fn critical_path_index_skip(summary: &str) -> CheckResult {
    not_applicable(
        CheckId::CriticalPathLengthIndex,
        CheckCategory::CriticalPath,
        summary,
    )
}

fn check_critical_path_test(
    project: &Project,
    schedule: &Schedule,
    config: &AuditConfig,
) -> CheckResult {
    let Some((activity, result)) = project
        .activities
        .iter()
        .filter(|candidate| is_open(candidate))
        .filter_map(|candidate| {
            schedule
                .activity(candidate.id)
                .map(|scheduled| (candidate, scheduled))
        })
        .filter(|(_, scheduled)| scheduled.critical)
        .max_by_key(|(candidate, scheduled)| (scheduled.early_finish, candidate.id))
    else {
        return critical_path_test_skip("No open critical activities to test.");
    };
    let calendar = activity_calendar(project, activity);
    let nudge = i64::from(config.critical_path_nudge_days);
    let Ok(nudged_start) = calendar.add_working_days(result.scheduled_start, nudge) else {
        return critical_path_test_skip(
            "The activity's calendar cannot add the nudge, so the test cannot run.",
        );
    };
    let mut minimum_starts = HashMap::new();
    minimum_starts.insert(activity.id, nudged_start);
    let Ok(test) = schedule_with_minimum_starts(project, &minimum_starts) else {
        return critical_path_test_skip(
            "Rescheduling with the delayed activity failed, so the test cannot run.",
        );
    };
    let moved = calendar
        .working_days_between(schedule.project_finish, test.project_finish)
        .abs();
    let outcome = if moved == nudge {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::CriticalPathTest,
        CheckCategory::CriticalPath,
        outcome,
        Some(format!(
            "delaying {} by {nudge} working days moves the finish {moved}",
            activity.name
        )),
        Some(format!("the finish moves by all {nudge} working days")),
        format!(
            "Delaying the last-finishing critical activity ({}) by {nudge} working days \
             moves the project finish by {moved}; a full move proves the critical path \
             is driving the finish.",
            activity.name
        ),
        Vec::new(),
    )
}

fn check_critical_path_length_index(
    project: &Project,
    schedule: &Schedule,
    baseline: Option<&Baseline>,
    config: &AuditConfig,
) -> CheckResult {
    let Some(baseline) = baseline else {
        return critical_path_index_skip(
            "A baseline provides the deadline the index measures against.",
        );
    };
    let deadline = baseline
        .snapshots
        .iter()
        .filter_map(|snapshot| snapshot.baseline_finish)
        .max();
    let Some(deadline) = deadline else {
        return critical_path_index_skip(
            "The baseline has no finish dates, so the index cannot run.",
        );
    };
    let calendar = &project.calendar;
    let path = calendar
        .working_days_between(schedule.project_start, schedule.project_finish)
        .saturating_add(1);
    if path <= 0 {
        return critical_path_index_skip(
            "The programme spans no working days, so the index cannot run.",
        );
    }
    let float = calendar.working_days_between(schedule.project_finish, deadline);
    let index = (path + float).saturating_mul(10_000) / path;
    let shown = u32::try_from(index.max(0)).unwrap_or(u32::MAX);
    let outcome = if index >= i64::from(config.cpli_min_bp) {
        CheckOutcome::Pass
    } else {
        CheckOutcome::Fail
    };
    make(
        CheckId::CriticalPathLengthIndex,
        CheckCategory::CriticalPath,
        outcome,
        Some(format!(
            "critical path {path} working days, float to the deadline {float}"
        )),
        Some(format!("at least {}", percent(config.cpli_min_bp))),
        format!(
            "The critical path length index is {} against the baseline deadline \
             {deadline}: the path plus its float, over the path length.",
            percent(shown)
        ),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::schedule;
    use crate::model::{ActivityBaselineSnapshot, BaselineId, Relationship, Resource, ResourceId};
    use time::Month;

    fn jan(day: u8) -> Date {
        Date::from_calendar_date(2026, Month::January, day).expect("valid test date")
    }

    fn project() -> Project {
        Project::new("Audit test", jan(5))
    }

    fn add_activity(project: &mut Project, id: u64, name: &str, duration: u32) -> ActivityId {
        let activity_id = ActivityId::new(id);
        project
            .activities
            .push(Activity::new(activity_id, name, duration));
        activity_id
    }

    fn add_relationship(
        project: &mut Project,
        id: u64,
        predecessor: ActivityId,
        successor: ActivityId,
        link_type: LinkType,
        lag: i64,
    ) {
        project.relationships.push(Relationship::new(
            RelationshipId::new(id),
            predecessor,
            successor,
            link_type,
            lag,
        ));
    }

    fn run(project: &Project) -> AuditReport {
        run_with(project, None, &AuditConfig::default())
    }

    fn run_with(
        project: &Project,
        baseline: Option<&Baseline>,
        config: &AuditConfig,
    ) -> AuditReport {
        let schedule = schedule(project).expect("schedule succeeds");
        audit(project, &schedule, baseline, config)
    }

    fn outcome(report: &AuditReport, id: CheckId) -> CheckOutcome {
        report.check(id).expect("check present").outcome
    }

    #[test]
    fn a_clean_project_passes_the_structure_checks() {
        let mut project = project();
        let start = add_activity(&mut project, 1, "Start", 0);
        let a = add_activity(&mut project, 2, "A", 3);
        let b = add_activity(&mut project, 3, "B", 3);
        let finish = add_activity(&mut project, 4, "Finish", 0);
        add_relationship(&mut project, 1, start, a, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 3, b, finish, LinkType::Fs, 0);

        let report = run(&project);

        assert_eq!(outcome(&report, CheckId::LogicOpenEnds), CheckOutcome::Pass);
        assert_eq!(outcome(&report, CheckId::Leads), CheckOutcome::Pass);
        assert_eq!(outcome(&report, CheckId::Lags), CheckOutcome::Pass);
        assert_eq!(
            outcome(&report, CheckId::RelationshipTypes),
            CheckOutcome::Pass
        );
        assert_eq!(
            outcome(&report, CheckId::HardConstraints),
            CheckOutcome::Pass
        );
        assert_eq!(outcome(&report, CheckId::LongLags), CheckOutcome::Pass);
        assert_eq!(
            outcome(&report, CheckId::RedundantLogic),
            CheckOutcome::Pass
        );
        assert_eq!(
            outcome(&report, CheckId::OutOfSequenceProgress),
            CheckOutcome::NotApplicable
        );
        assert_eq!(
            outcome(&report, CheckId::UnnecessaryConstraints),
            CheckOutcome::NotApplicable
        );
        assert_eq!(outcome(&report, CheckId::HighFloat), CheckOutcome::Pass);
        assert_eq!(outcome(&report, CheckId::NegativeFloat), CheckOutcome::Pass);
        assert_eq!(outcome(&report, CheckId::HighDuration), CheckOutcome::Pass);
        assert_eq!(
            outcome(&report, CheckId::MissingResources),
            CheckOutcome::NotApplicable
        );
        assert_eq!(
            outcome(&report, CheckId::MissedTasks),
            CheckOutcome::NotApplicable
        );
        assert_eq!(
            outcome(&report, CheckId::BaselineExecutionIndex),
            CheckOutcome::NotApplicable
        );
    }

    #[test]
    fn an_open_end_fails_the_logic_check() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 2);
        let c = add_activity(&mut project, 3, "C", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);

        let report = run(&project);

        assert_eq!(outcome(&report, CheckId::LogicOpenEnds), CheckOutcome::Fail);
        let check = report.check(CheckId::LogicOpenEnds).expect("check present");
        assert!(check
            .findings
            .iter()
            .any(|finding| matches!(finding, Finding::Activity { id, .. } if *id == c)));
    }

    #[test]
    fn a_negative_lag_fails_the_leads_check() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 4);
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, -1);

        let report = run(&project);

        assert_eq!(outcome(&report, CheckId::Leads), CheckOutcome::Fail);
    }

    #[test]
    fn too_many_lags_fail_after_the_threshold() {
        let mut project = project();
        let ids: Vec<ActivityId> = (1..=6)
            .map(|id| add_activity(&mut project, id, "A", 1))
            .collect();
        for window in ids.windows(2) {
            add_relationship(
                &mut project,
                window[0].0,
                window[0],
                window[1],
                LinkType::Fs,
                0,
            );
        }
        project.relationships[0].lag = 1;

        let report = run(&project);

        assert_eq!(outcome(&report, CheckId::Lags), CheckOutcome::Fail);
    }

    #[test]
    fn non_finish_to_start_relationships_fail_the_type_check() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 3);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Ss, 0);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::RelationshipTypes),
            CheckOutcome::Fail
        );
    }

    #[test]
    fn a_hard_constraint_fails_the_constraints_check() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        project.activities[0].constraint = Constraint::FinishNoLaterThan(jan(30));

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::HardConstraints),
            CheckOutcome::Fail
        );
        let check = report
            .check(CheckId::HardConstraints)
            .expect("check present");
        assert!(check
            .findings
            .iter()
            .any(|finding| matches!(finding, Finding::Activity { id, .. } if *id == a)));
    }

    #[test]
    fn negative_float_fails_the_float_check() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 2);
        let mut schedule = schedule(&project).expect("schedule succeeds");
        schedule.activities[0].total_float = -3;

        let check = check_negative_float(&project, &schedule);

        assert_eq!(check.outcome, CheckOutcome::Fail);
    }

    #[test]
    fn a_completed_activity_with_negative_float_does_not_fail_the_float_check() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 2);
        project.activities[0].percent_complete = Some(10_000);
        let mut schedule = schedule(&project).expect("schedule succeeds");
        schedule.activities[0].total_float = -3;

        let check = check_negative_float(&project, &schedule);

        assert_eq!(check.outcome, CheckOutcome::Pass);
    }

    #[test]
    fn a_long_activity_fails_the_duration_check() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 50);

        let report = run(&project);

        assert_eq!(outcome(&report, CheckId::HighDuration), CheckOutcome::Fail);
    }

    #[test]
    fn missing_resources_fail_the_resources_check() {
        let mut project = project();
        project
            .resources
            .push(Resource::new(ResourceId::new(1), "Crew"));
        add_activity(&mut project, 1, "A", 2);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::MissingResources),
            CheckOutcome::Fail
        );
    }

    #[test]
    fn no_resources_makes_the_check_not_applicable() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 2);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::MissingResources),
            CheckOutcome::NotApplicable
        );
    }

    #[test]
    fn invalid_dates_fail_when_a_status_date_is_set() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 5);
        let config = AuditConfig {
            status_date: Some(jan(8)),
            ..AuditConfig::default()
        };

        let report = run_with(&project, None, &config);

        assert_eq!(outcome(&report, CheckId::InvalidDates), CheckOutcome::Fail);
    }

    #[test]
    fn the_date_check_is_not_applicable_without_a_status_date() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 5);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::InvalidDates),
            CheckOutcome::NotApplicable
        );
    }

    #[test]
    fn an_in_progress_programme_passes_the_invalid_dates_check() {
        let mut project = project();
        // A duration-10 activity started on the project start (Monday 5
        // January) is still running at the status date (Thursday 8 January):
        // its forecast start is behind the status date only because the engine
        // ignores actual dates, so it must not be flagged.
        add_activity(&mut project, 1, "A", 10);
        project.activities[0].actual_start = Some(jan(5));
        project.activities[0].percent_complete = Some(5_000);
        let config = AuditConfig {
            status_date: Some(jan(8)),
            ..AuditConfig::default()
        };

        let report = run_with(&project, None, &config);

        assert_eq!(outcome(&report, CheckId::InvalidDates), CheckOutcome::Pass);
    }

    #[test]
    fn a_stale_forecast_finish_fails_the_invalid_dates_check() {
        let mut project = project();
        // A duration-2 activity started on 5 January finishes on 6 January but
        // is still open at the status date of 20 January: its forecast finish
        // is stale and must be flagged even though it has started.
        add_activity(&mut project, 1, "A", 2);
        project.activities[0].actual_start = Some(jan(5));
        project.activities[0].percent_complete = Some(5_000);
        let config = AuditConfig {
            status_date: Some(jan(20)),
            ..AuditConfig::default()
        };

        let report = run_with(&project, None, &config);

        assert_eq!(outcome(&report, CheckId::InvalidDates), CheckOutcome::Fail);
        let check = report.check(CheckId::InvalidDates).expect("check present");
        assert!(check.findings.iter().any(|finding| matches!(
            finding,
            Finding::Activity { detail, .. } if detail.contains("forecast finish")
        )));
    }

    #[test]
    fn missed_tasks_and_bei_use_the_baseline() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 3);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        let baseline = Baseline::from_snapshots(
            BaselineId::new(1),
            "Baseline",
            jan(5),
            vec![ActivityBaselineSnapshot {
                activity_id: a,
                baseline_duration: 3,
                baseline_start: Some(jan(5)),
                baseline_finish: Some(jan(7)),
            }],
        );
        let config = AuditConfig {
            status_date: Some(jan(12)),
            ..AuditConfig::default()
        };

        let report = run_with(&project, Some(&baseline), &config);

        assert_eq!(outcome(&report, CheckId::MissedTasks), CheckOutcome::Fail);
        assert_eq!(
            outcome(&report, CheckId::BaselineExecutionIndex),
            CheckOutcome::Fail
        );
    }

    #[test]
    fn completed_work_passes_missed_tasks_and_bei() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 3);
        project.activities[0].percent_complete = Some(10_000);
        project.activities[0].actual_finish = Some(jan(7));
        let baseline = Baseline::from_snapshots(
            BaselineId::new(1),
            "Baseline",
            jan(5),
            vec![ActivityBaselineSnapshot {
                activity_id: a,
                baseline_duration: 3,
                baseline_start: Some(jan(5)),
                baseline_finish: Some(jan(7)),
            }],
        );
        let config = AuditConfig {
            status_date: Some(jan(12)),
            ..AuditConfig::default()
        };

        let report = run_with(&project, Some(&baseline), &config);

        assert_eq!(outcome(&report, CheckId::MissedTasks), CheckOutcome::Pass);
        assert_eq!(
            outcome(&report, CheckId::BaselineExecutionIndex),
            CheckOutcome::Pass
        );
    }

    #[test]
    fn the_critical_path_test_passes_on_a_driving_chain() {
        let mut project = project();
        let start = add_activity(&mut project, 1, "Start", 0);
        let a = add_activity(&mut project, 2, "A", 3);
        let b = add_activity(&mut project, 3, "B", 3);
        let finish = add_activity(&mut project, 4, "Finish", 0);
        add_relationship(&mut project, 1, start, a, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 3, b, finish, LinkType::Fs, 0);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::CriticalPathTest),
            CheckOutcome::Pass
        );
    }

    #[test]
    fn the_length_index_needs_a_baseline() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 2);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::CriticalPathLengthIndex),
            CheckOutcome::NotApplicable
        );
    }

    #[test]
    fn the_length_index_passes_when_the_deadline_matches() {
        let mut project = project();
        let start = add_activity(&mut project, 1, "Start", 0);
        let a = add_activity(&mut project, 2, "A", 3);
        let b = add_activity(&mut project, 3, "B", 3);
        let finish = add_activity(&mut project, 4, "Finish", 0);
        add_relationship(&mut project, 1, start, a, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 3, b, finish, LinkType::Fs, 0);
        let baseline = baseline_from_schedule(&project);

        let report = run_with(&project, Some(&baseline), &AuditConfig::default());

        assert_eq!(
            outcome(&report, CheckId::CriticalPathLengthIndex),
            CheckOutcome::Pass
        );
    }

    #[test]
    fn a_missed_deadline_fails_the_length_index() {
        let mut project = project();
        let start = add_activity(&mut project, 1, "Start", 0);
        let a = add_activity(&mut project, 2, "A", 3);
        let b = add_activity(&mut project, 3, "B", 3);
        let finish = add_activity(&mut project, 4, "Finish", 0);
        add_relationship(&mut project, 1, start, a, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 3, b, finish, LinkType::Fs, 0);
        let schedule = schedule(&project).expect("schedule succeeds");
        let snapshots = project
            .activities
            .iter()
            .filter_map(|activity| {
                let scheduled = schedule.activity(activity.id)?;
                Some(ActivityBaselineSnapshot {
                    activity_id: activity.id,
                    baseline_duration: activity.original_duration,
                    baseline_start: Some(scheduled.scheduled_start),
                    baseline_finish: project
                        .calendar
                        .add_working_days(scheduled.scheduled_finish, -5)
                        .ok(),
                })
            })
            .collect();
        let baseline = Baseline::from_snapshots(BaselineId::new(1), "Baseline", jan(5), snapshots);

        let report = run_with(&project, Some(&baseline), &AuditConfig::default());

        assert_eq!(
            outcome(&report, CheckId::CriticalPathLengthIndex),
            CheckOutcome::Fail
        );
    }

    /// A baseline that records the programme exactly as it is scheduled now.
    fn baseline_from_schedule(project: &Project) -> Baseline {
        let schedule = schedule(project).expect("schedule succeeds");
        let snapshots = project
            .activities
            .iter()
            .filter_map(|activity| {
                let scheduled = schedule.activity(activity.id)?;
                Some(ActivityBaselineSnapshot {
                    activity_id: activity.id,
                    baseline_duration: activity.original_duration,
                    baseline_start: Some(scheduled.scheduled_start),
                    baseline_finish: Some(scheduled.scheduled_finish),
                })
            })
            .collect();
        Baseline::from_snapshots(BaselineId::new(1), "Baseline", jan(5), snapshots)
    }

    #[test]
    fn a_must_start_on_constraint_fails_the_hard_constraints_check() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 2);
        project.activities[0].constraint = Constraint::MustStartOn(jan(6));

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::HardConstraints),
            CheckOutcome::Fail
        );
    }

    #[test]
    fn a_long_lag_fails_the_long_lag_check() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 3);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 10);

        let report = run(&project);

        assert_eq!(outcome(&report, CheckId::LongLags), CheckOutcome::Fail);
        let check = report.check(CheckId::LongLags).expect("check present");
        assert!(check.findings.iter().any(|finding| matches!(
            finding,
            Finding::Relationship { id, .. } if *id == RelationshipId::new(1)
        )));
    }

    #[test]
    fn a_short_lag_passes_the_long_lag_check() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 3);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 2);

        let report = run(&project);

        assert_eq!(outcome(&report, CheckId::LongLags), CheckOutcome::Pass);
    }

    #[test]
    fn redundant_logic_fails_when_a_successor_has_two_paths() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 2);
        let c = add_activity(&mut project, 3, "C", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 2, b, c, LinkType::Fs, 0);
        // A already reaches C through B, so this direct link is a second path.
        add_relationship(&mut project, 3, a, c, LinkType::Fs, 0);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::RedundantLogic),
            CheckOutcome::Fail
        );
        let check = report
            .check(CheckId::RedundantLogic)
            .expect("check present");
        assert_eq!(check.findings.len(), 1);
        assert!(check.findings.iter().any(|finding| matches!(
            finding,
            Finding::Relationship { id, .. } if *id == RelationshipId::new(3)
        )));
    }

    #[test]
    fn duplicate_relationships_fail_the_redundant_logic_check() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, b, LinkType::Fs, 0);

        let check = check_redundant_logic(&project);

        assert_eq!(check.outcome, CheckOutcome::Fail);
        assert_eq!(check.findings.len(), 1);
        assert!(check.findings.iter().any(|finding| matches!(
            finding,
            Finding::Relationship { id, .. } if *id == RelationshipId::new(2)
        )));
    }

    #[test]
    fn a_clean_chain_has_no_redundant_logic() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 2);
        let c = add_activity(&mut project, 3, "C", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 2, b, c, LinkType::Fs, 0);

        let check = check_redundant_logic(&project);

        assert_eq!(check.outcome, CheckOutcome::Pass);
        assert!(check.findings.is_empty());
    }

    #[test]
    fn out_of_sequence_progress_fails_when_a_successor_starts_first() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 5);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        // B has progress while its finish-to-start predecessor A is unfinished.
        project.activities[1].actual_start = Some(jan(6));
        project.activities[1].percent_complete = Some(5_000);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::OutOfSequenceProgress),
            CheckOutcome::Fail
        );
        let check = report
            .check(CheckId::OutOfSequenceProgress)
            .expect("check present");
        assert!(check.findings.iter().any(|finding| matches!(
            finding,
            Finding::Activity { id, .. } if *id == b
        )));
    }

    #[test]
    fn out_of_sequence_progress_passes_when_the_predecessor_is_finished() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 5);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);
        project.activities[0].percent_complete = Some(10_000);
        project.activities[0].actual_finish = Some(jan(9));
        project.activities[1].actual_start = Some(jan(12));
        project.activities[1].percent_complete = Some(5_000);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::OutOfSequenceProgress),
            CheckOutcome::Pass
        );
    }

    #[test]
    fn out_of_sequence_progress_is_not_applicable_without_progress() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 2);
        let b = add_activity(&mut project, 2, "B", 2);
        add_relationship(&mut project, 1, a, b, LinkType::Fs, 0);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::OutOfSequenceProgress),
            CheckOutcome::NotApplicable
        );
    }

    #[test]
    fn out_of_sequence_progress_ignores_overlapping_links() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 5);
        let b = add_activity(&mut project, 2, "B", 5);
        add_relationship(&mut project, 1, a, b, LinkType::Ss, 0);
        // A start-to-start link is meant to overlap, so B starting while A runs
        // is normal, not out of sequence.
        project.activities[1].actual_start = Some(jan(6));
        project.activities[1].percent_complete = Some(5_000);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::OutOfSequenceProgress),
            CheckOutcome::Pass
        );
    }

    #[test]
    fn a_hard_constraint_on_a_non_critical_activity_is_unnecessary() {
        let mut project = project();
        let start = add_activity(&mut project, 1, "Start", 0);
        let a = add_activity(&mut project, 2, "A", 3);
        let b = add_activity(&mut project, 3, "B", 1);
        let finish = add_activity(&mut project, 4, "Finish", 0);
        add_relationship(&mut project, 1, start, a, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, finish, LinkType::Fs, 0);
        add_relationship(&mut project, 3, start, b, LinkType::Fs, 0);
        add_relationship(&mut project, 4, b, finish, LinkType::Fs, 0);
        // B has float (the A branch is longer), so a finish-no-later-than on it
        // is not needed to protect the finish.
        project.activities[2].constraint = Constraint::FinishNoLaterThan(jan(30));

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::UnnecessaryConstraints),
            CheckOutcome::Fail
        );
        let check = report
            .check(CheckId::UnnecessaryConstraints)
            .expect("check present");
        assert!(check.findings.iter().any(|finding| matches!(
            finding,
            Finding::Activity { id, .. } if *id == b
        )));
    }

    #[test]
    fn the_unnecessary_constraint_check_is_not_applicable_without_hard_constraints() {
        let mut project = project();
        add_activity(&mut project, 1, "A", 2);

        let report = run(&project);

        assert_eq!(
            outcome(&report, CheckId::UnnecessaryConstraints),
            CheckOutcome::NotApplicable
        );
    }

    #[test]
    fn the_report_survives_a_serde_roundtrip() {
        let mut project = project();
        let start = add_activity(&mut project, 1, "Start", 0);
        let a = add_activity(&mut project, 2, "A", 3);
        let b = add_activity(&mut project, 3, "B", 3);
        let finish = add_activity(&mut project, 4, "Finish", 0);
        add_relationship(&mut project, 1, start, a, LinkType::Fs, 0);
        add_relationship(&mut project, 2, a, b, LinkType::Fs, 0);
        add_relationship(&mut project, 3, b, finish, LinkType::Fs, 0);
        let report = run(&project);

        let json = serde_json::to_string(&report).expect("serialise report");
        let decoded: AuditReport = serde_json::from_str(&json).expect("deserialise report");

        assert_eq!(decoded, report);
    }

    #[test]
    fn the_new_check_ids_round_trip_through_serde() {
        for id in [
            CheckId::OutOfSequenceProgress,
            CheckId::RedundantLogic,
            CheckId::UnnecessaryConstraints,
            CheckId::LongLags,
        ] {
            let json = serde_json::to_string(&id).expect("serialise check id");
            let decoded: CheckId = serde_json::from_str(&json).expect("deserialise check id");
            assert_eq!(decoded, id);
        }
    }

    #[test]
    fn the_new_check_ids_have_stable_codes_and_names() {
        assert_eq!(CheckId::OutOfSequenceProgress.code(), "AUDIT-OOS");
        assert_eq!(CheckId::RedundantLogic.code(), "AUDIT-REDUNDANT");
        assert_eq!(CheckId::UnnecessaryConstraints.code(), "AUDIT-UNNECESSARY");
        assert_eq!(CheckId::LongLags.code(), "AUDIT-LONGLAG");
        assert_eq!(
            serde_json::to_string(&CheckId::LongLags).expect("serialise"),
            "\"long-lags\""
        );
        assert!(CheckId::OutOfSequenceProgress
            .default_title()
            .contains("out-of-sequence"));
    }

    #[test]
    fn the_config_round_trips_through_serde() {
        let config = AuditConfig {
            status_date: Some(jan(8)),
            long_lag_days: 7,
            ..AuditConfig::default()
        };

        let json = serde_json::to_string(&config).expect("serialise config");
        let decoded: AuditConfig = serde_json::from_str(&json).expect("deserialise config");

        assert_eq!(decoded, config);
    }

    #[test]
    fn the_long_lag_default_applies_to_a_config_without_the_field() {
        let json = r#"{"status_date":null,"percentage_limit_bp":500,"high_float_days":44,
            "high_duration_days":44,"min_finish_to_start_bp":9000,"bei_min_bp":9500,
            "cpli_min_bp":9500,"critical_path_nudge_days":600}"#;

        let config: AuditConfig = serde_json::from_str(json).expect("deserialise legacy config");

        assert_eq!(config.long_lag_days, default_long_lag_days());
    }

    #[test]
    fn the_report_is_deterministic() {
        let mut project = project();
        let a = add_activity(&mut project, 1, "A", 3);
        let b = add_activity(&mut project, 2, "B", 3);
        add_relationship(&mut project, 1, a, b, LinkType::Ss, 2);
        add_activity(&mut project, 3, "C", 2);

        let first = run(&project);
        let second = run(&project);

        assert_eq!(first, second);
    }
}
