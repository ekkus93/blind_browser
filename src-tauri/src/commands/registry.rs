use super::*;
use super::skill_loader::discover_skills;
use super::skill_parser::score_skill;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerSkillSelection {
    pub active_skill_names: Vec<String>,
    pub relevant_skill_summaries: Vec<SkillSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillSource {
    Project,
    User,
    Bundled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSkill {
    pub summary: SkillSummary,
    pub body: String,
    pub source: SkillSource,
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
