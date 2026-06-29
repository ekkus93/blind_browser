use super::*;

#[test]
fn config_enums_round_trip_and_reject_invalid_variants() {
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
        ProviderMode::Remote,
        serde_json::json!("remote"),
        serde_json::json!("REMOTE"),
    );
    assert_enum_round_trip(
        RemoteProviderKind::OpenAi,
        serde_json::json!("OpenAi"),
        serde_json::json!("Anthropic"),
    );
    assert_enum_round_trip(
        RemoteTtsAudioFormat::Wav,
        serde_json::json!("wav"),
        serde_json::json!("mp3"),
    );
    assert_enum_round_trip(
        LocalTtsBackend::KittenTtsRs,
        serde_json::json!("kitten_tts_rs"),
        serde_json::json!("kitten"),
    );
    assert_enum_round_trip(
        LocalAsrBackend::Whisper,
        serde_json::json!("whisper"),
        serde_json::json!("deepgram"),
    );
    assert_enum_round_trip(
        SpeechFeedbackStyle::Detailed,
        serde_json::json!("Detailed"),
        serde_json::json!("Verbose"),
    );
}

#[test]
fn provider_configs_round_trip_through_json() {
    let providers = ProviderSelections {
        planner: ProviderSelection {
            mode: ProviderMode::Remote,
            remote_profile: Some(String::from("openai-default")),
            local_profile: None,
            failover_to_local: None,
        },
        tts: ProviderSelection {
            mode: ProviderMode::Local,
            remote_profile: Some(String::from("openai-tts-default")),
            local_profile: Some(String::from("kitten-default")),
            failover_to_local: Some(false),
        },
        asr: ProviderSelection {
            mode: ProviderMode::Remote,
            remote_profile: Some(String::from("openai-asr-default")),
            local_profile: Some(String::from("whisper-default")),
            failover_to_local: Some(true),
        },
    };
    let planner_profile = RemotePlannerProfile {
        provider: RemoteProviderKind::OpenAi,
        base_url: String::from("https://api.openai.com/v1"),
        model: String::from("gpt-4.1"),
        api_key: SecretRef::FromEnv {
            from_env: String::from("OPENAI_API_KEY"),
        },
        organization: Some(SecretRef::FromEnv {
            from_env: String::from("OPENAI_ORG_ID"),
        }),
        project: Some(String::from("blind-browser")),
        temperature_milli: 250,
        max_output_tokens: 1024,
        timeout_ms: 30_000,
    };
    let remote_tts_profile = RemoteTtsProfile {
        provider: RemoteProviderKind::OpenAi,
        base_url: String::from("https://api.openai.com/v1"),
        model: String::from("gpt-4o-mini-tts"),
        api_key: SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: String::from("blind-browser"),
                account: String::from("tts/openai-tts-default"),
            },
        },
        organization: None,
        project: Some(String::from("blind-browser")),
        voice: String::from("alloy"),
        audio_format: RemoteTtsAudioFormat::Wav,
        timeout_ms: 20_000,
    };
    let remote_asr_profile = RemoteAsrProfile {
        provider: RemoteProviderKind::OpenAi,
        base_url: String::from("https://api.openai.com/v1"),
        model: String::from("gpt-4o-mini-transcribe"),
        api_key: SecretRef::FromFile {
            from_file: String::from("/tmp/openai-asr.key"),
        },
        organization: None,
        project: Some(String::from("blind-browser")),
        language: Some(String::from("en")),
        temperature_milli: 0,
        timeout_ms: 20_000,
    };
    let local_tts_profile = LocalTtsProfile {
        backend: LocalTtsBackend::KittenTtsRs,
        model_id: String::from("default"),
        model_path: String::from("/models/kitten/default.onnx"),
        default_voice: String::from("Bruno"),
        sample_rate: 24_000,
    };
    let local_asr_profile = LocalAsrProfile {
        backend: LocalAsrBackend::Whisper,
        model_id: String::from("tiny"),
        model_path: String::from("/models/whisper/tiny.bin"),
        language: Some(String::from("en")),
        threads: 4,
    };

    let serialized = serde_json::json!({
        "providers": providers,
        "planner_profile": planner_profile,
        "remote_tts_profile": remote_tts_profile,
        "remote_asr_profile": remote_asr_profile,
        "local_tts_profile": local_tts_profile,
        "local_asr_profile": local_asr_profile
    });

    let round_tripped =
        serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(serialized.clone())
            .expect("provider config payload should deserialize as JSON object");

    let decoded_providers: ProviderSelections =
        serde_json::from_value(serialized.get("providers").cloned().unwrap())
            .expect("provider selections should deserialize");
    let decoded_planner_profile: RemotePlannerProfile =
        serde_json::from_value(serialized.get("planner_profile").cloned().unwrap())
            .expect("planner profile should deserialize");
    let decoded_remote_tts_profile: RemoteTtsProfile =
        serde_json::from_value(serialized.get("remote_tts_profile").cloned().unwrap())
            .expect("remote tts profile should deserialize");
    let decoded_remote_asr_profile: RemoteAsrProfile =
        serde_json::from_value(serialized.get("remote_asr_profile").cloned().unwrap())
            .expect("remote asr profile should deserialize");
    let decoded_local_tts_profile: LocalTtsProfile =
        serde_json::from_value(serialized.get("local_tts_profile").cloned().unwrap())
            .expect("local tts profile should deserialize");
    let decoded_local_asr_profile: LocalAsrProfile =
        serde_json::from_value(serialized.get("local_asr_profile").cloned().unwrap())
            .expect("local asr profile should deserialize");

    assert_eq!(decoded_providers, providers);
    assert_eq!(decoded_planner_profile, planner_profile);
    assert_eq!(decoded_remote_tts_profile, remote_tts_profile);
    assert_eq!(decoded_remote_asr_profile, remote_asr_profile);
    assert_eq!(decoded_local_tts_profile, local_tts_profile);
    assert_eq!(decoded_local_asr_profile, local_asr_profile);
    assert!(round_tripped.contains_key("providers"));
}
