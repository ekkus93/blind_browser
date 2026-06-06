use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct NormalizedAudioSetting {
    pub(super) value: f32,
    pub(super) goal: String,
    pub(super) skill_name: &'static str,
}

pub(super) fn parse_volume_command(normalized: &str, current_volume: f32) -> Option<NormalizedAudioSetting> {
    if normalized == "mute" || normalized.contains("mute volume") {
        return Some(NormalizedAudioSetting {
            value: 0.0,
            goal: String::from("Set playback volume to muted."),
            skill_name: "mute_volume",
        });
    }

    if let Some(step) = volume_relative_step(normalized) {
        let target = (current_volume + step).clamp(0.0, MAX_PLAYBACK_VOLUME);
        let goal = if step.is_sign_positive() {
            String::from("Increase playback volume by the requested normalized step.")
        } else {
            String::from("Decrease playback volume by the requested normalized step.")
        };
        let skill_name = if step.is_sign_positive() {
            "increase_volume"
        } else {
            "decrease_volume"
        };
        return Some(NormalizedAudioSetting {
            value: round_audio_setting_value(target),
            goal,
            skill_name,
        });
    }

    if !normalized.contains("volume") {
        return None;
    }

    parse_absolute_volume_value(normalized).map(|value| NormalizedAudioSetting {
        value: round_audio_setting_value(value.clamp(0.0, MAX_PLAYBACK_VOLUME)),
        goal: String::from("Set playback volume to the requested normalized value."),
        skill_name: "set_volume",
    })
}

pub(super) fn parse_speed_command(normalized: &str, current_speed: f32) -> Option<NormalizedAudioSetting> {
    if let Some(step) = speed_relative_step(normalized) {
        let target = (current_speed + step).clamp(MIN_PLAYBACK_SPEED, MAX_PLAYBACK_SPEED);
        let goal = if step.is_sign_positive() {
            String::from("Increase playback speed by the requested normalized step.")
        } else {
            String::from("Decrease playback speed by the requested normalized step.")
        };
        let skill_name = if step.is_sign_positive() {
            "increase_playback_speed"
        } else {
            "decrease_playback_speed"
        };
        return Some(NormalizedAudioSetting {
            value: round_audio_setting_value(target),
            goal,
            skill_name,
        });
    }

    if !normalized.contains("speed") {
        return None;
    }

    parse_absolute_speed_value(normalized).map(|value| NormalizedAudioSetting {
        value: round_audio_setting_value(value.clamp(MIN_PLAYBACK_SPEED, MAX_PLAYBACK_SPEED)),
        goal: String::from("Set playback speed to the requested normalized value."),
        skill_name: "set_playback_speed",
    })
}

pub(super) fn parse_absolute_volume_value(normalized: &str) -> Option<f32> {
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    for (index, token) in tokens.iter().enumerate() {
        let Ok(value) = token.parse::<f32>() else {
            continue;
        };

        if tokens.get(index + 1).copied() == Some("percent") {
            return Some(value / 100.0);
        }
        if value.fract() == 0.0 && (0.0..=100.0).contains(&value) {
            return Some(value / 100.0);
        }
        return Some(value);
    }

    None
}

pub(super) fn parse_absolute_speed_value(normalized: &str) -> Option<f32> {
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    for (index, token) in tokens.iter().enumerate() {
        if let Some(multiplier) = parse_multiplier_token(token) {
            return Some(multiplier);
        }

        let Ok(value) = token.parse::<f32>() else {
            continue;
        };

        if matches!(tokens.get(index + 1).copied(), Some("times") | Some("time")) {
            return Some(value);
        }
        if tokens.get(index + 1).copied() == Some("percent") {
            return Some(value / 100.0);
        }
        return Some(value);
    }

    None
}

pub(super) fn parse_multiplier_token(token: &str) -> Option<f32> {
    token
        .strip_suffix('x')
        .and_then(|value| (!value.is_empty()).then_some(value))
        .and_then(|value| value.parse::<f32>().ok())
}

pub(super) fn volume_relative_step(normalized: &str) -> Option<f32> {
    if normalized.contains("increase volume")
        || normalized.contains("turn it up")
        || normalized.contains("volume up")
        || normalized.contains("louder")
    {
        return Some(volume_step_size(normalized));
    }

    if normalized.contains("decrease volume")
        || normalized.contains("turn it down")
        || normalized.contains("volume down")
        || normalized.contains("quieter")
    {
        return Some(-volume_step_size(normalized));
    }

    None
}

pub(super) fn speed_relative_step(normalized: &str) -> Option<f32> {
    if normalized.contains("increase playback speed")
        || normalized.contains("speed up")
        || normalized.contains("go faster")
        || normalized == "faster"
    {
        return Some(speed_step_size(normalized));
    }

    if normalized.contains("decrease playback speed")
        || normalized.contains("slow down")
        || normalized.contains("go slower")
        || normalized == "slower"
    {
        return Some(-speed_step_size(normalized));
    }

    None
}

pub(super) fn volume_step_size(normalized: &str) -> f32 {
    if normalized.contains("a little") || normalized.contains("slightly") {
        SMALL_VOLUME_STEP
    } else if normalized.contains("a lot") || normalized.contains("much") {
        LARGE_VOLUME_STEP
    } else {
        DEFAULT_VOLUME_STEP
    }
}

pub(super) fn speed_step_size(normalized: &str) -> f32 {
    if normalized.contains("a little") || normalized.contains("slightly") {
        SMALL_SPEED_STEP
    } else if normalized.contains("a lot") || normalized.contains("much") {
        LARGE_SPEED_STEP
    } else {
        DEFAULT_SPEED_STEP
    }
}
pub(super) fn format_playback_volume(volume: f32) -> String {
    format!("{}%", (volume * 100.0).round() as i32)
}

pub(super) fn format_playback_speed(speed: f32) -> String {
    let formatted = format!("{speed:.2}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed}x")
}
