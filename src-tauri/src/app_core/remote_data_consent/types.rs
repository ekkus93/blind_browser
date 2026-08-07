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
