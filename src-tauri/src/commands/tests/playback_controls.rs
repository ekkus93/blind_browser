use super::*;

#[test]
fn set_playback_volume_clamps_requested_value_and_updates_readback() {
    let mut executor = MockExecutor::default();

    let result = executor.execute_set_playback_volume(SetPlaybackVolumeInput {
        request_id: String::from("req-volume-clamp"),
        timeout_ms: Some(1_000),
        volume: -0.25,
    });

    assert!(result.ok);
    assert_eq!(
        result.observations,
        vec![
            String::from("Updated the playback volume setting."),
            String::from("New narration requests will use the updated playback volume."),
            String::from("Requested playback volume was clamped to the supported range."),
        ]
    );
    assert_eq!(
        result.data.expect("volume tool should return data"),
        SetPlaybackVolumeData {
            playback_volume: 0.0,
            muted: true,
            changed: true,
        }
    );
    assert_eq!(executor.last_volume, Some(-0.25));

    let state = executor.execute_get_agent_state(GetAgentStateInput {
        request_id: String::from("req-agent-state"),
        timeout_ms: Some(1_000),
        include_last_transcript: false,
    });
    assert!(state.ok);
    let state_data = state.data.expect("agent state should return data");
    assert_eq!(state_data.audio.playback_volume, 0.0);
    assert!(state_data.audio.muted);
}

#[test]
fn set_playback_speed_clamps_requested_value_and_updates_readback() {
    let mut executor = MockExecutor::default();

    let result = executor.execute_set_playback_speed(SetPlaybackSpeedInput {
        request_id: String::from("req-speed-clamp"),
        timeout_ms: Some(1_000),
        speed: 9.0,
    });

    assert!(result.ok);
    assert_eq!(
        result.observations,
        vec![
            String::from("Updated the playback speed setting."),
            String::from("New narration requests will use the updated native TTS speed."),
            String::from("Requested playback speed was clamped to the supported range."),
        ]
    );
    assert_eq!(
        result.data.expect("speed tool should return data"),
        SetPlaybackSpeedData {
            playback_speed: crate::config::MAX_PLAYBACK_SPEED,
            changed: true,
        }
    );
    assert_eq!(executor.last_speed, Some(9.0));

    let status = executor.execute_get_runtime_status(GetRuntimeStatusInput {
        request_id: String::from("req-runtime-status"),
        timeout_ms: Some(1_000),
        include_provider_modes: false,
    });
    assert!(status.ok);
    let status_data = status.data.expect("runtime status should return data");
    assert_eq!(
        status_data.audio.playback_speed,
        crate::config::MAX_PLAYBACK_SPEED
    );
}
