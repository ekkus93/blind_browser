//! Workflow shapes that carry a remote-planner request through preparation,
//! optional consent challenge/response, and authorization. See [`super::policy`]
//! for how a [`RemotePlannerDataAuthorization`] is decided and [`super::draft`]
//! for how a [`RemotePlannerRequestDraft`] is built.

use crate::app_core::planner_redaction::RemotePlannerInput;
use crate::commands::{
    RemotePlannerConsentChallenge, RemotePlannerConsentResponseOutcome,
    RemotePlannerDisclosureClass, RemotePlannerDisclosureCounts,
};
use crate::config::RemotePlannerProfile;
use crate::provider_endpoint::ProviderEndpointScope;
use crate::state::PlanningStateSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemotePlannerDataAuthorization {
    Loopback,
    GlobalAllow,
    PersistentAllow,
    SessionAllow,
    AllowOnce,
}

pub(crate) struct RemotePlannerRequestDraft {
    pub(crate) profile_name: String,
    pub(crate) profile: RemotePlannerProfile,
    pub(crate) endpoint_scope: ProviderEndpointScope,
    pub(crate) sanitized_input: Box<RemotePlannerInput>,
    pub(crate) payload_digest: String,
    pub(crate) page_origin: String,
    pub(crate) runtime_state_token: String,
    pub(crate) disclosure_classes: Vec<RemotePlannerDisclosureClass>,
    pub(crate) disclosure_counts: RemotePlannerDisclosureCounts,
}

pub(crate) struct PreparedRemotePlannerRequest {
    pub(crate) profile_name: String,
    pub(crate) profile: RemotePlannerProfile,
    pub(crate) endpoint_scope: ProviderEndpointScope,
    pub(crate) sanitized_input: RemotePlannerInput,
    pub(crate) authorization: RemotePlannerDataAuthorization,
    pub(crate) page_origin: String,
    pub(crate) runtime_state_token: String,
}

impl RemotePlannerRequestDraft {
    pub(super) fn authorize(
        self,
        authorization: RemotePlannerDataAuthorization,
    ) -> PreparedRemotePlannerRequest {
        PreparedRemotePlannerRequest {
            profile_name: self.profile_name,
            profile: self.profile,
            endpoint_scope: self.endpoint_scope,
            sanitized_input: *self.sanitized_input,
            authorization,
            page_origin: self.page_origin,
            runtime_state_token: self.runtime_state_token,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingRemotePlannerContinuation {
    ResolveOnly,
    Execute,
}

pub(crate) struct PendingRemotePlannerConsent {
    pub(crate) challenge: RemotePlannerConsentChallenge,
    pub(super) draft: RemotePlannerRequestDraft,
    pub(super) planning_snapshot: PlanningStateSnapshot,
    pub(super) continuation: PendingRemotePlannerContinuation,
}

pub(crate) enum RemotePlannerPreparation {
    Authorized(Box<PreparedRemotePlannerRequest>),
    ConsentRequired {
        challenge: Box<RemotePlannerConsentChallenge>,
        draft: Box<RemotePlannerRequestDraft>,
    },
}

pub(crate) struct AuthorizedPendingRemotePlannerRequest {
    pub(crate) prepared: PreparedRemotePlannerRequest,
    pub(crate) planning_snapshot: PlanningStateSnapshot,
    pub(crate) continuation: PendingRemotePlannerContinuation,
    pub(crate) request_id: String,
}

pub(crate) enum PendingConsentResolution {
    Terminal(RemotePlannerConsentResponseOutcome),
    Authorized(Box<AuthorizedPendingRemotePlannerRequest>),
}

// --- Narration (remote TTS) disclosure ---
//
// Narration has no planner-style sanitization step (the whole point is to
// speak the text; there is nothing to redact without breaking that), so
// unlike the planner there is no separate draft/prepared-request split --
// preparing a narration request either succeeds immediately (Authorized) or
// yields a challenge to answer (ConsentRequired), never an intermediate
// sanitized-but-not-yet-authorized value.

/// What to redo once a pending narration consent is authorized. Covers every
/// call site that reaches `AppCore::begin_region_narration`/
/// `begin_feedback_narration` (execute_read_region, execute_read_next_region,
/// execute_read_previous_region all resolve to a concrete region id before
/// calling in; execute_report_result supplies feedback text directly) --
/// storing the *resolved* target rather than "next"/"previous" semantics, so
/// resuming re-reads exactly the region the user was told about in the
/// consent dialog, not whatever "next" has drifted to become.
#[derive(Debug, Clone)]
pub(crate) enum NarrationResumeContext {
    Region {
        region_id: String,
        interrupt_current: bool,
    },
    Feedback {
        spoken_text: String,
    },
}

pub(crate) struct NarrationRequestDraft {
    pub(crate) text: String,
    pub(crate) endpoint_scope: ProviderEndpointScope,
    pub(crate) profile_name: String,
    pub(crate) model_label: String,
    pub(crate) page_origin: String,
    pub(crate) payload_digest: String,
    pub(crate) runtime_state_token: String,
    pub(crate) resume: NarrationResumeContext,
}

#[derive(Debug)]
pub(crate) enum NarrationPreparation {
    Authorized,
    ConsentRequired {
        challenge: Box<RemotePlannerConsentChallenge>,
    },
}

pub(crate) struct PendingNarrationConsent {
    pub(crate) challenge: RemotePlannerConsentChallenge,
    pub(super) page_origin: String,
    pub(super) endpoint_scope: String,
    pub(super) runtime_state_token: String,
    pub(super) resume: NarrationResumeContext,
}

pub(crate) enum NarrationConsentResolution {
    Terminal(RemotePlannerConsentResponseOutcome),
    Authorized { resume: NarrationResumeContext },
}

// Remote ASR (microphone audio) is not yet gated through this module -- see
// the module-level doc comment in mod.rs for why.
