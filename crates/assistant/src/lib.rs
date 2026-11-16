//! Cadence's optional AI assistant: the provider-agnostic foundation.
//!
//! The assistant is **off by default**. Nothing in this crate performs I/O or
//! reaches the network; it defines the contract a model backend implements, the
//! typed shape a suggestion takes, and the validation that runs before a
//! suggestion can be shown to the user.
//!
//! Three rules shape the design:
//!
//! * **The model proposes, the planner disposes.** A response becomes a
//!   [`Proposal`] of typed, reviewable [`ProposedChange`] values. Nothing is
//!   applied until the user accepts it through the ordinary command layer, so
//!   undo and redo behave exactly as for a manual edit.
//! * **Every proposal is validated against the live project first.**
//!   [`Proposal::validate`] returns *all* problems at once — unknown
//!   activities, out-of-range progress, self- and duplicate relationships,
//!   blank names, milestone-only zero durations.
//! * **No secret is ever printed.** [`SecretKey`] redacts itself in `Debug` and
//!   serialises as `"<redacted>"`, so a settings file, log line or crash report
//!   cannot carry it.

mod config;
mod proposal;
mod provider;

pub use config::{AssistantConfig, SecretKey};
pub use proposal::{Proposal, ProposalError, ProposedChange};
pub use provider::{
    AssistantError, AssistantProvider, AssistantProviderName, MockProvider, NoopProvider,
};

use serde::{Deserialize, Serialize};

/// A model response as the UI first receives it, before review.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistantSuggestion {
    /// The model's explanation.
    pub prose: String,
    /// The name of the model that produced it, when known.
    pub model_name: Option<String>,
}

/// Whether the assistant is enabled *and* fully configured.
///
/// The UI uses this to decide between showing the assistant and hiding it.
pub fn is_assistant_enabled(config: &AssistantConfig) -> bool {
    config.is_configured()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadence_core::model::{
        Activity, ActivityId, LinkType, Project, Relationship, RelationshipId,
    };
    use time::{Date, Month};

    fn start_date() -> Date {
        Date::from_calendar_date(2026, Month::January, 5).expect("a valid date")
    }

    fn project_with_two_activities() -> Project {
        let mut project = Project::new("Test programme", start_date());
        let write = Activity::new(ActivityId::new(1), "Write chapter", 1);
        let review = Activity::new(ActivityId::new(2), "Review chapter", 2);
        project.activities.push(write);
        project.activities.push(review);
        project
    }

    fn proposal_with(changes: Vec<ProposedChange>) -> Proposal {
        Proposal::new("Synthetic proposal.", changes).expect("well formed")
    }

    #[test]
    fn config_is_off_by_default() {
        let config = AssistantConfig::default();
        assert!(!config.enabled);
        assert!(!config.is_configured());
        assert!(!is_assistant_enabled(&config));
    }

    #[test]
    fn config_is_configured_only_when_every_field_is_set() {
        let full = AssistantConfig {
            enabled: true,
            provider_name: Some("openai".to_owned()),
            api_key: Some(SecretKey::new("not-a-real-credential")),
        };
        assert!(full.is_configured());

        let no_provider = AssistantConfig {
            provider_name: None,
            ..full.clone()
        };
        assert!(!no_provider.is_configured());

        let no_key = AssistantConfig {
            api_key: None,
            ..full
        };
        assert!(!no_key.is_configured());
    }

    #[test]
    fn api_key_never_appears_in_debug_or_serialised_output() {
        let key = SecretKey::new("not-a-real-credential");
        assert_eq!(format!("{key:?}"), "SecretKey(\"**redacted**\")");
        let json = serde_json::to_string(&key).expect("serialise");
        assert!(!json.contains("not-a-real-credential"));
        assert_eq!(json, "\"<redacted>\"");
    }

    #[test]
    fn a_configured_config_does_not_leak_its_key_when_serialised() {
        let config = AssistantConfig {
            enabled: true,
            provider_name: Some("openai".to_owned()),
            api_key: Some(SecretKey::new("not-a-real-credential")),
        };
        let json = serde_json::to_string(&config).expect("serialise");
        assert!(!json.contains("not-a-real-credential"));
        assert!(!format!("{config:?}").contains("not-a-real-credential"));
    }

    #[test]
    fn valid_proposals_pass_validation() {
        let project = project_with_two_activities();
        let duration = ProposedChange::SetActivityDuration {
            activity: ActivityId::new(1),
            duration_days: 2,
            rationale: "The first draft is already complete.".to_owned(),
        };
        let link = ProposedChange::CreateRelationship {
            predecessor: ActivityId::new(1),
            successor: ActivityId::new(2),
            rationale: "Review cannot start before writing.".to_owned(),
        };
        let proposal = proposal_with(vec![duration, link]);
        assert!(proposal.validate(&project).is_empty());
    }

    #[test]
    fn unknown_activity_reference_is_rejected() {
        let project = project_with_two_activities();
        let change = ProposedChange::SetActivityDuration {
            activity: ActivityId::new(99),
            duration_days: 5,
            rationale: "Synthetic.".to_owned(),
        };
        let proposal = proposal_with(vec![change]);
        let errors = proposal.validate(&project);
        let expected = ProposalError::UnknownActivity {
            id: ActivityId::new(99),
        };
        assert_eq!(errors, vec![expected]);
    }

    #[test]
    fn percent_above_100_is_rejected() {
        let project = project_with_two_activities();
        let change = ProposedChange::SetActivityProgress {
            activity: ActivityId::new(1),
            percent: 110,
            rationale: "Synthetic.".to_owned(),
        };
        let proposal = proposal_with(vec![change]);
        let errors = proposal.validate(&project);
        let expected = ProposalError::PercentOutOfRange { percent: 110 };
        assert_eq!(errors, vec![expected]);
    }

    #[test]
    fn self_relationship_is_rejected() {
        let project = project_with_two_activities();
        let change = ProposedChange::CreateRelationship {
            predecessor: ActivityId::new(1),
            successor: ActivityId::new(1),
            rationale: "Synthetic.".to_owned(),
        };
        let proposal = proposal_with(vec![change]);
        let expected = ProposalError::SelfRelationship {
            id: ActivityId::new(1),
        };
        assert!(proposal.validate(&project).contains(&expected));
    }

    #[test]
    fn duplicate_relationship_is_rejected() {
        let mut project = project_with_two_activities();
        let existing = Relationship::new(
            RelationshipId::new(1),
            ActivityId::new(1),
            ActivityId::new(2),
            LinkType::Fs,
            0,
        );
        project.relationships.push(existing);

        let change = ProposedChange::CreateRelationship {
            predecessor: ActivityId::new(1),
            successor: ActivityId::new(2),
            rationale: "Synthetic.".to_owned(),
        };
        let proposal = proposal_with(vec![change]);
        let expected = ProposalError::DuplicateRelationship {
            predecessor: ActivityId::new(1),
            successor: ActivityId::new(2),
        };
        assert!(proposal.validate(&project).contains(&expected));
    }

    #[test]
    fn blank_activity_name_is_rejected() {
        let project = project_with_two_activities();
        let change = ProposedChange::CreateActivity {
            name: "   ".to_owned(),
            duration_days: 3,
            rationale: "Synthetic.".to_owned(),
        };
        let proposal = proposal_with(vec![change]);
        assert!(proposal.validate(&project).contains(&ProposalError::EmptyName));
    }

    #[test]
    fn zero_duration_is_rejected_because_it_denotes_a_milestone() {
        let project = project_with_two_activities();
        let change = ProposedChange::CreateActivity {
            name: "Handover".to_owned(),
            duration_days: 0,
            rationale: "Synthetic.".to_owned(),
        };
        let proposal = proposal_with(vec![change]);
        assert!(proposal.validate(&project).contains(&ProposalError::ZeroDuration));
    }

    #[test]
    fn a_proposal_with_no_changes_is_rejected() {
        let result = Proposal::new("Nothing to do.", Vec::new());
        assert_eq!(result, Err(ProposalError::NoChanges));
    }

    #[test]
    fn mock_provider_returns_its_canned_response() {
        let provider = MockProvider::new("canned completion");
        assert_eq!(provider.name(), "mock");
        let completion = provider.complete("any prompt");
        assert_eq!(completion, Ok("canned completion".to_owned()));
    }

    #[test]
    fn noop_provider_fails_closed() {
        let provider = NoopProvider;
        assert_eq!(provider.name(), "disabled");
        let completion = provider.complete("any prompt");
        assert_eq!(completion, Err(AssistantError::NotConfigured));
    }
}
