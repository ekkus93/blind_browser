use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub allowed_tools: Option<Vec<ToolName>>,
    pub intent_tags: Vec<String>,
    pub requires_confirmation: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElementSearchResult {
    pub query: String,
    pub matches: Vec<ElementCandidate>,
    pub elements: Vec<InteractiveElement>,
}

pub(crate) const MAX_SELECTED_PLANNER_SKILLS: usize = 3;
pub(crate) const BUNDLED_SKILLS_MARKDOWN: &str = include_str!("../../../docs/SKILLS.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerSkillSelection {
    pub active_skill_names: Vec<String>,
    pub relevant_skill_summaries: Vec<SkillSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SkillSource {
    Project,
    User,
    Bundled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoadedSkill {
    pub(crate) summary: SkillSummary,
    pub(crate) body: String,
    pub(crate) source: SkillSource,
}

pub fn registered_tools() -> Vec<AvailableTool> {
    use ToolName::*;

    [
        OpenUrl,
        GoBack,
        GoForward,
        ReloadPage,
        GetHtml,
        EvalJs,
        ScrollPage,
        CaptureScreenshot,
        SetBrowserVisibility,
        GetPageSnapshot,
        ExtractPageModel,
        ListInteractiveElements,
        FindElement,
        ClickElement,
        FocusElement,
        TypeIntoElement,
        SubmitActiveForm,
        ReadRegion,
        ReadNextRegion,
        ReadPreviousRegion,
        StopSpeaking,
        StartListening,
        StopListening,
        TranscribeCommand,
        SetTtsVoice,
        SetPlaybackVolume,
        SetPlaybackSpeed,
        RunOcr,
        MergeOcrIntoPageModel,
        GetAgentState,
        GetRuntimeStatus,
        ConfirmAction,
        ReportResult,
    ]
    .into_iter()
    .map(|name| AvailableTool {
        input_schema_ref: format!("schema://tool-input/{name:?}"),
        output_schema_ref: format!("schema://tool-output/{name:?}"),
        description: format!("Deterministic tool contract for {name:?}."),
        name,
    })
    .collect()
}

pub fn planner_available_tools() -> Vec<AvailableTool> {
    registered_tools()
        .into_iter()
        .filter(|tool| is_plannable_tool(&tool.name))
        .collect()
}

pub fn planner_output_schema() -> serde_json::Value {
    schema_json::<PlannerOutput>()
}

pub fn canonical_planner_output_examples() -> BTreeMap<String, PlannerOutput> {
    BTreeMap::from([
        (
            String::from("get_status"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::GetStatus,
                    goal: String::from("Report the current runtime status."),
                    target_description: None,
                },
                selected_skills: vec![String::from("get_status")],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("fetch-runtime-status"),
                        tool_name: ToolName::GetRuntimeStatus,
                        arguments: serde_json::json!({
                            "request_id": "example-get-status",
                            "timeout_ms": null,
                            "include_provider_modes": true
                        }),
                        purpose: String::from("Read the current runtime status before speaking."),
                        on_success: StepTransition::NextStep {
                            step_id: String::from("report-runtime-status"),
                        },
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("report-runtime-status"),
                        tool_name: ToolName::ReportResult,
                        arguments: serde_json::json!({
                            "request_id": "example-get-status",
                            "timeout_ms": null,
                            "status": "Success",
                            "summary": "Browser is visible, listening is idle, and nothing is currently speaking.",
                            "next_recommended_action": null,
                            "user_message": "Browser visible. Listening idle. Not speaking."
                        }),
                        purpose: String::from("Speak a short status summary to the user."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("read_title"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::ReadTitle,
                    goal: String::from("Read the current page title."),
                    target_description: None,
                },
                selected_skills: vec![String::from("read_title")],
                steps: vec![PlannedStep {
                    step_id: String::from("report-page-title"),
                    tool_name: ToolName::ReportResult,
                    arguments: serde_json::json!({
                        "request_id": "example-read-title",
                        "timeout_ms": null,
                        "status": "Success",
                        "summary": "Page title is Example article.",
                        "next_recommended_action": null,
                        "user_message": "Page title is Example article."
                    }),
                    purpose: String::from("Speak the current page title."),
                    on_success: StepTransition::Complete,
                    on_failure: StepTransition::Replan,
                }],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("set_playback_volume"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::SetPlaybackVolume,
                    goal: String::from("Set playback volume to 70%."),
                    target_description: Some(String::from("70%")),
                },
                selected_skills: vec![String::from("set_volume")],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("set-playback-volume"),
                        tool_name: ToolName::SetPlaybackVolume,
                        arguments: serde_json::json!({
                            "request_id": "example-set-volume",
                            "timeout_ms": null,
                            "volume": 0.7
                        }),
                        purpose: String::from("Apply and persist the requested playback volume."),
                        on_success: StepTransition::NextStep {
                            step_id: String::from("report-playback-volume"),
                        },
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("report-playback-volume"),
                        tool_name: ToolName::ReportResult,
                        arguments: serde_json::json!({
                            "request_id": "example-set-volume",
                            "timeout_ms": null,
                            "status": "Success",
                            "summary": "Playback volume set to 70%.",
                            "next_recommended_action": null,
                            "user_message": "Playback volume set to 70%."
                        }),
                        purpose: String::from("Confirm the updated playback volume."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("click_element_ready"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::ClickElement,
                    goal: String::from("Open the help link."),
                    target_description: Some(String::from("help link")),
                },
                selected_skills: vec![String::from("open_link_by_text")],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("click-help-link"),
                        tool_name: ToolName::ClickElement,
                        arguments: serde_json::json!({
                            "request_id": "example-click-link",
                            "timeout_ms": null,
                            "element_id": "link-help",
                            "click_mode": "Single"
                        }),
                        purpose: String::from(
                            "Activate the requested link without an extra confirmation step.",
                        ),
                        on_success: StepTransition::NextStep {
                            step_id: String::from("report-click-link"),
                        },
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("report-click-link"),
                        tool_name: ToolName::ReportResult,
                        arguments: serde_json::json!({
                            "request_id": "example-click-link",
                            "timeout_ms": null,
                            "status": "Success",
                            "summary": "Activated the help link.",
                            "next_recommended_action": null,
                            "user_message": "Opened the help link."
                        }),
                        purpose: String::from("Confirm the ordinary click action to the user."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("click_element_with_confirmation"),
            PlannerOutput {
                status: PlannerStatus::NeedsConfirmation,
                intent: IntentSummary {
                    name: IntentName::ClickElement,
                    goal: String::from("Open the submit button after confirmation."),
                    target_description: Some(String::from("submit button")),
                },
                selected_skills: vec![
                    String::from("open_link_by_text"),
                    String::from("confirm_action"),
                ],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("confirm-click-target"),
                        tool_name: ToolName::ConfirmAction,
                        arguments: serde_json::json!({
                            "request_id": "example-confirm-click",
                            "timeout_ms": null,
                            "prompt_text": "Do you want me to activate the submit button?",
                            "reason": "The requested click may submit data or navigate away."
                        }),
                        purpose: String::from("Ask for confirmation before the protected click."),
                        on_success: StepTransition::RequestConfirmation,
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("click-submit-button"),
                        tool_name: ToolName::ClickElement,
                        arguments: serde_json::json!({
                            "request_id": "example-confirm-click",
                            "timeout_ms": null,
                            "element_id": "button-submit",
                            "click_mode": "Single"
                        }),
                        purpose: String::from("Activate the confirmed target element."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: true,
                confirmation_reason: Some(String::from(
                    "Clicking the submit button may send data or change page context.",
                )),
                blocked_reason: None,
                user_message: Some(String::from(
                    "Please confirm before I activate the submit button.",
                )),
            },
        ),
    ])
}

pub fn tool_input_schema(tool_name: &ToolName) -> Option<serde_json::Value> {
    match tool_name {
        ToolName::OpenUrl => Some(schema_json::<OpenUrlInput>()),
        ToolName::GoBack => Some(schema_json::<GoBackInput>()),
        ToolName::GoForward => Some(schema_json::<GoForwardInput>()),
        ToolName::ReloadPage => Some(schema_json::<ReloadPageInput>()),
        ToolName::GetHtml => Some(schema_json::<GetHtmlInput>()),
        ToolName::EvalJs => Some(schema_json::<EvalJsInput>()),
        ToolName::ScrollPage => Some(schema_json::<ScrollPageInput>()),
        ToolName::CaptureScreenshot => Some(schema_json::<CaptureScreenshotInput>()),
        ToolName::RunOcr => Some(schema_json::<RunOcrInput>()),
        ToolName::MergeOcrIntoPageModel => Some(schema_json::<MergeOcrIntoPageModelInput>()),
        ToolName::SetBrowserVisibility => Some(schema_json::<SetBrowserVisibilityInput>()),
        ToolName::GetPageSnapshot => Some(schema_json::<GetPageSnapshotInput>()),
        ToolName::ExtractPageModel => Some(schema_json::<ExtractPageModelInput>()),
        ToolName::ListInteractiveElements => Some(schema_json::<ListInteractiveElementsInput>()),
        ToolName::FindElement => Some(schema_json::<FindElementInput>()),
        ToolName::ClickElement => Some(schema_json::<ClickElementInput>()),
        ToolName::FocusElement => Some(schema_json::<FocusElementInput>()),
        ToolName::TypeIntoElement => Some(schema_json::<TypeIntoElementInput>()),
        ToolName::SubmitActiveForm => Some(schema_json::<SubmitActiveFormInput>()),
        ToolName::ReadRegion => Some(schema_json::<ReadRegionInput>()),
        ToolName::ReadNextRegion => Some(schema_json::<ReadNextRegionInput>()),
        ToolName::ReadPreviousRegion => Some(schema_json::<ReadPreviousRegionInput>()),
        ToolName::StopSpeaking => Some(schema_json::<StopSpeakingInput>()),
        ToolName::StartListening => Some(schema_json::<StartListeningInput>()),
        ToolName::StopListening => Some(schema_json::<StopListeningInput>()),
        ToolName::TranscribeCommand => Some(schema_json::<TranscribeCommandInput>()),
        ToolName::SetTtsVoice => Some(schema_json::<SetTtsVoiceInput>()),
        ToolName::SetPlaybackVolume => Some(schema_json::<SetPlaybackVolumeInput>()),
        ToolName::SetPlaybackSpeed => Some(schema_json::<SetPlaybackSpeedInput>()),
        ToolName::GetAgentState => Some(schema_json::<GetAgentStateInput>()),
        ToolName::GetRuntimeStatus => Some(schema_json::<GetRuntimeStatusInput>()),
        ToolName::ConfirmAction => Some(schema_json::<ConfirmActionInput>()),
        ToolName::ReportResult => Some(schema_json::<ReportResultInput>()),
    }
}

pub fn tool_output_schema(tool_name: &ToolName) -> Option<serde_json::Value> {
    match tool_name {
        ToolName::OpenUrl => Some(schema_json::<ToolResult<OpenUrlData>>()),
        ToolName::GoBack => Some(schema_json::<ToolResult<GoBackData>>()),
        ToolName::GoForward => Some(schema_json::<ToolResult<GoForwardData>>()),
        ToolName::ReloadPage => Some(schema_json::<ToolResult<ReloadPageData>>()),
        ToolName::GetHtml => Some(schema_json::<ToolResult<GetHtmlData>>()),
        ToolName::EvalJs => Some(schema_json::<ToolResult<EvalJsData>>()),
        ToolName::ScrollPage => Some(schema_json::<ToolResult<ScrollPageData>>()),
        ToolName::CaptureScreenshot => Some(schema_json::<ToolResult<CaptureScreenshotData>>()),
        ToolName::RunOcr => Some(schema_json::<ToolResult<RunOcrData>>()),
        ToolName::MergeOcrIntoPageModel => {
            Some(schema_json::<ToolResult<MergeOcrIntoPageModelData>>())
        }
        ToolName::SetBrowserVisibility => {
            Some(schema_json::<ToolResult<SetBrowserVisibilityData>>())
        }
        ToolName::GetPageSnapshot => Some(schema_json::<ToolResult<PageSnapshotData>>()),
        ToolName::ExtractPageModel => Some(schema_json::<ToolResult<ExtractPageModelData>>()),
        ToolName::ListInteractiveElements => {
            Some(schema_json::<ToolResult<ListInteractiveElementsData>>())
        }
        ToolName::FindElement => Some(schema_json::<ToolResult<FindElementData>>()),
        ToolName::ClickElement => Some(schema_json::<ToolResult<ClickElementData>>()),
        ToolName::FocusElement => Some(schema_json::<ToolResult<FocusElementData>>()),
        ToolName::TypeIntoElement => Some(schema_json::<ToolResult<TypeIntoElementData>>()),
        ToolName::SubmitActiveForm => Some(schema_json::<ToolResult<SubmitActiveFormData>>()),
        ToolName::ReadRegion => Some(schema_json::<ToolResult<ReadRegionData>>()),
        ToolName::ReadNextRegion => Some(schema_json::<ToolResult<ReadNextRegionData>>()),
        ToolName::ReadPreviousRegion => Some(schema_json::<ToolResult<ReadPreviousRegionData>>()),
        ToolName::StopSpeaking => Some(schema_json::<ToolResult<StopSpeakingData>>()),
        ToolName::StartListening => Some(schema_json::<ToolResult<StartListeningData>>()),
        ToolName::StopListening => Some(schema_json::<ToolResult<StopListeningData>>()),
        ToolName::TranscribeCommand => Some(schema_json::<ToolResult<TranscribeCommandData>>()),
        ToolName::SetTtsVoice => Some(schema_json::<ToolResult<SetTtsVoiceData>>()),
        ToolName::SetPlaybackVolume => Some(schema_json::<ToolResult<SetPlaybackVolumeData>>()),
        ToolName::SetPlaybackSpeed => Some(schema_json::<ToolResult<SetPlaybackSpeedData>>()),
        ToolName::GetAgentState => Some(schema_json::<ToolResult<AgentStateData>>()),
        ToolName::GetRuntimeStatus => Some(schema_json::<ToolResult<GetRuntimeStatusData>>()),
        ToolName::ConfirmAction => Some(schema_json::<ToolResult<ConfirmActionData>>()),
        ToolName::ReportResult => Some(schema_json::<ToolResult<ReportResultData>>()),
    }
}

pub fn build_planner_skill_selection(
    project_root: Option<&Path>,
    user_skill_root: Option<&Path>,
    transcript: &str,
    available_tools: &[AvailableTool],
) -> PlannerSkillSelection {
    let loaded_skills = discover_skills(project_root, user_skill_root, available_tools);
    let mut active_skill_names = loaded_skills
        .iter()
        .map(|skill| skill.summary.name.clone())
        .collect::<Vec<_>>();
    active_skill_names.sort();

    let inferred_intent = infer_intent_hint(transcript);
    let likely_tools = likely_tools_for_intent(&inferred_intent);
    let transcript_tokens = tokenize_text(transcript);

    let mut ranked_skills = loaded_skills
        .into_iter()
        .filter_map(|skill| {
            score_skill(&skill, &transcript_tokens, &inferred_intent, &likely_tools)
                .map(|score| (score, skill.summary))
        })
        .collect::<Vec<_>>();

    ranked_skills.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.name.cmp(&right.1.name))
    });

    let relevant_skill_summaries = ranked_skills
        .into_iter()
        .take(MAX_SELECTED_PLANNER_SKILLS)
        .map(|(_, summary)| summary)
        .collect();

    PlannerSkillSelection {
        active_skill_names,
        relevant_skill_summaries,
    }
}

fn is_plannable_tool(tool_name: &ToolName) -> bool {
    matches!(
        tool_name,
        ToolName::OpenUrl
            | ToolName::GoBack
            | ToolName::GoForward
            | ToolName::ReloadPage
            | ToolName::GetHtml
            | ToolName::ScrollPage
            | ToolName::CaptureScreenshot
            | ToolName::RunOcr
            | ToolName::MergeOcrIntoPageModel
            | ToolName::SetBrowserVisibility
            | ToolName::GetPageSnapshot
            | ToolName::ExtractPageModel
            | ToolName::ListInteractiveElements
            | ToolName::FindElement
            | ToolName::ClickElement
            | ToolName::FocusElement
            | ToolName::TypeIntoElement
            | ToolName::SubmitActiveForm
            | ToolName::ReadRegion
            | ToolName::ReadNextRegion
            | ToolName::ReadPreviousRegion
            | ToolName::StopSpeaking
            | ToolName::StartListening
            | ToolName::StopListening
            | ToolName::TranscribeCommand
            | ToolName::SetTtsVoice
            | ToolName::SetPlaybackVolume
            | ToolName::SetPlaybackSpeed
            | ToolName::GetAgentState
            | ToolName::GetRuntimeStatus
            | ToolName::ConfirmAction
            | ToolName::ReportResult
    )
}

pub(crate) fn schema_json<T: JsonSchema>() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(T)).expect("schema generation should serialize")
}

pub(crate) fn discover_skills(
    project_root: Option<&Path>,
    user_skill_root: Option<&Path>,
    available_tools: &[AvailableTool],
) -> Vec<LoadedSkill> {
    let available_tool_names = available_tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let mut discovered = HashMap::<String, LoadedSkill>::new();

    if let Some(project_root) = project_root {
        load_skills_from_directory(
            &project_root.join(".pi").join("skills"),
            SkillSource::Project,
            &available_tool_names,
            &mut discovered,
        );
    }

    if let Some(user_skill_root) = user_skill_root {
        load_skills_from_directory(
            user_skill_root,
            SkillSource::User,
            &available_tool_names,
            &mut discovered,
        );
    }

    for skill in parse_bundled_skills(BUNDLED_SKILLS_MARKDOWN, &available_tool_names) {
        discovered
            .entry(skill.summary.name.clone())
            .or_insert(skill);
    }

    discovered.into_values().collect()
}

fn load_skills_from_directory(
    skill_root: &Path,
    source: SkillSource,
    available_tool_names: &[ToolName],
    discovered: &mut HashMap<String, LoadedSkill>,
) {
    let entries = match fs::read_dir(skill_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            tracing::warn!(
                path = %skill_root.display(),
                error = %error,
                "failed to read skill directory"
            );
            return;
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_file_path = path.join("SKILL.md");
        let content = match fs::read_to_string(&skill_file_path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                tracing::warn!(
                    path = %skill_file_path.display(),
                    error = %error,
                    "failed to read SKILL.md"
                );
                continue;
            }
        };

        match parse_skill_document(&content, source, available_tool_names) {
            Ok(skill) => {
                let directory_name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                if directory_name != skill.summary.name {
                    tracing::warn!(
                        path = %skill_file_path.display(),
                        expected = directory_name,
                        actual = %skill.summary.name,
                        "skipping skill because directory name does not match frontmatter name"
                    );
                    continue;
                }
                discovered
                    .entry(skill.summary.name.clone())
                    .or_insert(skill);
            }
            Err(error) => {
                tracing::warn!(
                    path = %skill_file_path.display(),
                    error = %error,
                    "skipping invalid skill document"
                );
            }
        }
    }
}

pub(crate) fn parse_skill_document(
    content: &str,
    source: SkillSource,
    available_tool_names: &[ToolName],
) -> Result<LoadedSkill, String> {
    let normalized = content.replace("\r\n", "\n");
    let Some(frontmatter_body) = normalized.strip_prefix("---\n") else {
        return Err(String::from("SKILL.md is missing a YAML frontmatter block"));
    };
    let Some(split_index) = frontmatter_body.find("\n---\n") else {
        return Err(String::from("SKILL.md frontmatter block is not terminated"));
    };

    let frontmatter_block = &frontmatter_body[..split_index];
    let body = frontmatter_body[(split_index + 5)..].trim().to_string();
    let frontmatter = parse_skill_frontmatter(frontmatter_block, available_tool_names)?;
    Ok(LoadedSkill {
        summary: skill_summary_from_frontmatter(frontmatter),
        body,
        source,
    })
}

fn parse_skill_frontmatter(
    block: &str,
    available_tool_names: &[ToolName],
) -> Result<SkillFrontmatter, String> {
    let mut scalar_fields = HashMap::<String, String>::new();
    let mut list_fields = HashMap::<String, Vec<String>>::new();
    let mut active_list_key: Option<String> = None;

    for raw_line in block.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(list_key) = active_list_key.as_ref() {
            if let Some(item) = trimmed.strip_prefix("- ") {
                list_fields
                    .entry(list_key.clone())
                    .or_default()
                    .push(clean_skill_value(item));
                continue;
            }
            active_list_key = None;
        }

        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!("invalid frontmatter line '{trimmed}'"));
        };
        let key = normalize_skill_key(key.trim());
        let value = value.trim();

        match key.as_str() {
            "name" | "description" | "requires_confirmation" | "priority" => {
                scalar_fields.insert(key, clean_skill_value(value));
            }
            "allowed_tools" | "intent_tags" => {
                let list = list_fields.entry(key.clone()).or_default();
                list.extend(parse_inline_list(value));
                active_list_key = Some(key);
            }
            _ => return Err(format!("unsupported frontmatter field '{key}'")),
        }
    }

    skill_frontmatter_from_parts(scalar_fields, list_fields, available_tool_names)
}

pub(crate) fn parse_bundled_skills(
    markdown: &str,
    available_tool_names: &[ToolName],
) -> Vec<LoadedSkill> {
    let mut current_name: Option<String> = None;
    let mut description = String::new();
    let mut intent_tags = Vec::new();
    let mut allowed_tools = Vec::new();
    let mut skills = Vec::new();

    let flush_skill = |skills: &mut Vec<LoadedSkill>,
                       current_name: &mut Option<String>,
                       description: &mut String,
                       intent_tags: &mut Vec<String>,
                       allowed_tools: &mut Vec<String>,
                       requires_confirmation: bool| {
        let Some(name) = current_name.take() else {
            return;
        };

        let mut scalar_fields = HashMap::new();
        scalar_fields.insert(String::from("name"), name);
        scalar_fields.insert(String::from("description"), description.trim().to_string());
        scalar_fields.insert(
            String::from("requires_confirmation"),
            requires_confirmation.to_string(),
        );
        let mut list_fields = HashMap::new();
        list_fields.insert(String::from("intent_tags"), intent_tags.clone());
        list_fields.insert(String::from("allowed_tools"), allowed_tools.clone());

        match skill_frontmatter_from_parts(scalar_fields, list_fields, available_tool_names) {
            Ok(frontmatter) => skills.push(LoadedSkill {
                summary: skill_summary_from_frontmatter(frontmatter),
                body: description.trim().to_string(),
                source: SkillSource::Bundled,
            }),
            Err(error) => {
                tracing::warn!(skill_name = %skills.last().map(|skill| skill.summary.name.as_str()).unwrap_or("unknown"), error = %error, "skipping invalid bundled skill");
            }
        }

        description.clear();
        intent_tags.clear();
        allowed_tools.clear();
    };

    let mut requires_confirmation_value = false;
    for raw_line in markdown.lines() {
        let trimmed = raw_line.trim();
        if let Some(name) = trimmed.strip_prefix("#### ") {
            flush_skill(
                &mut skills,
                &mut current_name,
                &mut description,
                &mut intent_tags,
                &mut allowed_tools,
                requires_confirmation_value,
            );
            current_name = Some(name.trim().to_string());
            requires_confirmation_value = false;
            continue;
        }

        if current_name.is_none() {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- intent_tags:") {
            intent_tags = parse_backticked_list(value);
        } else if let Some(value) = trimmed.strip_prefix("- allowed_tools:") {
            allowed_tools = parse_backticked_list(value);
        } else if let Some(value) = trimmed.strip_prefix("- requires_confirmation:") {
            requires_confirmation_value = parse_bool_value(value).unwrap_or(false);
        } else if let Some(value) = trimmed.strip_prefix("- description:") {
            description = clean_skill_value(value);
        }
    }

    flush_skill(
        &mut skills,
        &mut current_name,
        &mut description,
        &mut intent_tags,
        &mut allowed_tools,
        requires_confirmation_value,
    );

    skills
}

fn skill_frontmatter_from_parts(
    scalar_fields: HashMap<String, String>,
    list_fields: HashMap<String, Vec<String>>,
    available_tool_names: &[ToolName],
) -> Result<SkillFrontmatter, String> {
    let name = scalar_fields
        .get("name")
        .ok_or_else(|| String::from("skill frontmatter is missing name"))?
        .trim()
        .to_string();
    if !is_valid_skill_name(&name) {
        return Err(format!("invalid skill name '{name}'"));
    }

    let description = scalar_fields
        .get("description")
        .ok_or_else(|| String::from("skill frontmatter is missing description"))?
        .trim()
        .to_string();
    if description.is_empty() {
        return Err(String::from("skill description must not be empty"));
    }

    let mut intent_tags = list_fields
        .get("intent_tags")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    intent_tags.sort();
    intent_tags.dedup();
    for tag in &intent_tags {
        if let Some(intent_name) = tag.strip_prefix("intent:") {
            parse_intent_name_value(intent_name)?;
        }
    }

    let allowed_tools = match list_fields.get("allowed_tools") {
        Some(tool_names) if !tool_names.is_empty() => {
            let mut resolved_tools = Vec::new();
            for tool_name in tool_names {
                let tool = parse_tool_name_value(tool_name)?;
                if !available_tool_names
                    .iter()
                    .any(|available| available == &tool)
                {
                    return Err(format!("skill references unavailable tool '{tool_name}'"));
                }
                resolved_tools.push(tool);
            }
            Some(resolved_tools)
        }
        _ => None,
    };

    let requires_confirmation = scalar_fields
        .get("requires_confirmation")
        .map(|value| parse_bool_value(value))
        .transpose()?
        .unwrap_or(false);

    let priority = scalar_fields
        .get("priority")
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|error| format!("invalid priority value '{value}': {error}"))
        })
        .transpose()?
        .unwrap_or(0);

    Ok(SkillFrontmatter {
        name,
        description,
        allowed_tools,
        intent_tags,
        requires_confirmation,
        priority,
    })
}

fn skill_summary_from_frontmatter(frontmatter: SkillFrontmatter) -> SkillSummary {
    SkillSummary {
        name: frontmatter.name,
        description: frontmatter.description,
        intent_tags: frontmatter.intent_tags,
        allowed_tools: frontmatter.allowed_tools,
        requires_confirmation: frontmatter.requires_confirmation,
        priority: frontmatter.priority,
    }
}

fn normalize_skill_key(key: &str) -> String {
    key.trim().replace('-', "_")
}

fn parse_inline_list(value: &str) -> Vec<String> {
    let cleaned = clean_skill_value(value);
    if cleaned.is_empty() {
        return Vec::new();
    }

    let trimmed = cleaned.trim_matches(['[', ']']);
    trimmed
        .split(',')
        .map(clean_skill_value)
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_backticked_list(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_tick = false;
    for character in value.chars() {
        match character {
            '`' => {
                if in_tick {
                    items.push(current.trim().to_string());
                    current.clear();
                }
                in_tick = !in_tick;
            }
            _ if in_tick => current.push(character),
            _ => {}
        }
    }

    if items.is_empty() {
        parse_inline_list(value)
    } else {
        items
    }
}

fn parse_bool_value(value: &str) -> Result<bool, String> {
    match clean_skill_value(value).as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("invalid boolean value '{other}'")),
    }
}

fn clean_skill_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn is_valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_'
        })
}

fn parse_tool_name_value(value: &str) -> Result<ToolName, String> {
    match clean_skill_value(value).as_str() {
        "open_url" => Ok(ToolName::OpenUrl),
        "go_back" => Ok(ToolName::GoBack),
        "go_forward" => Ok(ToolName::GoForward),
        "reload_page" => Ok(ToolName::ReloadPage),
        "get_html" => Ok(ToolName::GetHtml),
        "eval_js" => Ok(ToolName::EvalJs),
        "scroll_page" => Ok(ToolName::ScrollPage),
        "capture_screenshot" => Ok(ToolName::CaptureScreenshot),
        "set_browser_visibility" => Ok(ToolName::SetBrowserVisibility),
        "get_page_snapshot" => Ok(ToolName::GetPageSnapshot),
        "extract_page_model" => Ok(ToolName::ExtractPageModel),
        "list_interactive_elements" => Ok(ToolName::ListInteractiveElements),
        "find_element" => Ok(ToolName::FindElement),
        "click_element" => Ok(ToolName::ClickElement),
        "focus_element" => Ok(ToolName::FocusElement),
        "type_into_element" => Ok(ToolName::TypeIntoElement),
        "submit_active_form" => Ok(ToolName::SubmitActiveForm),
        "read_region" => Ok(ToolName::ReadRegion),
        "read_next_region" => Ok(ToolName::ReadNextRegion),
        "read_previous_region" => Ok(ToolName::ReadPreviousRegion),
        "stop_speaking" => Ok(ToolName::StopSpeaking),
        "start_listening" => Ok(ToolName::StartListening),
        "stop_listening" => Ok(ToolName::StopListening),
        "transcribe_command" => Ok(ToolName::TranscribeCommand),
        "set_tts_voice" => Ok(ToolName::SetTtsVoice),
        "set_playback_volume" => Ok(ToolName::SetPlaybackVolume),
        "set_playback_speed" => Ok(ToolName::SetPlaybackSpeed),
        "run_ocr" => Ok(ToolName::RunOcr),
        "merge_ocr_into_page_model" => Ok(ToolName::MergeOcrIntoPageModel),
        "get_agent_state" => Ok(ToolName::GetAgentState),
        "get_runtime_status" => Ok(ToolName::GetRuntimeStatus),
        "confirm_action" => Ok(ToolName::ConfirmAction),
        "report_result" => Ok(ToolName::ReportResult),
        other => Err(format!("unknown tool '{other}'")),
    }
}

pub(crate) fn parse_intent_name_value(value: &str) -> Result<IntentName, String> {
    match clean_skill_value(value).as_str() {
        "OpenUrl" => Ok(IntentName::OpenUrl),
        "GoBack" => Ok(IntentName::GoBack),
        "GoForward" => Ok(IntentName::GoForward),
        "ReloadPage" => Ok(IntentName::ReloadPage),
        "GetCurrentUrl" => Ok(IntentName::GetCurrentUrl),
        "ReadPage" => Ok(IntentName::ReadPage),
        "ReadTitle" => Ok(IntentName::ReadTitle),
        "ReadNext" => Ok(IntentName::ReadNext),
        "ReadPrevious" => Ok(IntentName::ReadPrevious),
        "Repeat" => Ok(IntentName::Repeat),
        "Stop" => Ok(IntentName::Stop),
        "StartListening" => Ok(IntentName::StartListening),
        "StopListening" => Ok(IntentName::StopListening),
        "TranscribeCommand" => Ok(IntentName::TranscribeCommand),
        "SetTtsVoice" => Ok(IntentName::SetTtsVoice),
        "SetPlaybackVolume" => Ok(IntentName::SetPlaybackVolume),
        "GetPlaybackVolume" => Ok(IntentName::GetPlaybackVolume),
        "SetPlaybackSpeed" => Ok(IntentName::SetPlaybackSpeed),
        "GetPlaybackSpeed" => Ok(IntentName::GetPlaybackSpeed),
        "SetBrowserVisibility" => Ok(IntentName::SetBrowserVisibility),
        "GetStatus" => Ok(IntentName::GetStatus),
        "FindElement" => Ok(IntentName::FindElement),
        "ClickElement" => Ok(IntentName::ClickElement),
        "FillInput" => Ok(IntentName::FillInput),
        "SubmitForm" => Ok(IntentName::SubmitForm),
        "Scroll" => Ok(IntentName::Scroll),
        "OcrRecovery" => Ok(IntentName::OcrRecovery),
        "Unknown" => Ok(IntentName::Unknown),
        other => Err(format!("unknown intent tag '{other}'")),
    }
}

fn score_skill(
    skill: &LoadedSkill,
    transcript_tokens: &HashSet<String>,
    inferred_intent: &IntentName,
    likely_tools: &[ToolName],
) -> Option<i32> {
    let skill_tokens = tokenize_text(&format!(
        "{} {} {} {}",
        skill.summary.name,
        skill.summary.description,
        skill.summary.intent_tags.join(" "),
        skill.body
    ));
    let lexical_overlap = transcript_tokens.intersection(&skill_tokens).count() as i32;
    let intent_match = skill
        .summary
        .intent_tags
        .iter()
        .any(|tag| tag == &format!("intent:{inferred_intent:?}"));
    let tool_overlap = skill
        .summary
        .allowed_tools
        .as_ref()
        .map(|tools| {
            tools
                .iter()
                .filter(|tool| likely_tools.iter().any(|candidate| candidate == *tool))
                .count() as i32
        })
        .unwrap_or(0);

    if lexical_overlap == 0 && !intent_match && tool_overlap == 0 {
        return None;
    }

    let precedence_score = match skill.source {
        SkillSource::Project => 3_000,
        SkillSource::User => 2_000,
        SkillSource::Bundled => 1_000,
    };

    Some(
        precedence_score
            + skill.summary.priority
            + (lexical_overlap * 75)
            + (tool_overlap * 100)
            + if intent_match { 500 } else { 0 },
    )
}
