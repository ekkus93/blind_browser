//! Workflow shapes that carry a remote-planner request through preparation,
//! optional consent challenge/response, and authorization. See [`super::policy`]
//! for how a [`RemotePlannerDataAuthorization`] is decided and [`super::draft`]
//! for how a [`RemotePlannerRequestDraft`] is built.

use crate::app_core::planner_redaction::RemotePlannerInput;
use crate::commands::{
    RemotePlannerConsentChallenge, RemotePlannerConsentResponseOutcome,
    RemotePlannerDisclosureClass, RemotePlannerDisclosureCounts, StartListeningInput,
    TranscribeCommandInput,
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

/// Crate-internal proof that one remote TTS dispatch has already passed the
/// shared remote-data policy. The field is private so ordinary callers cannot
/// manufacture authorization and bypass consent.
#[derive(Debug)]
pub(crate) struct RemoteNarrationAuthorization {
    _private: (),
}

impl RemoteNarrationAuthorization {
    pub(super) fn new() -> Self {
        Self { _private: () }
    }

    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) enum NarrationPreparation {
    Authorized(RemoteNarrationAuthorization),
    ConsentRequired {
        challenge: Box<RemotePlannerConsentChallenge>,
    },
}

pub(crate) struct PendingNarrationConsent {
    pub(crate) challenge: RemotePlannerConsentChallenge,
    pub(super) page_origin: String,
    pub(super) endpoint_scope: String,
    pub(super) profile_name: String,
    pub(super) model_label: String,
    pub(super) runtime_state_token: String,
    pub(super) resume: NarrationResumeContext,
}

pub(crate) enum NarrationConsentResolution {
    Terminal(RemotePlannerConsentResponseOutcome),
    Authorized {
        resume: NarrationResumeContext,
        authorization: RemoteNarrationAuthorization,
    },
}

// --- Microphone audio (remote ASR) disclosure ---

/// Exact operation that should continue after microphone consent. Top-level
/// transcription can resume directly without re-running policy. Planner-tool
/// execution remains explicit because the planner executor cannot currently
/// release/reacquire AppCore mid-plan (BB_CODE_REVIEW3 P1.2).
#[derive(Debug, Clone)]
pub(crate) enum MicrophoneResumeContext {
    Transcribe {
        input: TranscribeCommandInput,
        execute_after: bool,
    },
    StartListening {
        input: StartListeningInput,
    },
    PlannerTool {
        input: TranscribeCommandInput,
    },
}

impl MicrophoneResumeContext {
    pub(crate) fn request_id(&self) -> &str {
        match self {
            Self::Transcribe { input, .. } | Self::PlannerTool { input } => &input.request_id,
            Self::StartListening { input } => &input.request_id,
        }
    }

    pub(crate) fn transcribe_input(&self) -> Option<&TranscribeCommandInput> {
        match self {
            Self::Transcribe { input, .. } | Self::PlannerTool { input } => Some(input),
            Self::StartListening { .. } => None,
        }
    }
}

pub(crate) struct MicrophoneRequestDraft {
    pub(crate) endpoint_scope: ProviderEndpointScope,
    pub(crate) profile_name: String,
    pub(crate) model_label: String,
    pub(crate) page_origin: String,
    pub(crate) request_binding_digest: String,
    pub(crate) runtime_state_token: String,
    pub(crate) effective_duration_ms: u64,
    pub(crate) resume: MicrophoneResumeContext,
}

/// Crate-internal proof that one remote ASR disclosure has already passed the
/// shared remote-data policy. It is intentionally not Clone/Copy/Serialize.
#[derive(Debug)]
pub(crate) struct RemoteMicrophoneAuthorization {
    _private: (),
}

impl RemoteMicrophoneAuthorization {
    pub(super) fn new() -> Self {
        Self { _private: () }
    }

    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) enum MicrophonePreparation {
    Local,
    Authorized(RemoteMicrophoneAuthorization),
    ConsentRequired {
        challenge: Box<RemotePlannerConsentChallenge>,
    },
}

pub(crate) struct PendingMicrophoneConsent {
    pub(crate) challenge: RemotePlannerConsentChallenge,
    pub(super) page_origin: String,
    pub(super) endpoint_scope: String,
    pub(super) profile_name: String,
    pub(super) model_label: String,
    pub(super) runtime_state_token: String,
    pub(super) resume: MicrophoneResumeContext,
}

pub(crate) enum MicrophoneConsentResolution {
    Terminal(RemotePlannerConsentResponseOutcome),
    Authorized {
        resume: MicrophoneResumeContext,
        authorization: RemoteMicrophoneAuthorization,
    },
}
