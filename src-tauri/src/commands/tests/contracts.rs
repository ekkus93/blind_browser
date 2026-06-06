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

#[test]
fn tool_result_success_populates_common_envelope_fields() {
    let result = ToolResult::success(
        ToolName::GetRuntimeStatus,
        String::from("req-envelope-success"),
        GetRuntimeStatusData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com")),
            title: Some(String::from("Example")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            listening_state: ListeningState::default(),
            speaking: false,
            audio: RuntimeAudioState::default(),
            pending_confirmation_id: None,
            pending_plan_execution: None,
            provider_modes: None,
        },
        vec![String::from("runtime status read")],
    );

    assert!(result.ok);
    assert_eq!(result.tool_name, ToolName::GetRuntimeStatus);
    assert_eq!(result.request_id, "req-envelope-success");
    assert!(result.timestamp_ms > 0);
    assert!(result.error.is_none());
    assert!(result.warnings.is_empty());
    assert_eq!(
        result.observations,
        vec![String::from("runtime status read")]
    );
    assert_eq!(
        result.data.as_ref().and_then(|data| data.url.as_deref()),
        Some("https://example.com")
    );
}

#[test]
fn tool_result_failure_populates_common_envelope_fields() {
    let result: ToolResult<SetPlaybackVolumeData> = ToolResult::failure(
        ToolName::SetPlaybackVolume,
        String::from("req-envelope-failure"),
        ToolError {
            code: String::from("audio_update_failed"),
            message: String::from("failed to persist volume"),
            retryable: false,
            details: Some(serde_json::json!({
                "setting": "playback_volume"
            })),
        },
        vec![String::from("volume update failed")],
    );

    assert!(!result.ok);
    assert_eq!(result.tool_name, ToolName::SetPlaybackVolume);
    assert_eq!(result.request_id, "req-envelope-failure");
    assert!(result.timestamp_ms > 0);
    assert!(result.data.is_none());
    assert!(result.warnings.is_empty());
    assert_eq!(
        result.observations,
        vec![String::from("volume update failed")]
    );
    assert_eq!(
        result.error,
        Some(ToolError {
            code: String::from("audio_update_failed"),
            message: String::from("failed to persist volume"),
            retryable: false,
            details: Some(serde_json::json!({
                "setting": "playback_volume"
            })),
        })
    );
}

#[test]
fn serialized_tool_result_round_trips_with_warning_and_error_details() {
    let envelope = SerializedToolResult {
        ok: false,
        tool_name: ToolName::RunOcr,
        request_id: String::from("req-envelope-roundtrip"),
        timestamp_ms: 1_234_567,
        data: None,
        error: Some(ToolError {
            code: String::from("ocr_failed"),
            message: String::from("OCR provider was unavailable"),
            retryable: true,
            details: Some(serde_json::json!({
                "image_id": "image-1"
            })),
        }),
        warnings: vec![ToolWarning {
            code: String::from("low_contrast"),
            message: String::from("Image contrast was low."),
        }],
        observations: vec![String::from("OCR could not complete.")],
    };

    let serialized =
        serde_json::to_value(&envelope).expect("serialized tool result should serialize");
    assert_eq!(
        serialized,
        serde_json::json!({
            "ok": false,
            "tool_name": "RunOcr",
            "request_id": "req-envelope-roundtrip",
            "timestamp_ms": 1234567,
            "data": null,
            "error": {
                "code": "ocr_failed",
                "message": "OCR provider was unavailable",
                "retryable": true,
                "details": {
                    "image_id": "image-1"
                }
            },
            "warnings": [
                {
                    "code": "low_contrast",
                    "message": "Image contrast was low."
                }
            ],
            "observations": ["OCR could not complete."]
        })
    );

    let round_tripped: SerializedToolResult =
        serde_json::from_value(serialized).expect("serialized tool result should deserialize");
    assert_eq!(round_tripped, envelope);
}

#[test]
fn typed_tool_result_deserializes_common_envelope_and_payload() {
    let result: ToolResult<SetPlaybackSpeedData> = serde_json::from_value(serde_json::json!({
        "ok": true,
        "tool_name": "SetPlaybackSpeed",
        "request_id": "req-envelope-typed",
        "timestamp_ms": 987654,
        "data": {
            "playback_speed": 1.25,
            "changed": true
        },
        "error": null,
        "warnings": [
            {
                "code": "rounded_value",
                "message": "The requested speed was rounded."
            }
        ],
        "observations": ["Updated the playback speed setting."]
    }))
    .expect("typed tool result should deserialize");

    assert!(result.ok);
    assert_eq!(result.tool_name, ToolName::SetPlaybackSpeed);
    assert_eq!(result.request_id, "req-envelope-typed");
    assert_eq!(result.timestamp_ms, 987_654);
    assert_eq!(
        result.data,
        Some(SetPlaybackSpeedData {
            playback_speed: 1.25,
            changed: true,
        })
    );
    assert!(result.error.is_none());
    assert_eq!(
        result.warnings,
        vec![ToolWarning {
            code: String::from("rounded_value"),
            message: String::from("The requested speed was rounded."),
        }]
    );
    assert_eq!(
        result.observations,
        vec![String::from("Updated the playback speed setting.")]
    );
}

#[test]
fn shared_contract_enums_serialize_expected_variants() {
    assert_eq!(serde_json::json!(NarrationInterruptionMode::Queue), "Queue");
    assert_eq!(
        serde_json::json!(NarrationInterruptionMode::Interrupt),
        "Interrupt"
    );
    assert_eq!(serde_json::json!(NarrationBoundary::None), "None");
    assert_eq!(serde_json::json!(NarrationBoundary::Start), "Start");
    assert_eq!(serde_json::json!(NarrationBoundary::End), "End");
    assert_eq!(serde_json::json!(ElementVisibilityFilter::All), "All");
    assert_eq!(
        serde_json::json!(ElementVisibilityFilter::VisibleOnly),
        "VisibleOnly"
    );
    assert_eq!(serde_json::json!(ReloadMode::Standard), "Standard");
    assert_eq!(serde_json::json!(ReloadMode::Hard), "Hard");
    assert_eq!(serde_json::json!(ClickMode::Single), "Single");
    assert_eq!(serde_json::json!(ClickMode::Double), "Double");
    assert_eq!(serde_json::json!(TextEntryMode::Append), "Append");
    assert_eq!(serde_json::json!(TextEntryMode::Replace), "Replace");
    assert_eq!(
        serde_json::json!(TextEntrySubmitMode::KeepEditing),
        "KeepEditing"
    );
    assert_eq!(serde_json::json!(TextEntrySubmitMode::Submit), "Submit");
    assert_eq!(
        serde_json::json!(TranscriptionStopMode::KeepListening),
        "KeepListening"
    );
    assert_eq!(
        serde_json::json!(TranscriptionStopMode::AutoStop),
        "AutoStop"
    );
    assert_eq!(serde_json::json!(ScreenshotScope::Viewport), "Viewport");
    assert_eq!(serde_json::json!(ScreenshotScope::FullPage), "FullPage");
    assert_eq!(serde_json::json!(RemoteProviderLabel::OpenAi), "OpenAI");
    assert_eq!(serde_json::json!(RemoteProviderLabel::Ollama), "Ollama");
    assert_eq!(
        serde_json::json!(LocalTtsBackend::KittenTtsRs),
        "kitten_tts_rs"
    );
    assert_eq!(serde_json::json!(LocalAsrBackend::Whisper), "whisper");
    assert_eq!(serde_json::json!(RemoteTtsAudioFormat::Wav), "wav");
    assert_eq!(
        serde_json::json!(crate::page_model::ElementRole::Landmark),
        "Landmark"
    );
    assert_eq!(
        serde_json::json!(crate::page_model::ElementRole::Other),
        "Other"
    );
    assert_eq!(
        serde_json::json!(crate::page_model::RegionSource::Mixed),
        "Mixed"
    );
    assert_eq!(
        serde_json::json!(crate::page_model::ExtractionSource::Merged),
        "Merged"
    );
    assert_eq!(
        serde_json::json!(ReportStatus::NeedsFollowUp),
        "NeedsFollowUp"
    );
}

#[test]
fn canonical_planner_output_examples_serialize_expected_strings_and_fields() {
    let examples = canonical_planner_output_examples();

    let set_volume = serde_json::to_value(
        examples
            .get("set_playback_volume")
            .expect("set_playback_volume example should exist"),
    )
    .expect("planner example should serialize");
    assert_eq!(
        set_volume.pointer("/intent/name"),
        Some(&serde_json::json!("SetPlaybackVolume"))
    );
    assert_eq!(
        set_volume.pointer("/steps/0/arguments/request_id"),
        Some(&serde_json::json!("example-set-volume"))
    );
    assert_eq!(
        set_volume.pointer("/steps/0/arguments/volume"),
        Some(&serde_json::json!(0.7))
    );
    assert_eq!(
        set_volume.pointer("/steps/0/on_success"),
        Some(&serde_json::json!({
            "NextStep": {
                "step_id": "report-playback-volume"
            }
        }))
    );
    assert_eq!(
        set_volume.pointer("/steps/1/on_success"),
        Some(&serde_json::json!("Complete"))
    );

    let ready_click = serde_json::to_value(
        examples
            .get("click_element_ready")
            .expect("click_element_ready example should exist"),
    )
    .expect("planner example should serialize");
    assert_eq!(
        ready_click.pointer("/status"),
        Some(&serde_json::json!("Ready"))
    );
    assert_eq!(
        ready_click.pointer("/steps/0/tool_name"),
        Some(&serde_json::json!("ClickElement"))
    );
    assert_eq!(
        ready_click.pointer("/steps/0/arguments/element_id"),
        Some(&serde_json::json!("link-help"))
    );

    let needs_confirmation = serde_json::to_value(
        examples
            .get("click_element_with_confirmation")
            .expect("click_element_with_confirmation example should exist"),
    )
    .expect("planner example should serialize");
    assert_eq!(
        needs_confirmation.pointer("/status"),
        Some(&serde_json::json!("NeedsConfirmation"))
    );
    assert_eq!(
        needs_confirmation.pointer("/steps/0/on_success"),
        Some(&serde_json::json!("RequestConfirmation"))
    );
    assert_eq!(
        needs_confirmation.pointer("/steps/1/arguments/element_id"),
        Some(&serde_json::json!("button-submit"))
    );
}

#[test]
fn canonical_planner_output_examples_match_generated_planner_output_schema() {
    let schema = planner_output_schema();

    for (example_name, planner_output) in canonical_planner_output_examples() {
        let serialized = serde_json::to_value(&planner_output)
            .expect("planner example should serialize to JSON");
        assert_json_matches_schema(&serialized, &schema).unwrap_or_else(|error| {
            panic!(
                "canonical planner example '{example_name}' should match generated planner schema: {error}"
            )
        });
    }
}

#[test]
fn planner_output_round_trips_with_confirmation_metadata_and_matches_schema() {
    let planner_output = canonical_planner_output_examples()
        .remove("click_element_with_confirmation")
        .expect("click_element_with_confirmation example should exist");

    let serialized =
        serde_json::to_value(&planner_output).expect("planner output should serialize");
    assert_json_matches_schema(&serialized, &planner_output_schema())
        .expect("planner output should match generated planner schema");
    assert_eq!(
        serialized.pointer("/status"),
        Some(&serde_json::json!("NeedsConfirmation"))
    );
    assert_eq!(
        serialized.pointer("/requires_confirmation"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        serialized.pointer("/steps/0/on_success"),
        Some(&serde_json::json!("RequestConfirmation"))
    );

    let round_tripped: PlannerOutput =
        serde_json::from_value(serialized).expect("planner output should deserialize");
    assert_eq!(round_tripped, planner_output);
}

#[test]
fn canonical_planner_output_step_arguments_match_generated_tool_input_schemas() {
    for (example_name, planner_output) in canonical_planner_output_examples() {
        for step in &planner_output.steps {
            let schema = tool_input_schema(&step.tool_name).unwrap_or_else(|| {
                panic!(
                    "canonical planner example '{example_name}' uses tool {:?} without an input schema",
                    step.tool_name
                )
            });
            assert_json_matches_schema(&step.arguments, &schema).unwrap_or_else(|error| {
                panic!(
                    "canonical planner example '{example_name}' step '{}' arguments should match generated {:?} schema: {error}",
                    step.step_id,
                    step.tool_name
                )
            });
        }
    }
}

#[test]
fn planner_input_round_trips_with_nested_runtime_context_and_matches_schema() {
    let page_model = PageModel {
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("link-help"),
            dom_locator: Some(String::from("#help")),
            role: crate::page_model::ElementRole::Link,
            tag_name: String::from("a"),
            text: Some(String::from("Help")),
            accessible_name: Some(String::from("Help")),
            placeholder: None,
            href: Some(String::from("https://example.com/help")),
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
        ..fixture_page_model_without_regions()
    };

    let planner_input = PlannerInput {
        request_id: String::from("req-planner-roundtrip"),
        transcript: String::from("click the help link"),
        agent_state: fixture_agent_state(),
        safety: PlannerSafetySettings {
            confirmation_confidence_threshold: 0.9,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        },
        available_tools: planner_available_tools(),
        active_skill_names: vec![String::from("open_link_by_text")],
        relevant_skill_summaries: vec![SkillSummary {
            name: String::from("open_link_by_text"),
            description: String::from("Open a link by matching visible text."),
            intent_tags: vec![String::from("intent:ClickElement")],
            allowed_tools: Some(vec![ToolName::FindElement, ToolName::ClickElement]),
            requires_confirmation: false,
            priority: 80,
        }],
        page_snapshot: Some(PageSnapshotData {
            page_id: String::from("page-1"),
            url: String::from("https://example.com/article"),
            title: Some(String::from("Example article")),
            visible_text_excerpt: String::from("Example article body"),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("link-help"),
                dom_locator: Some(String::from("#help")),
                role: crate::page_model::ElementRole::Link,
                tag_name: String::from("a"),
                text: Some(String::from("Help")),
                accessible_name: Some(String::from("Help")),
                placeholder: None,
                href: Some(String::from("https://example.com/help")),
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
            scroll_y: 120.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
            document_height: 2400.0,
        }),
        page_model: Some(page_model),
        recent_tool_results: vec![PlannerToolHistoryEntry {
            tool_name: ToolName::GetAgentState,
            ok: true,
            observation_summary: vec![String::from("agent state read")],
        }],
    };

    let serialized = serde_json::to_value(&planner_input).expect("planner input should serialize");
    let schema = schema_json::<PlannerInput>();
    assert_json_matches_schema(&serialized, &schema)
        .expect("planner input should match generated planner input schema");
    assert_eq!(
        serialized.pointer("/agent_state/browser_history/current_entry_index"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        serialized.pointer("/relevant_skill_summaries/0/allowed_tools/1"),
        Some(&serde_json::json!("ClickElement"))
    );

    let round_tripped: PlannerInput =
        serde_json::from_value(serialized).expect("planner input should deserialize");
    assert_eq!(round_tripped, planner_input);
}

#[test]
fn planner_input_serializes_safety_settings_for_click_policy() {
    let planner_input = PlannerInput {
        request_id: String::from("req-planner"),
        transcript: String::from("click the help link"),
        agent_state: AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com")),
            title: Some(String::from("Example")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: None,
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(LocalTtsBackend::KittenTtsRs),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(LocalAsrBackend::Whisper),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(RemoteProviderLabel::OpenAi),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(RemoteProviderLabel::OpenAi),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                api_key_masked_value: None,
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(RemoteTtsAudioFormat::Wav),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(RemoteProviderLabel::OpenAi),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                api_key_masked_value: None,
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        },
        safety: PlannerSafetySettings {
            confirmation_confidence_threshold: 0.9,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        },
        available_tools: Vec::new(),
        active_skill_names: vec![String::from("open_link_by_text")],
        relevant_skill_summaries: Vec::new(),
        page_snapshot: None,
        page_model: None,
        recent_tool_results: Vec::new(),
    };

    let serialized = serde_json::to_value(&planner_input).expect("planner input should serialize");
    assert_eq!(
        serialized.pointer("/safety/allow_click_without_confirmation"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        serialized.pointer("/safety/always_confirm_submit"),
        Some(&serde_json::json!(true))
    );
}

#[test]
fn sample_serialized_tool_results_match_generated_tool_output_schemas() {
    let mut executor = MockExecutor::default();

    for step in sample_planned_steps_for_registered_tools() {
        let result = execute_planned_step(&mut executor, &step);
        let serialized =
            serde_json::to_value(&result).expect("serialized tool result should serialize");
        let schema = tool_output_schema(&step.tool_name).unwrap_or_else(|| {
            panic!(
                "sample tool result uses tool {:?} without an output schema",
                step.tool_name
            )
        });
        assert_json_matches_schema(&serialized, &schema).unwrap_or_else(|error| {
            panic!(
                "sample serialized {:?} result should match generated output schema: {error}",
                step.tool_name
            )
        });
    }
}

