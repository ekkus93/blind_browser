use super::*;

pub(super) fn execute_get_agent_state(
    ex: &mut MockExecutor,
    input: GetAgentStateInput,
) -> ToolResult<AgentStateData> {
    ToolResult::success(
        ToolName::GetAgentState,
        input.request_id,
        AgentStateData {
            page_id: None,
            url: Some(String::from("https://example.com")),
            title: Some(String::from("Example")),
            browser_visibility: ex.current_browser_visibility(),
            browser_history: ex.current_browser_history(),
            narration_cursor: Some(NarrationCursor::default()),
            speaking: false,
            listening_state: ex.current_listening_state(),
            audio: ex.audio.clone(),
            last_transcript: if input.include_last_transcript {
                ex.current_last_transcript()
            } else {
                None
            },
            last_tool_call: Some(LastToolCallSummary {
                request_id: String::from("req-5"),
                tool_name: ToolName::GetAgentState,
                ok: true,
                observation_summary: vec![String::from("agent state read")],
            }),
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
                api_key_reference_error: None,
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
                api_key_reference_error: None,
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
                api_key_reference_error: None,
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
        vec![String::from("agent state read")],
    )
}

pub(super) fn execute_get_runtime_status(
    ex: &mut MockExecutor,
    input: GetRuntimeStatusInput,
) -> ToolResult<GetRuntimeStatusData> {
    ToolResult::success(
        ToolName::GetRuntimeStatus,
        input.request_id,
        GetRuntimeStatusData {
            page_id: None,
            url: Some(String::from("https://example.com")),
            title: Some(String::from("Example")),
            browser_visibility: ex.current_browser_visibility(),
            browser_history: ex.current_browser_history(),
            listening_state: ex.current_listening_state(),
            speaking: false,
            audio: ex.audio.clone(),
            pending_confirmation_id: None,
            pending_plan_execution: None,
            provider_modes: if input.include_provider_modes {
                Some(ProviderSelectionStatus {
                    planner_mode: ProviderMode::Remote,
                    tts_mode: ProviderMode::Local,
                    asr_mode: ProviderMode::Local,
                })
            } else {
                None
            },
        },
        vec![String::from("runtime status read")],
    )
}

pub(super) fn execute_confirm_action(
    ex: &mut MockExecutor,
    input: ConfirmActionInput,
) -> ToolResult<ConfirmActionData> {
    ex.last_confirmation_prompt = Some(input.prompt_text.clone());
    ToolResult::success(
        ToolName::ConfirmAction,
        input.request_id,
        ConfirmActionData {
            confirmation_id: String::from("confirm-1"),
            prompt_text: input.prompt_text,
            confirmed: None,
            timed_out: false,
        },
        vec![input.reason],
    )
}

pub(super) fn execute_report_result(
    ex: &mut MockExecutor,
    input: ReportResultInput,
) -> ToolResult<ReportResultData> {
    let data = ReportResultData {
        status: input.status,
        summary: input.summary,
        next_recommended_action: input.next_recommended_action,
        user_message: input.user_message,
    };
    ex.last_report_result = Some(data.clone());
    ToolResult::success(
        ToolName::ReportResult,
        input.request_id,
        data,
        vec![String::from("reported final result")],
    )
}
