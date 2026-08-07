//! Evidence tests for the remote-planner consent workflow
//! (`super::super::remote_data_consent`), split by scenario:
//! - [`replay_and_concurrency_tests`] — replay/denial rejection, concurrent resolution
//! - [`expiry_and_hostile_state_tests`] — expiry, invalidation, persistence, leak-freedom
//! - [`identity_scope_and_restart_tests`] — challenge binding, unrelated-change tolerance, restart
//! - [`policy_and_disclosure_matrix_tests`] — the full allow/block/consent-required matrix
//! - [`narration_consent_tests`] — the same policy matrix for the narration
//!   (remote TTS) disclosure kind, proving it shares the engine rather than
//!   reimplementing it, and that its grant/origin-rule store stays independent
//!
//! [`helpers`] holds the shared `AppCore` fixture and request/store/resolve helpers
//! every scenario builds on.

use super::*;

mod helpers;
use helpers::*;

mod expiry_and_hostile_state_tests;
mod identity_scope_and_restart_tests;
mod narration_consent_tests;
mod policy_and_disclosure_matrix_tests;
mod replay_and_concurrency_tests;
