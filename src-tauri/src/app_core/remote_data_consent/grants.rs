//! Ephemeral (non-persisted) remote-planner consent grants: session-scoped
//! and one-shot "allow" decisions that live only in [`super::super::AppCore`]
//! runtime state, bounded and pruned so they can't grow unbounded across a
//! long-running session.

use std::sync::atomic::{AtomicU8, Ordering};

use crate::config::REMOTE_DATA_POLICY_VERSION;

use super::super::AppCore;

pub(super) const SESSION_GRANT_TTL_MS: u64 = 8 * 60 * 60 * 1_000;
const MAX_EPHEMERAL_GRANTS: usize = 64;

pub(crate) enum EphemeralConsentKind {
    Once {
        challenge_digest: String,
        remaining_uses: AtomicU8,
    },
    Session,
}

pub(crate) struct RemotePlannerEphemeralGrant {
    pub(super) page_origin: String,
    endpoint_scope: String,
    policy_version: u32,
    expires_at_ms: u64,
    kind: EphemeralConsentKind,
}

impl RemotePlannerEphemeralGrant {
    pub(super) fn session(
        page_origin: String,
        endpoint_scope: String,
        policy_version: u32,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            page_origin,
            endpoint_scope,
            policy_version,
            expires_at_ms,
            kind: EphemeralConsentKind::Session,
        }
    }

    pub(super) fn once(
        page_origin: String,
        endpoint_scope: String,
        policy_version: u32,
        challenge_digest: String,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            page_origin,
            endpoint_scope,
            policy_version,
            expires_at_ms,
            kind: EphemeralConsentKind::Once {
                challenge_digest,
                remaining_uses: AtomicU8::new(1),
            },
        }
    }

    pub(super) fn is_matching_session(
        &self,
        page_origin: &str,
        endpoint_scope: &str,
        now_ms: u64,
    ) -> bool {
        self.expires_at_ms > now_ms
            && self.page_origin == page_origin
            && self.endpoint_scope == endpoint_scope
            && self.policy_version == REMOTE_DATA_POLICY_VERSION
            && matches!(&self.kind, EphemeralConsentKind::Session)
    }

    pub(super) fn consume_matching_once(
        &self,
        page_origin: &str,
        endpoint_scope: &str,
        challenge_digest: &str,
        now_ms: u64,
    ) -> bool {
        if self.expires_at_ms <= now_ms
            || self.page_origin != page_origin
            || self.endpoint_scope != endpoint_scope
            || self.policy_version != REMOTE_DATA_POLICY_VERSION
        {
            return false;
        }
        let EphemeralConsentKind::Once {
            challenge_digest: expected_digest,
            remaining_uses,
        } = &self.kind
        else {
            return false;
        };
        if expected_digest != challenge_digest {
            return false;
        }
        remaining_uses
            .compare_exchange(1, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

/// Which disclosure kind an ephemeral grant or pending consent belongs to.
/// Selects which of [`AppCore`]'s three independent grant lists (and, in
/// [`super::origin_rules`], which of its three independent config sections)
/// a call operates on -- kept as an explicit, caller-supplied selector
/// (mirroring [`super::types::RemotePlannerDataAuthorization`]'s existing
/// style) rather than three sets of near-identical methods, so the
/// underlying grant-management logic below stays single-sourced.
// Remote ASR (microphone audio) doesn't have a disclosure kind here yet --
// see the module-level doc comment on `super` for why -- so this only
// selects between the planner and narration grant stores for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteDataDisclosureKind {
    PlannerPayload,
    NarrationText,
}

impl AppCore {
    fn ephemeral_grants_mut(
        &mut self,
        kind: RemoteDataDisclosureKind,
    ) -> &mut Vec<RemotePlannerEphemeralGrant> {
        match kind {
            RemoteDataDisclosureKind::PlannerPayload => &mut self.remote_planner_ephemeral_grants,
            RemoteDataDisclosureKind::NarrationText => &mut self.remote_narration_ephemeral_grants,
        }
    }

    pub(super) fn ephemeral_grants(
        &self,
        kind: RemoteDataDisclosureKind,
    ) -> &[RemotePlannerEphemeralGrant] {
        match kind {
            RemoteDataDisclosureKind::PlannerPayload => &self.remote_planner_ephemeral_grants,
            RemoteDataDisclosureKind::NarrationText => &self.remote_narration_ephemeral_grants,
        }
    }

    pub(super) fn prune_remote_planner_grants(
        &mut self,
        kind: RemoteDataDisclosureKind,
        now_ms: u64,
    ) {
        self.ephemeral_grants_mut(kind)
            .retain(|grant| grant.expires_at_ms > now_ms);
    }

    pub(super) fn install_session_grant(
        &mut self,
        kind: RemoteDataDisclosureKind,
        page_origin: String,
        endpoint_scope: String,
        expires_at_ms: u64,
    ) {
        let grants = self.ephemeral_grants_mut(kind);
        grants.retain(|grant| {
            !(grant.page_origin == page_origin
                && grant.endpoint_scope == endpoint_scope
                && matches!(&grant.kind, EphemeralConsentKind::Session))
        });
        grants.push(RemotePlannerEphemeralGrant::session(
            page_origin,
            endpoint_scope,
            REMOTE_DATA_POLICY_VERSION,
            expires_at_ms,
        ));
        self.bound_remote_planner_grants(kind);
    }

    pub(super) fn install_once_grant(
        &mut self,
        kind: RemoteDataDisclosureKind,
        page_origin: String,
        endpoint_scope: String,
        challenge_digest: String,
        expires_at_ms: u64,
    ) {
        self.ephemeral_grants_mut(kind)
            .push(RemotePlannerEphemeralGrant::once(
                page_origin,
                endpoint_scope,
                REMOTE_DATA_POLICY_VERSION,
                challenge_digest,
                expires_at_ms,
            ));
        self.bound_remote_planner_grants(kind);
    }

    pub(super) fn consume_once_grant(
        &self,
        kind: RemoteDataDisclosureKind,
        page_origin: &str,
        endpoint_scope: &str,
        challenge_digest: &str,
        now_ms: u64,
    ) -> bool {
        self.ephemeral_grants(kind).iter().any(|grant| {
            grant.consume_matching_once(page_origin, endpoint_scope, challenge_digest, now_ms)
        })
    }

    fn bound_remote_planner_grants(&mut self, kind: RemoteDataDisclosureKind) {
        let grants = self.ephemeral_grants_mut(kind);
        if grants.len() > MAX_EPHEMERAL_GRANTS {
            let remove = grants.len() - MAX_EPHEMERAL_GRANTS;
            grants.drain(0..remove);
        }
    }
}
