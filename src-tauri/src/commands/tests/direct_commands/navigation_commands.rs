use super::*;

#[test]
fn resolve_direct_navigation_readback_command_builds_history_and_reload_plans() {
    let go_back_plan =
        resolve_direct_navigation_readback_command("back", "req-back", &[String::from("go_back")])
            .expect("back command should normalize");

    assert_eq!(go_back_plan.intent.name, IntentName::GoBack);
    assert_eq!(go_back_plan.selected_skills, vec![String::from("go_back")]);
    assert_eq!(go_back_plan.steps.len(), 1);
    assert_eq!(go_back_plan.steps[0].tool_name, ToolName::GoBack);
    assert_eq!(
        go_back_plan.steps[0].arguments.get("steps"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        go_back_plan.steps[0].arguments.get("wait_for_load_state"),
        Some(&serde_json::json!(LoadState::Load))
    );

    let reload_plan = resolve_direct_navigation_readback_command(
        "refesh page",
        "req-reload",
        &[String::from("reload_page")],
    )
    .expect("reload command should normalize");

    assert_eq!(reload_plan.intent.name, IntentName::ReloadPage);
    assert_eq!(
        reload_plan.selected_skills,
        vec![String::from("reload_page")]
    );
    assert_eq!(reload_plan.steps[0].tool_name, ToolName::ReloadPage);
    assert_eq!(
        reload_plan.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(ReloadMode::Standard))
    );
}

#[test]
fn resolve_direct_navigation_readback_command_builds_reading_and_stop_plans() {
    let next_plan = resolve_direct_navigation_readback_command(
        "continue reading",
        "req-next",
        &[String::from("read_next")],
    )
    .expect("next command should normalize");

    assert_eq!(next_plan.intent.name, IntentName::ReadNext);
    assert_eq!(next_plan.selected_skills, vec![String::from("read_next")]);
    assert_eq!(next_plan.steps[0].tool_name, ToolName::ReadNextRegion);
    assert_eq!(
        next_plan.steps[0].arguments.get("interruption_mode"),
        Some(&serde_json::json!(NarrationInterruptionMode::Interrupt))
    );

    let previous_plan = resolve_direct_navigation_readback_command(
        "prevous section",
        "req-previous",
        &[String::from("read_previous")],
    )
    .expect("previous command should normalize");

    assert_eq!(previous_plan.intent.name, IntentName::ReadPrevious);
    assert_eq!(
        previous_plan.selected_skills,
        vec![String::from("read_previous")]
    );
    assert_eq!(
        previous_plan.steps[0].tool_name,
        ToolName::ReadPreviousRegion
    );

    let stop_plan = resolve_direct_navigation_readback_command(
        "stpo reading",
        "req-stop",
        &[String::from("stop_reading")],
    )
    .expect("stop command should normalize");

    assert_eq!(stop_plan.intent.name, IntentName::Stop);
    assert_eq!(
        stop_plan.selected_skills,
        vec![String::from("stop_reading")]
    );
    assert_eq!(stop_plan.steps[0].tool_name, ToolName::StopSpeaking);
}

#[test]
fn resolve_direct_voice_input_command_builds_start_and_stop_listening_plans() {
    let start_plan = resolve_direct_voice_input_command(
        "start listening",
        "req-start-listening",
        &[String::from("start_listening")],
    )
    .expect("start listening command should normalize");

    assert_eq!(start_plan.intent.name, IntentName::StartListening);
    assert_eq!(
        start_plan.selected_skills,
        vec![String::from("start_listening")]
    );
    assert_eq!(start_plan.steps[0].tool_name, ToolName::StartListening);

    let stop_plan = resolve_direct_voice_input_command(
        "stop listenin",
        "req-stop-listening",
        &[String::from("stop_listening")],
    )
    .expect("stop listening command should normalize");

    assert_eq!(stop_plan.intent.name, IntentName::StopListening);
    assert_eq!(
        stop_plan.selected_skills,
        vec![String::from("stop_listening")]
    );
    assert_eq!(stop_plan.steps[0].tool_name, ToolName::StopListening);
}

#[test]
fn resolve_direct_voice_input_command_builds_transcribe_plan() {
    let planner_output = resolve_direct_voice_input_command(
        "what did i just say",
        "req-transcribe",
        &[String::from("transcribe_command")],
    )
    .expect("transcribe command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::TranscribeCommand);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("transcribe_command")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(
        planner_output.steps[0].tool_name,
        ToolName::TranscribeCommand
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("stop_mode"),
        Some(&serde_json::json!(TranscriptionStopMode::AutoStop))
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("max_duration_ms"),
        Some(&serde_json::Value::Null)
    );
}

#[test]
fn resolve_direct_open_url_command_normalizes_spoken_and_absolute_urls() {
    let spoken_plan = resolve_direct_open_url_command(
        "open github dot com slash features",
        "req-open-spoken",
        &[String::from("open_url")],
    )
    .expect("spoken open-url command should normalize");

    assert_eq!(spoken_plan.intent.name, IntentName::OpenUrl);
    assert_eq!(spoken_plan.selected_skills, vec![String::from("open_url")]);
    assert_eq!(spoken_plan.steps.len(), 1);
    assert_eq!(spoken_plan.steps[0].tool_name, ToolName::OpenUrl);
    assert_eq!(
        spoken_plan.steps[0].arguments.get("url"),
        Some(&serde_json::json!("https://github.com/features"))
    );
    assert_eq!(
        spoken_plan.steps[0].arguments.get("wait_for_load_state"),
        Some(&serde_json::json!(LoadState::Load))
    );

    let localhost_plan = resolve_direct_open_url_command(
        "visit localhost colon 3000",
        "req-open-localhost",
        &[String::from("open_url")],
    )
    .expect("localhost command should normalize");

    assert_eq!(
        localhost_plan.steps[0].arguments.get("url"),
        Some(&serde_json::json!("http://localhost:3000"))
    );

    let absolute_plan = resolve_direct_open_url_command(
        "go to https://example.com/docs",
        "req-open-absolute",
        &[String::from("open_url")],
    )
    .expect("absolute open-url command should normalize");

    assert_eq!(
        absolute_plan.steps[0].arguments.get("url"),
        Some(&serde_json::json!("https://example.com/docs"))
    );
}
