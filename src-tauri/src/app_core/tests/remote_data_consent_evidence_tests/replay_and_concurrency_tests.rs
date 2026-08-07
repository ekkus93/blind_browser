//! Evidence that a consumed or denied consent response can never be replayed,
//! and that concurrent resolution attempts against the same challenge yield
//! exactly one authorization.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

use super::*;

use crate::app_core::remote_data_consent::PendingConsentResolution;
use crate::commands::{RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome};
use crate::provider_endpoint::ProviderEndpointScope;

#[test]
#[cfg_attr(
    any(windows, target_os = "linux"),
    ignore = "real Wry AppCore fixture must run in a process-isolated test invocation"
)]
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    ignore = "real Wry AppCore fixture requires Tauri's any-thread desktop builder"
)]
fn remote_data_consent_request_counts_replay_and_concurrency_are_enforced() {
    let (app, _config_root) = test_app();
    let (mut core, _secret) = test_core(&app);

    let (denied, draft) = requirement(&mut core, "deny", "analyze this article");
    store(&mut core, denied.clone(), draft);
    let result = core
        .resolve_pending_remote_planner_consent(
            &denied.challenge_id,
            &denied.challenge_digest,
            RemotePlannerConsentDecision::Deny,
        )
        .expect("deny should resolve");
    assert!(matches!(
        result,
        PendingConsentResolution::Terminal(RemotePlannerConsentResponseOutcome::Denied)
    ));
    assert_replay_missing(&mut core, &denied);

    let (challenge, draft) = requirement(&mut core, "allow", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    let core = Arc::new(Mutex::new(core));
    let barrier = Arc::new(Barrier::new(3));
    let workers = (0..2)
        .map(|_| {
            let core = Arc::clone(&core);
            let barrier = Arc::clone(&barrier);
            let challenge = challenge.clone();
            thread::spawn(move || {
                barrier.wait();
                core.lock()
                    .expect("core lock should not be poisoned")
                    .resolve_pending_remote_planner_consent(
                        &challenge.challenge_id,
                        &challenge.challenge_digest,
                        RemotePlannerConsentDecision::AllowOnce,
                    )
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();

    let mut authorized = None;
    let mut missing = 0;
    for worker in workers {
        match worker.join().expect("consent worker should join") {
            Ok(PendingConsentResolution::Authorized(ready)) => authorized = Some(*ready),
            Err(error) if error.code == "remote_data_consent_missing" => missing += 1,
            Ok(PendingConsentResolution::Terminal(outcome)) => {
                panic!("allow-once returned terminal outcome: {outcome:?}")
            }
            Err(error) => panic!("unexpected consent error: {error:?}"),
        }
    }
    assert_eq!(missing, 1);
    let mut authorized = authorized.expect("exactly one response should authorize");
    assert_eq!(
        authorized.prepared.endpoint_scope.normalized_base_url(),
        challenge.endpoint_scope
    );

    let (base_url, request_count, server) = counting_server();
    assert_eq!(request_count.load(Ordering::Acquire), 0);
    authorized.prepared.endpoint_scope =
        ProviderEndpointScope::parse(&base_url).expect("loopback test endpoint should parse");
    let send_error = tauri::async_runtime::block_on(async {
        crate::app_core::remote_planner::resolve_remote_planner(&authorized.prepared)
    })
    .expect_err("test server intentionally rejects the request");
    assert_eq!(send_error.code, "planner_request_failed");
    server.join().expect("test server should join");
    assert_eq!(request_count.load(Ordering::Acquire), 1);

    let mut core = core.lock().expect("core lock should not be poisoned");
    assert_replay_missing(&mut core, &challenge);
    let _ = requirement(&mut core, "next", "analyze this article");
    assert_eq!(request_count.load(Ordering::Acquire), 1);
}
