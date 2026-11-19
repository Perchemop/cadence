//! Cadence's optional AI assistant: the provider seam, its HTTP backends, and
//! the typed proposal a model response becomes.
//!
//! The assistant is **off by default**. No network call ever happens on its
//! own: the HTTP backends run only through an explicitly configured
//! [`AssistantConfig`] (provider, model and key, where one is needed) and only
//! when the application calls [`HttpProvider::complete`] in response to an
//! explicit user action. With nothing configured the feature is hidden and
//! fails closed.
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
//!   cannot carry it; HTTP errors never echo request material either.

mod command_map;
mod config;
mod http;
mod proposal;
mod prompt;
mod provider;

pub use command_map::map_proposal_to_commands;
pub use config::{AssistantConfig, SecretKey};
pub use http::HttpProvider;
pub use proposal::{Proposal, ProposalError, ProposedChange};
pub use prompt::{ACTIVITY_SNAPSHOT_CAP, build_proposal_prompt, parse_proposal_response};
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
            model: Some("test-model".to_owned()),
            base_url: None,
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
    fn a_local_ollama_server_needs_no_api_key() {
        let ollama = AssistantConfig {
            enabled: true,
            provider_name: Some("ollama".to_owned()),
            api_key: None,
            model: Some("test-model".to_owned()),
            base_url: None,
        };
        assert!(ollama.is_configured());

        // Any hosted provider still requires a key.
        let hosted = AssistantConfig {
            provider_name: Some("openai".to_owned()),
            ..ollama
        };
        assert!(!hosted.is_configured());
    }

    #[test]
    fn a_keyless_openai_compatible_server_needs_only_its_address() {
        let config = AssistantConfig {
            enabled: true,
            provider_name: Some("lm-studio".to_owned()),
            api_key: None,
            model: Some("test-model".to_owned()),
            base_url: Some("http://lm-studio.local:1234/v1".to_owned()),
        };
        assert!(config.is_configured());
    }

    #[test]
    fn provider_names_are_case_insensitive() {
        // The wire mapping lowercases names, so the config gate must too —
        // a capitalised "Ollama" is still the keyless local server.
        let ollama = AssistantConfig {
            enabled: true,
            provider_name: Some("Ollama".to_owned()),
            api_key: None,
            model: Some("test-model".to_owned()),
            base_url: None,
        };
        assert!(ollama.is_configured());
    }

    #[test]
    fn a_model_is_required_before_the_assistant_is_configured() {
        let no_model = AssistantConfig {
            enabled: true,
            provider_name: Some("openai".to_owned()),
            api_key: Some(SecretKey::new("not-a-real-credential")),
            model: None,
            base_url: None,
        };
        assert!(!no_model.is_configured());

        let blank_model = AssistantConfig {
            model: Some("   ".to_owned()),
            ..no_model
        };
        assert!(!blank_model.is_configured());
    }

    #[test]
    fn a_config_saved_before_the_http_slice_still_parses() {
        // Settings written by the foundation slice carry no model or base_url;
        // they must still load (and simply not yet be usable until a model is
        // chosen).
        let legacy = r#"{"enabled":true,"provider_name":"openai","api_key":"<redacted>"}"#;
        let config: AssistantConfig = serde_json::from_str(legacy).expect("legacy settings parse");
        assert!(config.enabled);
        assert_eq!(config.provider_name.as_deref(), Some("openai"));
        assert_eq!(config.model, None);
        assert_eq!(config.base_url, None);
        assert!(!config.is_configured());
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
            model: Some("test-model".to_owned()),
            base_url: None,
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
        assert!(proposal
            .validate(&project)
            .contains(&ProposalError::EmptyName));
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
        assert!(proposal
            .validate(&project)
            .contains(&ProposalError::ZeroDuration));
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
