use super::*;

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
            skill_discovery_diagnostics: Default::default(),
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
