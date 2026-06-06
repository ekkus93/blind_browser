use super::invalid_planner_output;
use crate::commands::{SetPlaybackSpeedInput, SetPlaybackVolumeInput, SetTtsVoiceInput, ToolError};
use crate::config::{MAX_PLAYBACK_SPEED, MAX_PLAYBACK_VOLUME, MIN_PLAYBACK_SPEED};

pub(super) fn validate_set_tts_voice_input(_input: &SetTtsVoiceInput) -> Result<(), ToolError> {
    Ok(())
}

pub(super) fn validate_set_playback_volume_input(
    input: &SetPlaybackVolumeInput,
) -> Result<(), ToolError> {
    if !input.volume.is_finite() {
        return Err(invalid_planner_output(
            "set_playback_volume requires a finite numeric volume value",
            None,
        ));
    }

    if !(0.0..=MAX_PLAYBACK_VOLUME).contains(&input.volume) {
        return Err(invalid_planner_output(
            format!("set_playback_volume volume must be between 0.0 and {MAX_PLAYBACK_VOLUME}"),
            Some(serde_json::json!({ "volume": input.volume })),
        ));
    }

    Ok(())
}

pub(super) fn validate_set_playback_speed_input(
    input: &SetPlaybackSpeedInput,
) -> Result<(), ToolError> {
    if !input.speed.is_finite() {
        return Err(invalid_planner_output(
            "set_playback_speed requires a finite numeric speed value",
            None,
        ));
    }

    if !(MIN_PLAYBACK_SPEED..=MAX_PLAYBACK_SPEED).contains(&input.speed) {
        return Err(invalid_planner_output(
            format!(
                "set_playback_speed speed must be between {MIN_PLAYBACK_SPEED} and {MAX_PLAYBACK_SPEED}"
            ),
            Some(serde_json::json!({ "speed": input.speed })),
        ));
    }

    Ok(())
}
