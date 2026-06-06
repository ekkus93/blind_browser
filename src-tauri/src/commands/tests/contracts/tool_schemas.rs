use super::*;

#[test]
fn registered_tools_all_expose_input_schemas() {
    let missing = registered_tools()
        .into_iter()
        .filter(|tool| tool_input_schema(&tool.name).is_none())
        .map(|tool| tool.name)
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "registered tools missing input schemas: {missing:?}"
    );
}

#[test]
fn sample_planned_steps_match_generated_tool_input_schemas() {
    for step in sample_planned_steps_for_registered_tools() {
        let schema = tool_input_schema(&step.tool_name).unwrap_or_else(|| {
            panic!(
                "sample tool input uses tool {:?} without an input schema",
                step.tool_name
            )
        });
        assert_json_matches_schema(&step.arguments, &schema).unwrap_or_else(|error| {
            panic!(
                "sample {:?} arguments should match generated input schema: {error}",
                step.tool_name
            )
        });
        validate_planned_step_arguments(&step).unwrap_or_else(|error| {
            panic!(
                "sample {:?} arguments should pass runtime validator: {error:?}",
                step.tool_name
            )
        });
    }
}

#[test]
fn registered_tools_all_expose_output_schemas() {
    let missing = registered_tools()
        .into_iter()
        .filter(|tool| tool_output_schema(&tool.name).is_none())
        .map(|tool| tool.name)
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "registered tools missing output schemas: {missing:?}"
    );
}

#[test]
fn registered_tools_include_output_schema_refs() {
    for tool in registered_tools() {
        assert_eq!(
            tool.output_schema_ref,
            format!("schema://tool-output/{:?}", tool.name)
        );
    }
}
