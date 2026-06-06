use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmActionData {
    pub confirmation_id: String,
    pub prompt_text: String,
    pub confirmed: Option<bool>,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmActionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub prompt_text: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ConfirmActionResolution {
    pub tool_result: ToolResult<ConfirmActionData>,
    pub resume_outcome: ExecutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TtsVoiceName {
    #[serde(rename = "Bella")]
    Bella,
    #[serde(rename = "Jasper")]
    Jasper,
    #[serde(rename = "Luna")]
    Luna,
    #[serde(rename = "Bruno")]
    Bruno,
    #[serde(rename = "Rosie")]
    Rosie,
    #[serde(rename = "Hugo")]
    Hugo,
    #[serde(rename = "Kiki")]
    Kiki,
    #[serde(rename = "Leo")]
    Leo,
    #[serde(rename = "alloy")]
    Alloy,
    #[serde(rename = "ash")]
    Ash,
    #[serde(rename = "ballad")]
    Ballad,
    #[serde(rename = "coral")]
    Coral,
    #[serde(rename = "echo")]
    Echo,
    #[serde(rename = "fable")]
    Fable,
    #[serde(rename = "onyx")]
    Onyx,
    #[serde(rename = "nova")]
    Nova,
    #[serde(rename = "sage")]
    Sage,
    #[serde(rename = "shimmer")]
    Shimmer,
    #[serde(rename = "verse")]
    Verse,
    #[serde(rename = "marin")]
    Marin,
    #[serde(rename = "cedar")]
    Cedar,
}

impl TtsVoiceName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bella => "Bella",
            Self::Jasper => "Jasper",
            Self::Luna => "Luna",
            Self::Bruno => "Bruno",
            Self::Rosie => "Rosie",
            Self::Hugo => "Hugo",
            Self::Kiki => "Kiki",
            Self::Leo => "Leo",
            Self::Alloy => "alloy",
            Self::Ash => "ash",
            Self::Ballad => "ballad",
            Self::Coral => "coral",
            Self::Echo => "echo",
            Self::Fable => "fable",
            Self::Onyx => "onyx",
            Self::Nova => "nova",
            Self::Sage => "sage",
            Self::Shimmer => "shimmer",
            Self::Verse => "verse",
            Self::Marin => "marin",
            Self::Cedar => "cedar",
        }
    }
}

impl std::fmt::Display for TtsVoiceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum NarrationInterruptionMode {
    Queue,
    Interrupt,
}

impl NarrationInterruptionMode {
    pub fn interrupts_current_playback(self) -> bool {
        matches!(self, Self::Interrupt)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum NarrationBoundary {
    None,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ElementVisibilityFilter {
    All,
    VisibleOnly,
}

impl ElementVisibilityFilter {
    pub fn visible_only(self) -> bool {
        matches!(self, Self::VisibleOnly)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ReloadMode {
    Standard,
    Hard,
}

impl ReloadMode {
    pub fn uses_cache_bypass(self) -> bool {
        matches!(self, Self::Hard)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ClickMode {
    Single,
    Double,
}

impl ClickMode {
    pub fn is_double_click(self) -> bool {
        matches!(self, Self::Double)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TextEntryMode {
    Append,
    Replace,
}

impl TextEntryMode {
    pub fn clears_existing_value(self) -> bool {
        matches!(self, Self::Replace)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TextEntrySubmitMode {
    KeepEditing,
    Submit,
}

impl TextEntrySubmitMode {
    pub fn submits_after_entry(self) -> bool {
        matches!(self, Self::Submit)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TranscriptionStopMode {
    KeepListening,
    AutoStop,
}

impl TranscriptionStopMode {
    pub fn auto_stops(self) -> bool {
        matches!(self, Self::AutoStop)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ScreenshotScope {
    Viewport,
    FullPage,
}

impl ScreenshotScope {
    pub fn captures_full_page(self) -> bool {
        matches!(self, Self::FullPage)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ReportStatus {
    Success,
    NeedsFollowUp,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReportResultData {
    pub status: ReportStatus,
    pub summary: String,
    pub next_recommended_action: Option<String>,
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReportResultInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub status: ReportStatus,
    pub summary: String,
    pub next_recommended_action: Option<String>,
    pub user_message: Option<String>,
}
