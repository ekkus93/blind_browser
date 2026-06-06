use super::*;

#[test]
fn resolve_direct_audio_command_normalizes_absolute_volume_percent() {
    let planner_output = resolve_direct_audio_command(
        "set volume to 70 percent",
        "req-volume",
        1.0,
        1.0,
        &[String::from("set_volume")],
    )
    .expect("volume command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::SetPlaybackVolume);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("set_volume")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(
        planner_output.steps[0].tool_name,
        ToolName::SetPlaybackVolume
    );
    let volume = planner_output.steps[0]
        .arguments
        .get("volume")
        .and_then(serde_json::Value::as_f64)
        .expect("volume should be numeric");
    assert!((volume - 0.7).abs() < 0.000_001);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Playback volume set to 70%."))
    );

    let fuzzy_planner_output = resolve_direct_audio_command(
        "set volum to 70 percent",
        "req-volume-fuzzy",
        1.0,
        1.0,
        &[String::from("set_volume")],
    )
    .expect("fuzzy volume command should normalize");

    let fuzzy_volume = fuzzy_planner_output.steps[0]
        .arguments
        .get("volume")
        .and_then(serde_json::Value::as_f64)
        .expect("fuzzy volume should be numeric");
    assert!((fuzzy_volume - 0.7).abs() < 0.000_001);
}

#[test]
fn resolve_direct_audio_command_applies_large_relative_speed_step() {
    let planner_output = resolve_direct_audio_command(
        "go faster a lot",
        "req-speed",
        1.0,
        1.0,
        &[String::from("increase_playback_speed")],
    )
    .expect("speed command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::SetPlaybackSpeed);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("increase_playback_speed")]
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("speed"),
        Some(&serde_json::json!(1.5))
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Playback speed set to 1.5x."))
    );
}

#[test]
fn resolve_direct_audio_command_reports_current_speed_for_queries() {
    let planner_output = resolve_direct_audio_command(
        "tell me the speed",
        "req-speed-query",
        0.8,
        1.25,
        &[String::from("get_playback_speed")],
    )
    .expect("speed query should normalize");

    assert_eq!(planner_output.intent.name, IntentName::GetPlaybackSpeed);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("get_playback_speed")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("summary"),
        Some(&serde_json::json!("Playback speed is 1.25x."))
    );
}

#[test]
fn resolve_direct_browser_visibility_command_normalizes_headless_phrase() {
    let planner_output = resolve_direct_browser_visibility_command(
        "go headless",
        "req-headless",
        BrowserVisibilityMode::Visible,
        &[String::from("toggle_browser_visibility")],
    )
    .expect("visibility command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::SetBrowserVisibility);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("toggle_browser_visibility")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(
        planner_output.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(BrowserVisibilityMode::Headless))
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Browser mode set to headless."))
    );

    let fuzzy_planner_output = resolve_direct_browser_visibility_command(
        "show the browsr",
        "req-visible-fuzzy",
        BrowserVisibilityMode::Headless,
        &[String::from("toggle_browser_visibility")],
    )
    .expect("fuzzy visibility command should normalize");

    assert_eq!(
        fuzzy_planner_output.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(BrowserVisibilityMode::Visible))
    );
}

#[test]
fn resolve_direct_browser_visibility_command_toggles_when_requested() {
    let planner_output = resolve_direct_browser_visibility_command(
        "toggle browser visibility",
        "req-toggle",
        BrowserVisibilityMode::Headless,
        &[String::from("toggle_browser_visibility")],
    )
    .expect("toggle visibility command should normalize");

    assert_eq!(
        planner_output.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(BrowserVisibilityMode::Visible))
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Browser mode set to visible."))
    );
}
