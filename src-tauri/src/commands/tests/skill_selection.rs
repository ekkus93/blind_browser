use super::*;

#[test]
fn parse_skill_document_rejects_invalid_frontmatter_cases() {
    let available_tool_names = planner_available_tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    let cases = [
        (
            "missing frontmatter",
            "Use the browser state to guide the user.",
            "SKILL.md is missing a YAML frontmatter block",
        ),
        (
            "unsupported field",
            r#"---
name: browse_help
description: Help the user browse
unsupported: true
---
Use the browser state to guide the user."#,
            "unsupported frontmatter field 'unsupported'",
        ),
        (
            "missing description",
            r#"---
name: browse_help
allowed_tools:
  - get_runtime_status
---
Use the browser state to guide the user."#,
            "skill frontmatter is missing description",
        ),
        (
            "unknown tool",
            r#"---
name: browse_help
description: Help the user browse
allowed_tools:
  - not_a_real_tool
---
Use the browser state to guide the user."#,
            "unknown tool 'not_a_real_tool'",
        ),
    ];

    for (label, content, expected_error) in cases {
        let error = parse_skill_document(content, SkillSource::Project, &available_tool_names)
            .expect_err("invalid skill document should be rejected");
        assert!(
            error.contains(expected_error),
            "case {label} expected error containing {expected_error:?}, got {error:?}"
        );
    }
}

#[test]
fn discover_skills_prefers_higher_precedence_duplicate_skill_names() {
    let project_root = unique_temp_path("skill-project");
    let user_root = unique_temp_path("skill-user");
    let project_skills_root = project_root.join(".pi").join("skills");
    std::fs::create_dir_all(&project_skills_root).expect("project skill root should be created");
    std::fs::create_dir_all(&user_root).expect("user skill root should be created");

    write_skill_document(
        &project_skills_root,
        "open_url",
        r#"---
name: open_url
description: Project-local open URL workflow
priority: 90
allowed_tools:
  - open_url
intent_tags:
  - intent:OpenUrl
---
Project skills should override lower-precedence copies."#,
    );
    write_skill_document(
        &user_root,
        "open_url",
        r#"---
name: open_url
description: User-level open URL workflow
priority: 10
allowed_tools:
  - open_url
intent_tags:
  - intent:OpenUrl
---
User skills should lose to project-local copies."#,
    );

    let available_tools = planner_available_tools();
    let loaded_skills = discover_skills(
        Some(project_root.as_path()),
        Some(user_root.as_path()),
        &available_tools,
    );
    let matching_skills = loaded_skills
        .skills
        .iter()
        .filter(|skill| skill.summary.name == "open_url")
        .collect::<Vec<_>>();

    assert_eq!(
        matching_skills.len(),
        1,
        "duplicate skill names should resolve to one loaded skill"
    );
    let resolved = matching_skills[0];
    assert_eq!(resolved.source, SkillSource::Project);
    assert_eq!(
        resolved.summary.description,
        "Project-local open URL workflow"
    );
    assert_eq!(resolved.summary.priority, 90);
    assert_eq!(
        resolved.body,
        "Project skills should override lower-precedence copies."
    );

    std::fs::remove_dir_all(&project_root).expect("project temp directory should be removed");
    std::fs::remove_dir_all(&user_root).expect("user temp directory should be removed");
}

#[test]
fn build_planner_skill_selection_ranks_custom_skills_and_caps_to_top_n() {
    let project_root = unique_temp_path("skill-ranking-project");
    let project_skills_root = project_root.join(".pi").join("skills");
    std::fs::create_dir_all(&project_skills_root).expect("project skill root should be created");

    write_skill_document(
        &project_skills_root,
        "open_dashboard_exact",
        r#"---
name: open_dashboard_exact
description: Open the dashboard URL directly
priority: 10
allowed_tools:
  - open_url
intent_tags:
  - intent:OpenUrl
---
Open the dashboard URL directly when the user asks to open the dashboard."#,
    );
    write_skill_document(
        &project_skills_root,
        "open_dashboard_priority",
        r#"---
name: open_dashboard_priority
description: Open the dashboard URL quickly
priority: 200
allowed_tools:
  - open_url
---
Use this when dashboard navigation should stay fast."#,
    );
    write_skill_document(
        &project_skills_root,
        "dashboard_url_reference",
        r#"---
name: dashboard_url_reference
description: Explain the dashboard URL steps
priority: 50
---
Guide the user through the dashboard URL flow."#,
    );
    write_skill_document(
        &project_skills_root,
        "dashboard_helper",
        r#"---
name: dashboard_helper
description: Help with the dashboard
priority: 0
---
This helper mentions dashboard guidance only."#,
    );
    write_skill_document(
        &project_skills_root,
        "completely_unrelated",
        r#"---
name: completely_unrelated
description: Explain OCR fallback tuning
priority: 500
---
Use OCR threshold tuning when extraction fails."#,
    );

    let available_tools = planner_available_tools();
    let selection = build_planner_skill_selection(
        Some(project_root.as_path()),
        None,
        "please open the dashboard url",
        &available_tools,
    );
    let ranked_skill_names = selection
        .relevant_skill_summaries
        .iter()
        .map(|summary| summary.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ranked_skill_names,
        vec![
            "open_dashboard_exact",
            "open_dashboard_priority",
            "dashboard_url_reference",
        ]
    );
    assert_eq!(
        selection.relevant_skill_summaries.len(),
        MAX_SELECTED_PLANNER_SKILLS
    );
    assert!(!selection
        .relevant_skill_summaries
        .iter()
        .any(|summary| summary.name == "dashboard_helper"));
    assert!(!selection
        .relevant_skill_summaries
        .iter()
        .any(|summary| summary.name == "completely_unrelated"));

    std::fs::remove_dir_all(&project_root).expect("project temp directory should be removed");
}

#[test]
fn build_planner_skill_selection_prefers_matching_bundled_skill() {
    let available_tools = planner_available_tools();
    let selection = build_planner_skill_selection(
        None,
        None,
        "please go back to the previous page",
        &available_tools,
    );

    assert!(selection
        .active_skill_names
        .iter()
        .any(|name| name == "go_back"));
    assert_eq!(
        selection
            .relevant_skill_summaries
            .first()
            .map(|skill| skill.name.as_str()),
        Some("go_back")
    );
}

#[test]
fn build_planner_skill_selection_prefers_set_tts_voice_skill_for_voice_commands() {
    let available_tools = planner_available_tools();
    let selection =
        build_planner_skill_selection(None, None, "change the voice to Bruno", &available_tools);

    assert!(selection
        .active_skill_names
        .iter()
        .any(|name| name == "set_tts_voice"));
    assert_eq!(
        selection
            .relevant_skill_summaries
            .first()
            .map(|skill| skill.name.as_str()),
        Some("set_tts_voice")
    );
}

#[test]
fn build_planner_skill_selection_selects_expected_bundled_skills_for_representative_tasks() {
    let available_tools = planner_available_tools();
    let cases = [
        ("open github dot com slash features", "open_url"),
        ("please go back to the previous page", "go_back"),
        ("read this page", "read_page"),
        ("what page am i on", "get_current_url"),
        ("continue reading", "read_next"),
        ("are you listening", "announce_state"),
        ("start listening", "start_listening"),
        ("what's the playback speed", "get_playback_speed"),
        ("change the voice to Bruno", "set_tts_voice"),
        ("show the browser window", "toggle_browser_visibility"),
    ];

    for (transcript, expected_skill_name) in cases {
        let selection = build_planner_skill_selection(None, None, transcript, &available_tools);
        let ranked_skill_names = selection
            .relevant_skill_summaries
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<Vec<_>>();

        assert!(
            selection
                .active_skill_names
                .iter()
                .any(|name| name == expected_skill_name),
            "transcript {transcript:?} should expose bundled skill {expected_skill_name}"
        );
        assert_eq!(
            selection
                .relevant_skill_summaries
                .first()
                .map(|skill| skill.name.as_str()),
            Some(expected_skill_name),
            "transcript {transcript:?} ranked unexpected bundled skill: {ranked_skill_names:?}"
        );
    }
}

#[test]
fn bundled_skills_cover_planner_visible_command_family_intents() {
    let available_tool_names = registered_tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    let bundled_skills = parse_bundled_skills(BUNDLED_SKILLS_MARKDOWN, &available_tool_names)
        .expect("bundled skills should parse");
    let bundled_intents = bundled_skills
        .iter()
        .flat_map(|skill| skill.summary.intent_tags.iter())
        .filter_map(|tag| tag.strip_prefix("intent:"))
        .map(parse_intent_name_value)
        .collect::<Result<HashSet<_>, _>>()
        .expect("bundled intent tags should parse");

    let required_intents = [
        IntentName::OpenUrl,
        IntentName::GoBack,
        IntentName::GoForward,
        IntentName::ReloadPage,
        IntentName::GetCurrentUrl,
        IntentName::ReadPage,
        IntentName::ReadTitle,
        IntentName::ReadNext,
        IntentName::ReadPrevious,
        IntentName::Repeat,
        IntentName::Stop,
        IntentName::StartListening,
        IntentName::StopListening,
        IntentName::TranscribeCommand,
        IntentName::SetTtsVoice,
        IntentName::SetPlaybackVolume,
        IntentName::GetPlaybackVolume,
        IntentName::SetPlaybackSpeed,
        IntentName::GetPlaybackSpeed,
        IntentName::SetBrowserVisibility,
        IntentName::GetStatus,
        IntentName::FindElement,
        IntentName::ClickElement,
        IntentName::Scroll,
        IntentName::OcrRecovery,
    ];

    let missing = required_intents
        .into_iter()
        .filter(|intent| !bundled_intents.contains(intent))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "bundled skills are missing explicit intent coverage for {missing:?}"
    );
}

#[test]
fn canonical_planner_output_examples_validate_against_current_contract() {
    let available_tools = planner_available_tools();
    let active_skill_names =
        build_planner_skill_selection(None, None, "", &available_tools).active_skill_names;

    for (example_name, planner_output) in canonical_planner_output_examples() {
        validate_planner_output(&planner_output, &available_tools, &active_skill_names)
            .unwrap_or_else(|error| {
                panic!("canonical planner example '{example_name}' should validate: {error:?}")
            });
    }
}

#[test]
fn planner_skill_regression_fixtures_cover_representative_direct_command_flows() {
    let fixtures = vec![
        PlannerSkillFixture {
            name: "fixture-set-volume",
            transcript: "set volume to 70 percent",
            resolver: PlannerSkillFixtureResolver::Audio,
            agent_state: fixture_agent_state(),
            page_model: None,
            expected_intent: IntentName::SetPlaybackVolume,
            expected_selected_skills: vec!["set_volume"],
            expected_tool_sequence: vec![ToolName::SetPlaybackVolume, ToolName::ReportResult],
        },
        PlannerSkillFixture {
            name: "fixture-go-back",
            transcript: "back",
            resolver: PlannerSkillFixtureResolver::NavigationReadback,
            agent_state: fixture_agent_state(),
            page_model: None,
            expected_intent: IntentName::GoBack,
            expected_selected_skills: vec!["go_back"],
            expected_tool_sequence: vec![ToolName::GoBack],
        },
        PlannerSkillFixture {
            name: "fixture-read-page-extract",
            transcript: "read page",
            resolver: PlannerSkillFixtureResolver::ReadPage,
            agent_state: fixture_agent_state(),
            page_model: Some(fixture_page_model_without_regions()),
            expected_intent: IntentName::ReadPage,
            expected_selected_skills: vec!["read_page"],
            expected_tool_sequence: vec![ToolName::ExtractPageModel, ToolName::ReadNextRegion],
        },
        PlannerSkillFixture {
            name: "fixture-current-url",
            transcript: "what page am i on",
            resolver: PlannerSkillFixtureResolver::StatusQuery,
            agent_state: fixture_agent_state(),
            page_model: None,
            expected_intent: IntentName::GetCurrentUrl,
            expected_selected_skills: vec!["get_current_url"],
            expected_tool_sequence: vec![ToolName::GetAgentState, ToolName::ReportResult],
        },
    ];

    for fixture in fixtures {
        assert_planner_skill_fixture(fixture);
    }
}

#[test]
fn planner_skill_regression_fixtures_cover_problematic_page_shapes() {
    let fixtures = vec![
        PlannerSkillFixture {
            name: "problematic-article-read-page",
            transcript: "read page",
            resolver: PlannerSkillFixtureResolver::ReadPage,
            agent_state: fixture_agent_state_for_page(
                "Metro news | Night trains finally return",
                "https://news.example.com/city/night-trains-return",
            ),
            page_model: Some(fixture_problematic_article_page_without_regions()),
            expected_intent: IntentName::ReadPage,
            expected_selected_skills: vec!["read_page"],
            expected_tool_sequence: vec![ToolName::ExtractPageModel, ToolName::ReadNextRegion],
        },
        PlannerSkillFixture {
            name: "problematic-docs-current-url",
            transcript: "what page am i on",
            resolver: PlannerSkillFixtureResolver::StatusQuery,
            agent_state: fixture_problematic_docs_agent_state(),
            page_model: None,
            expected_intent: IntentName::GetCurrentUrl,
            expected_selected_skills: vec!["get_current_url"],
            expected_tool_sequence: vec![ToolName::GetAgentState, ToolName::ReportResult],
        },
    ];

    for fixture in fixtures {
        assert_planner_skill_fixture(fixture);
    }
}

fn bundled_parser_tool_names() -> Vec<ToolName> {
    planner_available_tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect()
}

#[test]
fn parse_bundled_skills_accepts_the_shipped_bundle() {
    // Regression guard: the bundled docs/SKILLS.md must always parse, so a malformed
    // bundled skill fails CI before it can panic the app at startup.
    parse_bundled_skills(BUNDLED_SKILLS_MARKDOWN, &bundled_parser_tool_names())
        .expect("the shipped bundled skills must parse");
}

#[test]
fn parse_bundled_skills_rejects_invalid_requires_confirmation() {
    let markdown = "\
#### risky_skill
- intent_tags: `intent:submit_form`
- allowed_tools: `SubmitForm`
- requires_confirmation: maybe
- description: Submit a form.
";

    let error = parse_bundled_skills(markdown, &bundled_parser_tool_names())
        .expect_err("invalid requires_confirmation must fail bundled skill parsing");
    assert!(
        error.contains("requires_confirmation"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_bundled_skills_rejects_unknown_tool() {
    let markdown = "\
#### bad_tool_skill
- allowed_tools: `DefinitelyNotATool`
- requires_confirmation: false
- description: Bad tool.
";

    let error = parse_bundled_skills(markdown, &bundled_parser_tool_names())
        .expect_err("an unknown bundled tool must fail bundled skill parsing");
    assert!(
        error.contains("DefinitelyNotATool"),
        "unexpected error: {error}"
    );
}
