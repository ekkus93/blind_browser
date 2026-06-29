use super::*;

#[test]
fn provider_selection_status_round_trips_with_snake_case_modes() {
    let status = ProviderSelectionStatus {
        planner_mode: ProviderMode::Remote,
        tts_mode: ProviderMode::Local,
        asr_mode: ProviderMode::Local,
    };

    let serialized =
        serde_json::to_value(&status).expect("provider selection status should serialize");
    assert_eq!(
        serialized,
        serde_json::json!({
            "planner_mode": "remote",
            "tts_mode": "local",
            "asr_mode": "local"
        })
    );

    let round_tripped: ProviderSelectionStatus =
        serde_json::from_value(serialized).expect("provider selection status should deserialize");
    assert_eq!(round_tripped, status);
}

#[test]
fn shared_command_enums_round_trip_and_reject_invalid_variants() {
    fn assert_enum_round_trip<T>(value: T, expected: serde_json::Value, invalid: serde_json::Value)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let serialized = serde_json::to_value(&value).expect("enum should serialize");
        assert_eq!(serialized, expected);

        let round_tripped: T = serde_json::from_value(serialized).expect("enum should deserialize");
        assert_eq!(round_tripped, value);
        assert!(serde_json::from_value::<T>(invalid).is_err());
    }

    assert_enum_round_trip(
        NarrationInterruptionMode::Interrupt,
        serde_json::json!("Interrupt"),
        serde_json::json!("Pause"),
    );
    assert_enum_round_trip(
        NarrationBoundary::End,
        serde_json::json!("End"),
        serde_json::json!("Middle"),
    );
    assert_enum_round_trip(
        ElementVisibilityFilter::VisibleOnly,
        serde_json::json!("VisibleOnly"),
        serde_json::json!("HiddenOnly"),
    );
    assert_enum_round_trip(
        ReloadMode::Hard,
        serde_json::json!("Hard"),
        serde_json::json!("Soft"),
    );
    assert_enum_round_trip(
        ClickMode::Double,
        serde_json::json!("Double"),
        serde_json::json!("Triple"),
    );
    assert_enum_round_trip(
        TextEntryMode::Replace,
        serde_json::json!("Replace"),
        serde_json::json!("Overwrite"),
    );
    assert_enum_round_trip(
        TextEntrySubmitMode::Submit,
        serde_json::json!("Submit"),
        serde_json::json!("Enter"),
    );
    assert_enum_round_trip(
        TranscriptionStopMode::AutoStop,
        serde_json::json!("AutoStop"),
        serde_json::json!("ManualStop"),
    );
    assert_enum_round_trip(
        ScreenshotScope::FullPage,
        serde_json::json!("FullPage"),
        serde_json::json!("Region"),
    );
    assert_enum_round_trip(
        ReportStatus::NeedsFollowUp,
        serde_json::json!("NeedsFollowUp"),
        serde_json::json!("Retry"),
    );
    assert_enum_round_trip(
        BrowserVisibilityMode::Headless,
        serde_json::json!("Headless"),
        serde_json::json!("Minimized"),
    );
}

#[test]
fn get_runtime_status_result_matches_schema_with_provider_modes() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-runtime-schema"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-schema",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read runtime status with provider modes"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);
    assert!(result.ok);

    let serialized =
        serde_json::to_value(&result).expect("runtime status tool result should serialize");
    let schema = tool_output_schema(&ToolName::GetRuntimeStatus)
        .expect("get_runtime_status should expose an output schema");
    assert_json_matches_schema(&serialized, &schema)
        .expect("serialized get_runtime_status result should match its output schema");

    let provider_modes = serialized
        .get("data")
        .and_then(|data| data.get("provider_modes"))
        .expect("provider_modes should be present when requested");
    assert_eq!(
        provider_modes,
        &serde_json::json!({
            "planner_mode": "remote",
            "tts_mode": "local",
            "asr_mode": "local"
        })
    );
}

#[test]
fn get_runtime_status_reports_null_provider_modes_when_not_requested() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-runtime-no-provider-modes"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-no-provider-modes",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status without provider modes"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);
    assert!(result.ok);

    let serialized =
        serde_json::to_value(&result).expect("runtime status tool result should serialize");
    let provider_modes = serialized
        .get("data")
        .and_then(|data| data.get("provider_modes"))
        .expect("provider_modes field should still be present in serialized output");
    assert_eq!(provider_modes, &serde_json::Value::Null);
}
