use super::*;

#[test]
fn infer_intent_hint_prefers_audio_queries_over_setters() {
    assert_eq!(
        infer_intent_hint("what is the volume"),
        IntentName::GetPlaybackVolume
    );
    assert_eq!(
        infer_intent_hint("what's the playback speed"),
        IntentName::GetPlaybackSpeed
    );
    assert_eq!(
        infer_intent_hint("what is the volum"),
        IntentName::GetPlaybackVolume
    );
    assert_eq!(
        infer_intent_hint("what s the play back spead"),
        IntentName::GetPlaybackSpeed
    );
}

#[test]
fn infer_intent_hint_recognizes_browser_visibility_phrases() {
    assert_eq!(
        infer_intent_hint("go headless"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        infer_intent_hint("make the browser visible"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        infer_intent_hint("show the browsr"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        infer_intent_hint("go head less"),
        IntentName::SetBrowserVisibility
    );
}

#[test]
fn infer_intent_hint_recognizes_status_and_history_queries() {
    assert_eq!(infer_intent_hint("can i go back"), IntentName::GetStatus);
    assert_eq!(
        infer_intent_hint("are you listening"),
        IntentName::GetStatus
    );
    assert_eq!(
        infer_intent_hint("what page am i on"),
        IntentName::GetCurrentUrl
    );
    assert_eq!(
        infer_intent_hint("what is the statuz"),
        IntentName::GetStatus
    );
    assert_eq!(infer_intent_hint("are you listenin"), IntentName::GetStatus);
    assert_eq!(
        infer_intent_hint("what is the curent url"),
        IntentName::GetCurrentUrl
    );
}

#[test]
fn infer_intent_hint_recognizes_navigation_readback_action_phrases() {
    assert_eq!(infer_intent_hint("back"), IntentName::GoBack);
    assert_eq!(infer_intent_hint("go forward"), IntentName::GoForward);
    assert_eq!(infer_intent_hint("refesh page"), IntentName::ReloadPage);
    assert_eq!(infer_intent_hint("next"), IntentName::ReadNext);
    assert_eq!(
        infer_intent_hint("prevous region"),
        IntentName::ReadPrevious
    );
    assert_eq!(infer_intent_hint("stpo reading"), IntentName::Stop);
}

#[test]
fn infer_intent_hint_recognizes_voice_input_phrases() {
    assert_eq!(
        infer_intent_hint("start listening"),
        IntentName::StartListening
    );
    assert_eq!(
        infer_intent_hint("stop listenin"),
        IntentName::StopListening
    );
    assert_eq!(
        infer_intent_hint("what did i just say"),
        IntentName::TranscribeCommand
    );
    assert_eq!(
        infer_intent_hint("transcribe this"),
        IntentName::TranscribeCommand
    );
}

#[test]
fn infer_intent_hint_recognizes_open_url_phrases() {
    assert_eq!(
        infer_intent_hint("open github dot com"),
        IntentName::OpenUrl
    );
    assert_eq!(
        infer_intent_hint("go to https://example.com"),
        IntentName::OpenUrl
    );
    assert_eq!(
        infer_intent_hint("visit localhost colon 3000"),
        IntentName::OpenUrl
    );
}

#[test]
fn infer_intent_hint_recognizes_read_page_phrases() {
    assert_eq!(infer_intent_hint("read page"), IntentName::ReadPage);
    assert_eq!(infer_intent_hint("read this page"), IntentName::ReadPage);
    assert_eq!(infer_intent_hint("read current page"), IntentName::ReadPage);
}

#[test]
fn infer_intent_hint_recognizes_form_filling_and_submission_phrases() {
    assert_eq!(
        infer_intent_hint("focus the email field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("fill the password field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("type hello into the search field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("submit this form"),
        IntentName::SubmitForm
    );
    assert_eq!(
        infer_intent_hint("fill the email field and then submit"),
        IntentName::SubmitForm
    );
    assert_eq!(
        infer_intent_hint("no, the other field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("put Seattle there instead"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("choose California from the state list"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("foccus the email feild"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("submitt this form"),
        IntentName::SubmitForm
    );
}

#[test]
fn parse_direct_focus_field_command_extracts_field_description() {
    assert_eq!(
        parse_direct_focus_field_command("focus the email field"),
        Some(FocusFieldCommand {
            description: Some(String::from("email"))
        })
    );
    assert_eq!(
        parse_direct_focus_field_command("foccus the password feild"),
        Some(FocusFieldCommand {
            description: Some(String::from("password"))
        })
    );
    assert_eq!(
        parse_direct_focus_field_command("focus field"),
        Some(FocusFieldCommand { description: None })
    );
    assert_eq!(parse_direct_focus_field_command("read page"), None);
}

#[test]
fn parse_direct_fill_field_command_extracts_description_and_text() {
    assert_eq!(
        parse_direct_fill_field_command("fill the email field with phil@example.com"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: Some(String::from("phil@example.com"))
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("type \"hello world\" into the search field"),
        Some(FillFieldCommand {
            description: Some(String::from("search")),
            text: Some(String::from("hello world"))
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("enter secret in the password field"),
        Some(FillFieldCommand {
            description: Some(String::from("password")),
            text: Some(String::from("secret"))
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("fill the email field"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: None
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("focus the email field"),
        None
    );
}

#[test]
fn parse_fill_field_correction_command_extracts_follow_up_corrections() {
    assert_eq!(
        parse_fill_field_correction_command("no, the other field"),
        Some(FillFieldCorrectionCommand::AlternateField)
    );
    assert_eq!(
        parse_fill_field_correction_command("put Seattle there instead"),
        Some(FillFieldCorrectionCommand::ReplaceValue {
            text: String::from("Seattle")
        })
    );
    assert_eq!(
        parse_fill_field_correction_command("type \"hello world\" there instead"),
        Some(FillFieldCorrectionCommand::ReplaceValue {
            text: String::from("hello world")
        })
    );
    assert_eq!(parse_fill_field_correction_command("read page"), None);
}

#[test]
fn parse_direct_fill_and_submit_command_extracts_description_and_text() {
    assert_eq!(
        parse_direct_fill_and_submit_command(
            "fill the email field with phil@example.com and then submit"
        ),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: Some(String::from("phil@example.com"))
        })
    );
    assert_eq!(
        parse_direct_fill_and_submit_command(
            "type hello world into the search field and submit form"
        ),
        Some(FillFieldCommand {
            description: Some(String::from("search")),
            text: Some(String::from("hello world"))
        })
    );
    assert_eq!(
        parse_direct_fill_and_submit_command("fill the email field and submit"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: None
        })
    );
    assert_eq!(parse_direct_fill_and_submit_command("submit form"), None);
}

#[test]
fn parse_direct_fill_field_command_handles_fill_in_prefix() {
    assert_eq!(
        parse_direct_fill_field_command("fill in the email field with hello"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: Some(String::from("hello")),
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("fill in the first name field"),
        Some(FillFieldCommand {
            description: Some(String::from("first name")),
            text: None,
        })
    );
}

#[test]
fn parse_direct_fill_field_command_handles_put_in_and_enter_in_patterns() {
    assert_eq!(
        parse_direct_fill_field_command("put hello in the search field"),
        Some(FillFieldCommand {
            description: Some(String::from("search")),
            text: Some(String::from("hello")),
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("enter Seattle in the city field"),
        Some(FillFieldCommand {
            description: Some(String::from("city")),
            text: Some(String::from("Seattle")),
        })
    );
}

#[test]
fn parse_direct_fill_field_command_normalizes_textbox_and_input_suffixes() {
    // "fill in" prefix is recognized by is_fill_input_phrase, enabling normalize_field_target
    // to strip the " textbox" / " input" suffix from the field target
    assert_eq!(
        parse_direct_fill_field_command("fill in the name textbox with Alice"),
        Some(FillFieldCommand {
            description: Some(String::from("name")),
            text: Some(String::from("Alice")),
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("fill in the email input with test"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: Some(String::from("test")),
        })
    );
}

#[test]
fn parse_direct_fill_field_command_strips_single_quoted_fill_value() {
    assert_eq!(
        parse_direct_fill_field_command("fill the name field with 'John Doe'"),
        Some(FillFieldCommand {
            description: Some(String::from("name")),
            text: Some(String::from("John Doe")),
        })
    );
}

#[test]
fn is_direct_submit_form_command_detects_submit_phrases() {
    assert!(is_direct_submit_form_command("submit this form"));
    assert!(is_direct_submit_form_command("submit form"));
    assert!(!is_direct_submit_form_command("fill the email field"));
    assert!(!is_direct_submit_form_command("read page"));
    assert!(!is_direct_submit_form_command(""));
}

#[test]
fn parse_direct_fill_and_submit_command_recognizes_additional_submit_suffixes() {
    let expected = Some(FillFieldCommand {
        description: Some(String::from("email")),
        text: Some(String::from("test@example.com")),
    });
    assert_eq!(
        parse_direct_fill_and_submit_command(
            "fill the email field with test@example.com and then press submit"
        ),
        expected.clone()
    );
    assert_eq!(
        parse_direct_fill_and_submit_command(
            "fill the email field with test@example.com and hit submit"
        ),
        expected.clone()
    );
    assert_eq!(
        parse_direct_fill_and_submit_command(
            "fill the email field with test@example.com and submit form"
        ),
        expected
    );
}

#[test]
fn normalize_transcript_for_routing_merges_compound_tokens_and_sanitizes_punctuation() {
    assert_eq!(
        normalize_transcript_for_routing("Go HEAD less, please!!"),
        "go headless please"
    );
    assert_eq!(
        normalize_transcript_for_routing("What'S the PLAY back spead???"),
        "what s the playback speed"
    );
    assert_eq!(
        normalize_transcript_for_routing("focus the e-mail field."),
        "focus the e mail field"
    );
}

#[test]
fn parse_intent_name_value_accepts_cleaned_values_and_rejects_unknown_intents() {
    assert_eq!(
        parse_intent_name_value(" `OpenUrl` ").expect("open url intent should parse"),
        IntentName::OpenUrl
    );
    assert_eq!(
        parse_intent_name_value("\"SetBrowserVisibility\"")
            .expect("browser visibility intent should parse"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        parse_intent_name_value("'Unknown'").expect("unknown sentinel should parse"),
        IntentName::Unknown
    );

    let error =
        parse_intent_name_value("LaunchMissiles").expect_err("unknown intents should be rejected");
    assert!(error.contains("unknown intent tag"));
    assert!(error.contains("LaunchMissiles"));
}

#[test]
fn infer_intent_hint_recognizes_repeat_phrases() {
    assert_eq!(infer_intent_hint("repeat"), IntentName::Repeat);
    assert_eq!(infer_intent_hint("repeat that"), IntentName::Repeat);
    assert_eq!(infer_intent_hint("read that again"), IntentName::Repeat);
    assert_eq!(infer_intent_hint("say that again"), IntentName::Repeat);
}

#[test]
fn infer_intent_hint_recognizes_read_title_phrases() {
    assert_eq!(infer_intent_hint("read title"), IntentName::ReadTitle);
    assert_eq!(
        infer_intent_hint("read the page title"),
        IntentName::ReadTitle
    );
    assert_eq!(
        infer_intent_hint("what is the title"),
        IntentName::ReadTitle
    );
}

#[test]
fn infer_intent_hint_recognizes_tts_voice_phrases() {
    assert_eq!(
        infer_intent_hint("change the voice to Bruno"),
        IntentName::SetTtsVoice
    );
    assert_eq!(
        infer_intent_hint("switch to the Bella voice"),
        IntentName::SetTtsVoice
    );
    assert_eq!(
        infer_intent_hint("use the Hugo voice"),
        IntentName::SetTtsVoice
    );
    assert_eq!(
        infer_intent_hint("set the voise to Luna"),
        IntentName::SetTtsVoice
    );
}

