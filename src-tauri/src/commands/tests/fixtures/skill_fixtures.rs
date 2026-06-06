use super::*;

pub fn resolve_planner_skill_fixture(
    fixture: &PlannerSkillFixture,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    match fixture.resolver {
        PlannerSkillFixtureResolver::Audio => resolve_direct_audio_command(
            fixture.transcript,
            fixture.name,
            fixture.agent_state.audio.playback_volume,
            fixture.agent_state.audio.playback_speed,
            active_skill_names,
        ),
        PlannerSkillFixtureResolver::NavigationReadback => {
            resolve_direct_navigation_readback_command(
                fixture.transcript,
                fixture.name,
                active_skill_names,
            )
        }
        PlannerSkillFixtureResolver::ReadPage => resolve_direct_read_page_command(
            fixture.transcript,
            fixture.name,
            fixture.page_model.as_ref(),
            &fixture.agent_state,
            active_skill_names,
        ),
        PlannerSkillFixtureResolver::StatusQuery => resolve_direct_status_query_command(
            fixture.transcript,
            fixture.name,
            &fixture.agent_state,
            &fixture_runtime_status(&fixture.agent_state),
            active_skill_names,
        ),
    }
}

pub fn assert_planner_skill_fixture(fixture: PlannerSkillFixture) {
    let available_tools = planner_available_tools();
    let selection = build_planner_skill_selection(None, None, fixture.transcript, &available_tools);
    let expected_selected_skills = fixture
        .expected_selected_skills
        .iter()
        .map(|skill| String::from(*skill))
        .collect::<Vec<_>>();
    let relevant_skill_names = selection
        .relevant_skill_summaries
        .iter()
        .map(|summary| summary.name.clone())
        .collect::<Vec<_>>();

    for expected_skill in &expected_selected_skills {
        assert!(
            selection
                .active_skill_names
                .iter()
                .any(|active_name| active_name == expected_skill),
            "fixture {} should have active skill {expected_skill}",
            fixture.name
        );
        assert!(
            relevant_skill_names
                .iter()
                .any(|skill_name| skill_name == expected_skill),
            "fixture {} should rank skill {expected_skill}; got {:?}",
            fixture.name,
            relevant_skill_names
        );
    }

    let planner_output = resolve_planner_skill_fixture(&fixture, &selection.active_skill_names)
        .unwrap_or_else(|| panic!("fixture {} should resolve directly", fixture.name));

    assert_eq!(
        planner_output.intent.name, fixture.expected_intent,
        "fixture {} resolved unexpected intent",
        fixture.name
    );
    assert_eq!(
        planner_output.selected_skills, expected_selected_skills,
        "fixture {} selected unexpected skills",
        fixture.name
    );

    let planned_tool_sequence = planner_output
        .steps
        .iter()
        .map(|step| step.tool_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        planned_tool_sequence, fixture.expected_tool_sequence,
        "fixture {} planned unexpected tool sequence",
        fixture.name
    );

    validate_planner_output(
        &planner_output,
        &available_tools,
        &selection.active_skill_names,
    )
    .unwrap_or_else(|error| panic!("fixture {} should validate, got {error:?}", fixture.name));

    let mut executor = MockExecutor::default();
    let outcome =
        execute_planner_output(&mut executor, String::from(fixture.name), &planner_output);
    let trace = match outcome {
        ExecutionOutcome::Complete { trace } => trace,
        other => panic!(
            "fixture {} should execute to completion, got {other:?}",
            fixture.name
        ),
    };
    let executed_tool_sequence = trace
        .tool_results
        .iter()
        .map(|result| result.tool_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        executed_tool_sequence, fixture.expected_tool_sequence,
        "fixture {} executed unexpected tool sequence",
        fixture.name
    );
}
