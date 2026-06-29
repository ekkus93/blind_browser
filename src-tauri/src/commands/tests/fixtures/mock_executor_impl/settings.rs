use super::*;

pub(super) fn execute_set_tts_voice(
    ex: &mut MockExecutor,
    input: SetTtsVoiceInput,
) -> ToolResult<SetTtsVoiceData> {
    ex.last_voice = Some(input.voice.to_string());
    ToolResult::success(
        ToolName::SetTtsVoice,
        input.request_id,
        SetTtsVoiceData {
            voice: input.voice.to_string(),
            changed: true,
        },
        vec![String::from("voice updated")],
    )
}

pub(super) fn execute_set_playback_volume(
    ex: &mut MockExecutor,
    input: SetPlaybackVolumeInput,
) -> ToolResult<SetPlaybackVolumeData> {
    let clamped_volume = input.volume.clamp(
        crate::config::MIN_PLAYBACK_VOLUME,
        crate::config::MAX_PLAYBACK_VOLUME,
    );
    let changed = (ex.audio.playback_volume - clamped_volume).abs() > f32::EPSILON;
    ex.last_volume = Some(input.volume);
    ex.audio.playback_volume = clamped_volume;
    ex.audio.muted = clamped_volume == 0.0;
    let mut observations = vec![
        String::from("Updated the playback volume setting."),
        String::from("New narration requests will use the updated playback volume."),
    ];
    if (input.volume - clamped_volume).abs() > f32::EPSILON {
        observations.push(String::from(
            "Requested playback volume was clamped to the supported range.",
        ));
    }
    ToolResult::success(
        ToolName::SetPlaybackVolume,
        input.request_id,
        SetPlaybackVolumeData {
            playback_volume: ex.audio.playback_volume,
            muted: ex.audio.muted,
            changed,
        },
        observations,
    )
}

pub(super) fn execute_set_playback_speed(
    ex: &mut MockExecutor,
    input: SetPlaybackSpeedInput,
) -> ToolResult<SetPlaybackSpeedData> {
    let clamped_speed = input.speed.clamp(
        crate::config::MIN_PLAYBACK_SPEED,
        crate::config::MAX_PLAYBACK_SPEED,
    );
    let changed = (ex.audio.playback_speed - clamped_speed).abs() > f32::EPSILON;
    ex.last_speed = Some(input.speed);
    ex.audio.playback_speed = clamped_speed;
    let mut observations = vec![
        String::from("Updated the playback speed setting."),
        String::from("New narration requests will use the updated native TTS speed."),
    ];
    if (input.speed - clamped_speed).abs() > f32::EPSILON {
        observations.push(String::from(
            "Requested playback speed was clamped to the supported range.",
        ));
    }
    ToolResult::success(
        ToolName::SetPlaybackSpeed,
        input.request_id,
        SetPlaybackSpeedData {
            playback_speed: ex.audio.playback_speed,
            changed,
        },
        observations,
    )
}

pub(super) fn execute_set_browser_visibility(
    ex: &mut MockExecutor,
    input: SetBrowserVisibilityInput,
) -> ToolResult<SetBrowserVisibilityData> {
    ex.last_visibility = Some(input.mode);
    if ex.browser_visibility == input.mode {
        return ToolResult::success(
            ToolName::SetBrowserVisibility,
            input.request_id,
            SetBrowserVisibilityData {
                mode: ex.browser_visibility,
                changed: false,
                supported: true,
            },
            vec![String::from(
                "Browser visibility mode is already set to the requested value.",
            )],
        );
    }
    if !ex.browser_visibility_switch_supported {
        return ToolResult::success(
            ToolName::SetBrowserVisibility,
            input.request_id,
            SetBrowserVisibilityData {
                mode: ex.browser_visibility,
                changed: false,
                supported: false,
            },
            vec![String::from(
                "Browser visibility switching is not supported in this build.",
            )],
        );
    }
    ex.browser_visibility = input.mode;
    ToolResult::success(
        ToolName::SetBrowserVisibility,
        input.request_id,
        SetBrowserVisibilityData {
            mode: ex.browser_visibility,
            changed: true,
            supported: true,
        },
        vec![String::from("Browser visibility mode was updated.")],
    )
}
